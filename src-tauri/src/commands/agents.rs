use crate::commands::common::{
    get_agents_dir, get_agent_workspace_dir, join_path, load_openclaw_config,
    path_separator, save_openclaw_config,
};
use crate::models::{
    AgentBinding, AgentBindingMatch, AgentBindingsResult, AgentChannelBinding, AgentConfig,
    AgentIdentityConfig, AgentInfo, AgentModelConfigFull, AgentsListResult, CreateAgentParams,
    DeleteAgentParams, SetAgentBindingsParams, SetDefaultAgentParams, UpdateAgentParams,
};
use crate::utils::file;
use log::{error, info, warn};
use rand::Rng;
use serde_json::{json, Value};
use std::fs;
use std::path::Path;
use tauri::command;

const DEFAULT_AGENT_ID: &str = "default";
const BUILTIN_AGENT_IDS: &[&str] = &["default"];
const IDENTITY_FILENAME: &str = "IDENTITY.md";
const SOUL_FILENAME: &str = "SOUL.md";
const AGENTS_FILENAME: &str = "AGENTS.md";
const USER_FILENAME: &str = "USER.md";
const MEMORY_FILENAME: &str = "MEMORY.md";

fn generate_numeric_agent_id() -> String {
    let mut rng = rand::thread_rng();
    let id: u64 = rng.gen_range(10000..999999999);
    id.to_string()
}

fn get_default_identity_template(name: &str, emoji: Option<&str>) -> String {
    let emoji_line = if let Some(e) = emoji {
        format!("- Emoji: {}\n", e)
    } else {
        String::new()
    };
    format!(
        "# IDENTITY.md - Agent Identity\n\n\
        - Name: {}\n\
        {}\
        - Description:\n\
        \n\
        ## 基础信息\n\
        - Role:\n\
        - Expertise:\n\
        - Goals:\n",
        name, emoji_line
    )
}

fn get_default_soul_template() -> String {
    "# SOUL.md - Persona & Boundaries\n\n\
    ## 人格特质\n\
    - Tone:\n\
    - Communication style:\n\
    - Personality:\n\n\
    ## 行为边界\n\
    - Can do:\n\
    - Cannot do:\n\
    - Limitations:\n\n\
    ## 语气风格\n\
    - Greeting style:\n\
    - Response length:\n\
    - Use of emojis:\n".to_string()
}

fn get_default_agents_template() -> String {
    "# AGENTS.md - Capabilities & Safety\n\n\
    ## 能力范围\n\
    - Primary functions:\n\
    - Specialized skills:\n\
    - Tools available:\n\n\
    ## 安全准则\n\
    - Data privacy:\n\
    - Content restrictions:\n\
    - Verification requirements:\n\n\
    ## 工作流程\n\
    - Task prioritization:\n\
    - Error handling:\n\
    - Escalation:\n".to_string()
}

fn get_default_user_template() -> String {
    "# USER.md - User Profile\n\n\
    ## 基础信息\n\
    - Name:\n\
    - Preferred address:\n\
    - Pronouns (optional):\n\
    - Timezone (optional):\n\n\
    ## 偏好设置\n\
    - Communication preferences:\n\
    - Topic interests:\n\
    - Feedback style:\n\n\
    ## 备注\n\
    - Notes:\n".to_string()
}

fn get_default_memory_template() -> String {
    "# MEMORY.md - Interaction History\n\n\
    ## 重要记录\n\
    ### 首次互动\n\
    - Date:\n\
    - Context:\n\n\
    ## 偏好历史\n\
    ## 重要事项\n\
    ## 待跟进\n".to_string()
}

fn ensure_agent_workspace(workspace_dir: &str, name: &str, emoji: Option<&str>) -> Result<(), String> {
    let path = Path::new(workspace_dir);
    if !path.exists() {
        fs::create_dir_all(path)
            .map_err(|e| format!("创建工作区目录失败: {}", e))?;
    }

    let sep = path_separator();

    let identity_path = format!("{}{}{}", workspace_dir, sep, IDENTITY_FILENAME);
    if !Path::new(&identity_path).exists() {
        let content = get_default_identity_template(name, emoji);
        file::write_file(&identity_path, &content)
            .map_err(|e| format!("创建 IDENTITY.md 失败: {}", e))?;
    }

    let soul_path = format!("{}{}{}", workspace_dir, sep, SOUL_FILENAME);
    if !Path::new(&soul_path).exists() {
        let content = get_default_soul_template();
        file::write_file(&soul_path, &content)
            .map_err(|e| format!("创建 SOUL.md 失败: {}", e))?;
    }

    let agents_path = format!("{}{}{}", workspace_dir, sep, AGENTS_FILENAME);
    if !Path::new(&agents_path).exists() {
        let content = get_default_agents_template();
        file::write_file(&agents_path, &content)
            .map_err(|e| format!("创建 AGENTS.md 失败: {}", e))?;
    }

    let user_path = format!("{}{}{}", workspace_dir, sep, USER_FILENAME);
    if !Path::new(&user_path).exists() {
        let content = get_default_user_template();
        file::write_file(&user_path, &content)
            .map_err(|e| format!("创建 USER.md 失败: {}", e))?;
    }

    let memory_path = format!("{}{}{}", workspace_dir, sep, MEMORY_FILENAME);
    if !Path::new(&memory_path).exists() {
        let content = get_default_memory_template();
        file::write_file(&memory_path, &content)
            .map_err(|e| format!("创建 MEMORY.md 失败: {}", e))?;
    }

    Ok(())
}

fn update_identity_file(
    workspace_dir: &str,
    name: &str,
    emoji: Option<&str>,
    description: Option<&str>,
) -> Result<(), String> {
    let identity_path = format!("{}{}{}", workspace_dir, path_separator(), IDENTITY_FILENAME);

    let mut content = String::new();
    content.push_str("# Agent Identity\n\n");
    content.push_str(&format!("- Name: {}\n", name));

    if let Some(e) = emoji {
        content.push_str(&format!("- Emoji: {}\n", e));
    }

    if let Some(d) = description {
        content.push_str(&format!("- Description: {}\n", d));
    }

    file::write_file(&identity_path, &content)
        .map_err(|e| format!("更新 IDENTITY.md 失败: {}", e))
}

#[derive(Debug, Clone, Default)]
struct IdentityFileInfo {
    name: Option<String>,
    emoji: Option<String>,
    description: Option<String>,
    avatar: Option<String>,
}

fn parse_identity_markdown(content: &str) -> IdentityFileInfo {
    let mut identity = IdentityFileInfo::default();
    
    for line in content.lines() {
        let cleaned = line.trim().trim_start_matches('-').trim();
        let colon_index = cleaned.find(':');
        
        if colon_index.is_none() {
            continue;
        }
        
        let colon_idx = colon_index.unwrap();
        let label = cleaned[..colon_idx]
            .replace('*', "")
            .replace('_', "")
            .trim()
            .to_lowercase();
        let value = cleaned[colon_idx + 1..]
            .trim_start_matches('*')
            .trim_end_matches('*')
            .trim()
            .to_string();
        
        if value.is_empty() {
            continue;
        }
        
        match label.as_str() {
            "name" => identity.name = Some(value),
            "emoji" => identity.emoji = Some(value),
            "description" => identity.description = Some(value),
            "avatar" => identity.avatar = Some(value),
            _ => {}
        }
    }
    
    identity
}

fn load_identity_from_workspace(workspace_dir: &str) -> Option<IdentityFileInfo> {
    let identity_path = format!("{}{}{}", workspace_dir, path_separator(), IDENTITY_FILENAME);
    
    if !file::file_exists(&identity_path) {
        return None;
    }
    
    let content = file::read_file(&identity_path).ok()?;
    let identity = parse_identity_markdown(&content);
    
    if identity.name.is_none() && identity.emoji.is_none() && identity.description.is_none() {
        return None;
    }
    
    Some(identity)
}

fn parse_agent_list(config: &Value) -> Vec<AgentConfig> {
    config
        .pointer("/agents/list")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default()
}

fn find_agent_index(agents: &[AgentConfig], agent_id: &str) -> Option<usize> {
    agents.iter().position(|a| a.id == agent_id)
}

fn get_default_model(config: &Value) -> Option<String> {
    config
        .pointer("/agents/defaults/model/primary")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

fn is_builtin_agent(agent_id: &str) -> bool {
    BUILTIN_AGENT_IDS.contains(&agent_id)
}

fn parse_agent_bindings(config: &Value) -> Vec<AgentBinding> {
    config
        .get("bindings")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default()
}

fn get_agent_bindings_from_list(bindings: &[AgentBinding], agent_id: &str) -> Vec<AgentChannelBinding> {
    bindings
        .iter()
        .filter(|b| b.agent_id == agent_id)
        .map(|b| AgentChannelBinding {
            channel: b.r#match.channel.clone(),
            account_id: b.r#match.account_id.clone(),
        })
        .collect()
}

#[command]
pub async fn get_agents_list() -> Result<AgentsListResult, String> {
    info!("[智能体管理] 获取智能体列表...");

    let config = load_openclaw_config()?;
    let agents_config = parse_agent_list(&config);
    let default_model = get_default_model(&config);
    let bindings = parse_agent_bindings(&config);

    let mut agents: Vec<AgentInfo> = Vec::new();
    let mut default_id: Option<String> = None;

    if agents_config.is_empty() {
        agents.push(AgentInfo {
            id: DEFAULT_AGENT_ID.to_string(),
            name: "默认助手".to_string(),
            description: Some("通用AI助手，使用配置的主模型进行对话".to_string()),
            avatar: Some("�".to_string()),
            is_default: true,
            is_builtin: true,
            workspace: None,
            model: default_model.clone(),
            skills: vec![],
            created_at: None,
            updated_at: None,
        });
        default_id = Some(DEFAULT_AGENT_ID.to_string());
    } else {
        for agent in agents_config {
            let is_default = agent.default;
            if is_default {
                default_id = Some(agent.id.clone());
            }

            let model = agent
                .model
                .as_ref()
                .and_then(|m| m.primary.clone())
                .or_else(|| default_model.clone());

            let config_avatar = agent
                .identity
                .as_ref()
                .and_then(|i| i.emoji.clone().or_else(|| i.avatar.clone()));

            let (name, description, avatar) = if let Some(ref workspace) = agent.workspace {
                if let Some(identity_file) = load_identity_from_workspace(workspace) {
                    let n = identity_file.name.or_else(|| agent.name.clone());
                    let desc = identity_file.description;
                    let av = identity_file.emoji.or(identity_file.avatar).or(config_avatar);
                    (n, desc, av)
                } else {
                    (agent.name.clone(), None, config_avatar)
                }
            } else {
                (agent.name.clone(), None, config_avatar)
            };

            let is_builtin = is_builtin_agent(&agent.id);

            agents.push(AgentInfo {
                id: agent.id,
                name: name.unwrap_or_else(|| "未命名智能体".to_string()),
                description,
                avatar,
                is_default,
                is_builtin,
                workspace: agent.workspace,
                model,
                skills: agent.skills,
                created_at: None,
                updated_at: None,
            });
        }

        if default_id.is_none() && !agents.is_empty() {
            default_id = Some(agents[0].id.clone());
        }
    }

    info!("[智能体管理] ✓ 返回 {} 个智能体", agents.len());
    Ok(AgentsListResult {
        agents,
        default_id,
    })
}

#[command]
pub async fn create_agent(params: CreateAgentParams) -> Result<AgentInfo, String> {
    info!("[智能体管理] 创建智能体: {}", params.name);

    let mut config = load_openclaw_config()?;
    let agents = parse_agent_list(&config);

    let mut agent_id = generate_numeric_agent_id();
    while find_agent_index(&agents, &agent_id).is_some() {
        agent_id = generate_numeric_agent_id();
    }

    let workspace_dir = params.workspace.clone().unwrap_or_else(|| {
        get_agent_workspace_dir(&agent_id)
    });

    ensure_agent_workspace(&workspace_dir, &params.name, params.avatar.as_deref())?;

    let default_model = get_default_model(&config);
    let model = params.model.or(default_model);

    let new_agent = AgentConfig {
        id: agent_id.clone(),
        default: false,
        name: Some(params.name.clone()),
        workspace: Some(workspace_dir.clone()),
        agent_dir: None,
        model: model.as_ref().map(|m| AgentModelConfigFull {
            primary: Some(m.clone()),
            fallbacks: vec![],
        }),
        skills: params.skills.clone(),
        identity: Some(AgentIdentityConfig {
            name: Some(params.name.clone()),
            theme: None,
            emoji: params.avatar.clone(),
            avatar: None,
        }),
    };

    if config.get("agents").is_none() {
        config["agents"] = json!({});
    }
    if config["agents"].get("list").is_none() {
        config["agents"]["list"] = json!([]);
    }

    let agents_list = config["agents"]["list"].as_array_mut();
    if let Some(list) = agents_list {
        list.push(serde_json::to_value(&new_agent).map_err(|e| {
            error!("[智能体管理] 序列化智能体配置失败: {}", e);
            format!("序列化智能体配置失败: {}", e)
        })?);
    }

    update_identity_file(
        &workspace_dir,
        &params.name,
        params.avatar.as_deref(),
        params.description.as_deref(),
    )?;

    save_openclaw_config(&config)?;

    info!("[智能体管理] ✓ 智能体 {} 创建成功", agent_id);

    Ok(AgentInfo {
        id: agent_id,
        name: params.name,
        description: params.description,
        avatar: params.avatar,
        is_default: false,
        is_builtin: false,
        workspace: Some(workspace_dir),
        model,
        skills: params.skills,
        created_at: Some(chrono::Utc::now().to_rfc3339()),
        updated_at: Some(chrono::Utc::now().to_rfc3339()),
    })
}

#[command]
pub async fn update_agent(params: UpdateAgentParams) -> Result<AgentInfo, String> {
    info!("[智能体管理] 更新智能体: {}", params.agent_id);

    let mut config = load_openclaw_config()?;

    let mut agents = parse_agent_list(&config);
    let index = find_agent_index(&agents, &params.agent_id)
        .ok_or_else(|| format!("智能体 \"{}\" 不存在", params.agent_id))?;

    {
        let agent = &mut agents[index];

        if let Some(name) = &params.name {
            agent.name = Some(name.clone());
        }

        if let Some(workspace) = &params.workspace {
            agent.workspace = Some(workspace.clone());
            let name = agent.name.as_deref().unwrap_or(&params.agent_id);
            let emoji = agent.identity.as_ref().and_then(|i| i.emoji.as_deref());
            ensure_agent_workspace(workspace, name, emoji)?;
        }

        if let Some(model) = &params.model {
            agent.model = Some(AgentModelConfigFull {
                primary: Some(model.clone()),
                fallbacks: vec![],
            });
        }

        if let Some(skills) = &params.skills {
            agent.skills = skills.clone();
        }

        if agent.identity.is_none() {
            agent.identity = Some(AgentIdentityConfig {
                name: None,
                theme: None,
                emoji: None,
                avatar: None,
            });
        }

        if let Some(identity) = &mut agent.identity {
            if let Some(name) = &params.name {
                identity.name = Some(name.clone());
            }
            if let Some(avatar) = &params.avatar {
                identity.emoji = Some(avatar.clone());
            }
        }

        if let Some(workspace) = &agent.workspace {
            let name = agent.name.as_deref().unwrap_or(&params.agent_id);
            let emoji = agent.identity.as_ref().and_then(|i| i.emoji.as_deref());
            let description = params.description.as_deref();

            let _ = update_identity_file(workspace, name, emoji, description);
        }
    }

    config["agents"]["list"] = serde_json::to_value(&agents).map_err(|e| {
        error!("[智能体管理] 序列化智能体列表失败: {}", e);
        format!("序列化智能体列表失败: {}", e)
    })?;

    save_openclaw_config(&config)?;

    let agent = &agents[index];
    let default_model = get_default_model(&config);
    let model = agent
        .model
        .as_ref()
        .and_then(|m| m.primary.clone())
        .or(default_model);

    let config_avatar = agent
        .identity
        .as_ref()
        .and_then(|i| i.emoji.clone().or_else(|| i.avatar.clone()));

    let (description, avatar) = if let Some(ref workspace) = agent.workspace {
        if let Some(identity_file) = load_identity_from_workspace(workspace) {
            let desc = identity_file.description;
            let av = identity_file.emoji.or(identity_file.avatar).or(config_avatar);
            (desc, av)
        } else {
            (None, config_avatar)
        }
    } else {
        (None, config_avatar)
    };

    info!("[智能体管理] ✓ 智能体 {} 更新成功", params.agent_id);

    Ok(AgentInfo {
        id: agent.id.clone(),
        name: agent.name.clone().unwrap_or_else(|| params.agent_id.clone()),
        description,
        avatar,
        is_default: agent.default,
        is_builtin: is_builtin_agent(&agent.id),
        workspace: agent.workspace.clone(),
        model,
        skills: agent.skills.clone(),
        created_at: None,
        updated_at: Some(chrono::Utc::now().to_rfc3339()),
    })
}

#[command]
pub async fn delete_agent(params: DeleteAgentParams) -> Result<String, String> {
    info!("[智能体管理] 删除智能体: {}", params.agent_id);

    if is_builtin_agent(&params.agent_id) {
        return Err("内置智能体不能删除".to_string());
    }

    let mut config = load_openclaw_config()?;

    let mut agents = parse_agent_list(&config);
    let index = find_agent_index(&agents, &params.agent_id)
        .ok_or_else(|| format!("智能体 \"{}\" 不存在", params.agent_id))?;

    let agent = &agents[index];
    let was_default = agent.default;

    if params.delete_files {
        if let Some(workspace) = &agent.workspace {
            if Path::new(workspace).exists() {
                if let Err(e) = fs::remove_dir_all(workspace) {
                    warn!("[智能体管理] 删除工作区目录失败: {}", e);
                }
            }
        }

        let agents_dir = get_agents_dir();
        let agent_dir = join_path(&agents_dir, &params.agent_id);
        if Path::new(&agent_dir).exists() {
            if let Err(e) = fs::remove_dir_all(&agent_dir) {
                warn!("[智能体管理] 删除智能体目录失败: {}", e);
            }
        }
    }

    agents.remove(index);

    if was_default && !agents.is_empty() {
        agents[0].default = true;
    }

    config["agents"]["list"] = serde_json::to_value(&agents).map_err(|e| {
        error!("[智能体管理] 序列化智能体列表失败: {}", e);
        format!("序列化智能体列表失败: {}", e)
    })?;

    save_openclaw_config(&config)?;

    info!("[智能体管理] ✓ 智能体 {} 已删除", params.agent_id);

    Ok(format!("智能体 {} 已删除", params.agent_id))
}

#[command]
pub async fn set_default_agent(params: SetDefaultAgentParams) -> Result<String, String> {
    info!("[智能体管理] 设置默认智能体: {}", params.agent_id);

    let mut config = load_openclaw_config()?;

    let mut agents = parse_agent_list(&config);
    let index = find_agent_index(&agents, &params.agent_id)
        .ok_or_else(|| format!("智能体 \"{}\" 不存在", params.agent_id))?;

    for agent in &mut agents {
        agent.default = false;
    }

    agents[index].default = true;

    config["agents"]["list"] = serde_json::to_value(&agents).map_err(|e| {
        error!("[智能体管理] 序列化智能体列表失败: {}", e);
        format!("序列化智能体列表失败: {}", e)
    })?;

    save_openclaw_config(&config)?;

    info!("[智能体管理] ✓ 默认智能体已设置为 {}", params.agent_id);

    Ok(format!("默认智能体已设置为 {}", params.agent_id))
}

#[command]
pub async fn get_agent_by_id(agent_id: String) -> Result<AgentInfo, String> {
    info!("[智能体管理] 获取智能体详情: {}", agent_id);

    let config = load_openclaw_config()?;
    let agents = parse_agent_list(&config);
    let default_model = get_default_model(&config);

    let agent = agents
        .iter()
        .find(|a| a.id == agent_id)
        .ok_or_else(|| format!("智能体 \"{}\" 不存在", agent_id))?;

    let model = agent
        .model
        .as_ref()
        .and_then(|m| m.primary.clone())
        .or(default_model);

    let config_avatar = agent
        .identity
        .as_ref()
        .and_then(|i| i.emoji.clone().or_else(|| i.avatar.clone()));

    let (description, avatar) = if let Some(ref workspace) = agent.workspace {
        if let Some(identity_file) = load_identity_from_workspace(workspace) {
            let desc = identity_file.description;
            let av = identity_file.emoji.or(identity_file.avatar).or(config_avatar);
            (desc, av)
        } else {
            (None, config_avatar)
        }
    } else {
        (None, config_avatar)
    };

    Ok(AgentInfo {
        id: agent.id.clone(),
        name: agent.name.clone().unwrap_or_else(|| agent_id.clone()),
        description,
        avatar,
        is_default: agent.default,
        is_builtin: is_builtin_agent(&agent.id),
        workspace: agent.workspace.clone(),
        model,
        skills: agent.skills.clone(),
        created_at: None,
        updated_at: None,
    })
}

#[command]
pub async fn get_agent_bindings(agent_id: String) -> Result<AgentBindingsResult, String> {
    info!("[智能体管理] 获取智能体渠道关联: {}", agent_id);

    let config = load_openclaw_config()?;
    let bindings = parse_agent_bindings(&config);

    let agent_bindings = get_agent_bindings_from_list(&bindings, &agent_id);

    info!("[智能体管理] ✓ 智能体 {} 关联了 {} 个渠道", agent_id, agent_bindings.len());

    Ok(AgentBindingsResult {
        agent_id,
        bindings: agent_bindings,
    })
}

#[command]
pub async fn set_agent_bindings(params: SetAgentBindingsParams) -> Result<String, String> {
    info!("[智能体管理] 设置智能体渠道关联: {}", params.agent_id);

    let mut config = load_openclaw_config()?;

    let agents = parse_agent_list(&config);
    if find_agent_index(&agents, &params.agent_id).is_none() {
        return Err(format!("智能体 \"{}\" 不存在", params.agent_id));
    }

    let existing_bindings = parse_agent_bindings(&config);
    let mut new_bindings: Vec<AgentBinding> = existing_bindings
        .into_iter()
        .filter(|b| b.agent_id != params.agent_id)
        .collect();

    for binding in params.bindings {
        new_bindings.push(AgentBinding {
            r#type: Some("route".to_string()),
            agent_id: params.agent_id.clone(),
            comment: None,
            r#match: AgentBindingMatch {
                channel: binding.channel,
                account_id: binding.account_id,
                peer: None,
                guild_id: None,
                team_id: None,
                roles: None,
            },
        });
    }

    if new_bindings.is_empty() {
        config["bindings"] = json!([]);
    } else {
        config["bindings"] = serde_json::to_value(&new_bindings).map_err(|e| {
            error!("[智能体管理] 序列化绑定配置失败: {}", e);
            format!("序列化绑定配置失败: {}", e)
        })?;
    }

    save_openclaw_config(&config)?;

    info!("[智能体管理] ✓ 智能体 {} 渠道关联已更新", params.agent_id);

    Ok(format!("智能体 {} 渠道关联已更新", params.agent_id))
}

#[command]
pub async fn get_available_channels() -> Result<Vec<String>, String> {
    info!("[智能体管理] 获取可用渠道列表...");

    let config = load_openclaw_config()?;
    let channels: Vec<String> = config
        .get("channels")
        .and_then(|c| c.as_object())
        .map(|obj| obj.keys().cloned().collect())
        .unwrap_or_default();

    info!("[智能体管理] ✓ 返回 {} 个可用渠道", channels.len());

    Ok(channels)
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorkspaceFile {
    pub filename: String,
    pub content: String,
    pub exists: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorkspaceFilesResult {
    pub workspace_dir: String,
    pub files: Vec<WorkspaceFile>,
}

#[command]
pub async fn get_agent_workspace_files(agent_id: String) -> Result<WorkspaceFilesResult, String> {
    info!("[智能体管理] 获取智能体工作区文件: {}", agent_id);

    let config = load_openclaw_config()?;
    let agents = parse_agent_list(&config);

    let agent = agents
        .iter()
        .find(|a| a.id == agent_id)
        .ok_or_else(|| format!("智能体 \"{}\" 不存在", agent_id))?;

    let workspace = agent.workspace.clone()
        .ok_or_else(|| format!("智能体 \"{}\" 没有工作区", agent_id))?;

    let sep = path_separator();
    let filenames = [IDENTITY_FILENAME, SOUL_FILENAME, AGENTS_FILENAME, USER_FILENAME, MEMORY_FILENAME];

    let mut files = Vec::new();
    for filename in filenames {
        let file_path = format!("{}{}{}", workspace, sep, filename);
        let (content, exists) = if Path::new(&file_path).exists() {
            match file::read_file(&file_path) {
                Ok(c) => (c, true),
                Err(e) => {
                    warn!("[智能体管理] 读取文件 {} 失败: {}", filename, e);
                    (String::new(), false)
                }
            }
        } else {
            (String::new(), false)
        };
        files.push(WorkspaceFile { filename: filename.to_string(), content, exists });
    }

    info!("[智能体管理] ✓ 返回 {} 个工作区文件", files.len());

    Ok(WorkspaceFilesResult {
        workspace_dir: workspace,
        files,
    })
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SaveWorkspaceFileParams {
    #[serde(rename = "agentId")]
    pub agent_id: String,
    pub filename: String,
    pub content: String,
}

#[command]
pub async fn save_agent_workspace_file(params: SaveWorkspaceFileParams) -> Result<String, String> {
    info!("[智能体管理] 保存智能体工作区文件: {} - {}", params.agent_id, params.filename);

    let valid_filenames = [IDENTITY_FILENAME, SOUL_FILENAME, AGENTS_FILENAME, USER_FILENAME, MEMORY_FILENAME];
    if !valid_filenames.contains(&params.filename.as_str()) {
        return Err(format!("无效的文件名: {}", params.filename));
    }

    let config = load_openclaw_config()?;
    let agents = parse_agent_list(&config);

    let agent = agents
        .iter()
        .find(|a| a.id == params.agent_id)
        .ok_or_else(|| format!("智能体 \"{}\" 不存在", params.agent_id))?;

    let workspace = agent.workspace.clone()
        .ok_or_else(|| format!("智能体 \"{}\" 没有工作区", params.agent_id))?;

    let sep = path_separator();
    let file_path = format!("{}{}{}", workspace, sep, params.filename);

    file::write_file(&file_path, &params.content)
        .map_err(|e| format!("写入文件失败: {}", e))?;

    info!("[智能体管理] ✓ 文件 {} 已保存", params.filename);

    Ok(format!("文件 {} 已保存", params.filename))
}
