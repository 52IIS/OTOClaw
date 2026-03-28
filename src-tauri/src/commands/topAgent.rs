use crate::commands::common::{
    get_config_dir, get_sub_dir, load_openclaw_config, path_separator,
    save_openclaw_config,
};
use crate::models::AgentConfig;
use crate::utils::file;
use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tauri::command;

const BUILTIN_AGENT_IDS: &[&str] = &["default"];
const IDENTITY_FILENAME: &str = "IDENTITY.md";
const SOUL_FILENAME: &str = "SOUL.md";
const AGENTS_FILENAME: &str = "AGENTS.md";
const USER_FILENAME: &str = "USER.md";
const MEMORY_FILENAME: &str = "MEMORY.md";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuiltinAgentInfo {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub avatar: Option<String>,
    pub is_default: bool,
    pub workspace: Option<String>,
    pub model: Option<String>,
    pub skills: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuiltinAgentsResult {
    pub agents: Vec<BuiltinAgentInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceFile {
    pub filename: String,
    pub content: String,
    pub exists: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceFilesResult {
    pub workspace_dir: String,
    pub files: Vec<WorkspaceFile>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SaveWorkspaceFileParams {
    #[serde(rename = "agentId")]
    pub agent_id: String,
    pub filename: String,
    pub content: String,
}

fn is_builtin_agent(agent_id: &str) -> bool {
    BUILTIN_AGENT_IDS.contains(&agent_id)
}

fn get_builtin_agent_workspace_dir() -> String {
    get_sub_dir(&get_config_dir(), "workspace")
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

fn ensure_workspace_files(workspace_dir: &str) -> Result<(), String> {
    let path = Path::new(workspace_dir);
    if !path.exists() {
        std::fs::create_dir_all(path)
            .map_err(|e| format!("创建工作区目录失败: {}", e))?;
    }

    let sep = path_separator();

    let identity_path = format!("{}{}{}", workspace_dir, sep, IDENTITY_FILENAME);
    if !Path::new(&identity_path).exists() {
        let content = get_default_identity_template("默认助手", Some("🤖"));
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

fn get_default_model(config: &serde_json::Value) -> Option<String> {
    config
        .pointer("/agents/defaults/model/primary")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

fn parse_agent_list(config: &serde_json::Value) -> Vec<AgentConfig> {
    config
        .pointer("/agents/list")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default()
}

#[command]
pub async fn get_builtin_agents() -> Result<BuiltinAgentsResult, String> {
    info!("[内置智能体管理] 获取内置智能体列表...");

    let config = load_openclaw_config()?;
    let agents_config = parse_agent_list(&config);
    let default_model = get_default_model(&config);
    let workspace_dir = get_builtin_agent_workspace_dir();

    let _ = ensure_workspace_files(&workspace_dir);

    let mut builtin_agents: Vec<BuiltinAgentInfo> = Vec::new();

    for agent in agents_config {
        if !is_builtin_agent(&agent.id) {
            continue;
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

        let workspace = agent.workspace.clone().unwrap_or_else(|| workspace_dir.clone());

        let (description, avatar) = if let Some(identity_file) = load_identity_from_workspace(&workspace) {
            let desc = identity_file.description;
            let av = identity_file.emoji.or(identity_file.avatar).or(config_avatar);
            (desc, av)
        } else {
            (None, config_avatar)
        };

        builtin_agents.push(BuiltinAgentInfo {
            id: agent.id,
            name: agent.name.unwrap_or_else(|| "未命名智能体".to_string()),
            description,
            avatar,
            is_default: agent.default,
            workspace: Some(workspace),
            model,
            skills: agent.skills,
        });
    }

    if builtin_agents.is_empty() {
        let identity = load_identity_from_workspace(&workspace_dir);
        builtin_agents.push(BuiltinAgentInfo {
            id: "default".to_string(),
            name: identity.as_ref().and_then(|i| i.name.clone()).unwrap_or_else(|| "默认助手".to_string()),
            description: identity.as_ref().and_then(|i| i.description.clone()).or_else(|| Some("通用AI助手，使用配置的主模型进行对话".to_string())),
            avatar: identity.as_ref().and_then(|i| i.emoji.clone().or_else(|| i.avatar.clone())).or_else(|| Some("🤖".to_string())),
            is_default: true,
            workspace: Some(workspace_dir),
            model: default_model,
            skills: vec![],
        });
    }

    info!("[内置智能体管理] ✓ 返回 {} 个内置智能体", builtin_agents.len());
    Ok(BuiltinAgentsResult {
        agents: builtin_agents,
    })
}

#[command]
pub async fn get_builtin_agent_workspace_files(agent_id: String) -> Result<WorkspaceFilesResult, String> {
    info!("[内置智能体管理] 获取内置智能体工作区文件: {}", agent_id);

    if !is_builtin_agent(&agent_id) {
        return Err(format!("智能体 \"{}\" 不是内置智能体", agent_id));
    }

    let config = load_openclaw_config()?;
    let agents = parse_agent_list(&config);
    let default_workspace = get_builtin_agent_workspace_dir();

    let agent = agents
        .iter()
        .find(|a| a.id == agent_id);

    let workspace = if let Some(agent) = agent {
        agent.workspace.clone().unwrap_or(default_workspace)
    } else {
        default_workspace
    };

    let _ = ensure_workspace_files(&workspace);

    let sep = path_separator();
    let filenames = [
        IDENTITY_FILENAME,
        SOUL_FILENAME,
        AGENTS_FILENAME,
        USER_FILENAME,
        MEMORY_FILENAME,
    ];

    let mut files = Vec::new();
    for filename in filenames {
        let file_path = format!("{}{}{}", workspace, sep, filename);
        let (content, exists) = if Path::new(&file_path).exists() {
            match file::read_file(&file_path) {
                Ok(c) => (c, true),
                Err(e) => {
                    warn!("[内置智能体管理] 读取文件 {} 失败: {}", filename, e);
                    (String::new(), false)
                }
            }
        } else {
            (String::new(), false)
        };
        files.push(WorkspaceFile {
            filename: filename.to_string(),
            content,
            exists,
        });
    }

    info!("[内置智能体管理] ✓ 返回 {} 个工作区文件", files.len());

    Ok(WorkspaceFilesResult {
        workspace_dir: workspace,
        files,
    })
}

#[command]
pub async fn save_builtin_agent_workspace_file(params: SaveWorkspaceFileParams) -> Result<String, String> {
    info!(
        "[内置智能体管理] 保存内置智能体工作区文件: {} - {}",
        params.agent_id, params.filename
    );

    if !is_builtin_agent(&params.agent_id) {
        return Err(format!("智能体 \"{}\" 不是内置智能体", params.agent_id));
    }

    let valid_filenames = [
        IDENTITY_FILENAME,
        SOUL_FILENAME,
        AGENTS_FILENAME,
        USER_FILENAME,
        MEMORY_FILENAME,
    ];
    if !valid_filenames.contains(&params.filename.as_str()) {
        return Err(format!("无效的文件名: {}", params.filename));
    }

    let config = load_openclaw_config()?;
    let agents = parse_agent_list(&config);
    let default_workspace = get_builtin_agent_workspace_dir();

    let agent = agents
        .iter()
        .find(|a| a.id == params.agent_id);

    let workspace = if let Some(agent) = agent {
        agent.workspace.clone().unwrap_or(default_workspace)
    } else {
        default_workspace
    };

    let _ = ensure_workspace_files(&workspace);

    let sep = path_separator();
    let file_path = format!("{}{}{}", workspace, sep, params.filename);

    file::write_file(&file_path, &params.content).map_err(|e| format!("写入文件失败: {}", e))?;

    info!("[内置智能体管理] ✓ 文件 {} 已保存", params.filename);

    Ok(format!("文件 {} 已保存", params.filename))
}

#[command]
pub async fn get_builtin_agent_skills(agent_id: String) -> Result<Vec<String>, String> {
    info!("[内置智能体管理] 获取内置智能体技能: {}", agent_id);

    if !is_builtin_agent(&agent_id) {
        return Err(format!("智能体 \"{}\" 不是内置智能体", agent_id));
    }

    let config = load_openclaw_config()?;
    let agents = parse_agent_list(&config);

    let skills = if let Some(agent) = agents.iter().find(|a| a.id == agent_id) {
        agent.skills.clone()
    } else {
        vec![]
    };

    info!("[内置智能体管理] ✓ 智能体 {} 有 {} 个技能", agent_id, skills.len());
    Ok(skills)
}

#[command]
pub async fn assign_skill_to_builtin_agent(skill_id: String, agent_id: String) -> Result<String, String> {
    info!(
        "[内置智能体管理] 分配技能 {} 到内置智能体 {}",
        skill_id, agent_id
    );

    if !is_builtin_agent(&agent_id) {
        return Err(format!("智能体 \"{}\" 不是内置智能体", agent_id));
    }

    let mut config = load_openclaw_config()?;

    let agents = parse_agent_list(&config);
    let index = agents
        .iter()
        .position(|a| a.id == agent_id);

    if let Some(idx) = index {
        let agents_list = config["agents"]["list"].as_array_mut();
        if let Some(list) = agents_list {
            let agent = &mut list[idx];
            let skills = agent["skills"].as_array_mut();

            if let Some(skills_arr) = skills {
                if !skills_arr.contains(&serde_json::Value::String(skill_id.clone())) {
                    skills_arr.push(serde_json::Value::String(skill_id.clone()));
                }
            } else {
                agent["skills"] = serde_json::json!([skill_id]);
            }
        }
    } else {
        let default_workspace = get_builtin_agent_workspace_dir();
        let new_agent = serde_json::json!({
            "id": agent_id,
            "default": true,
            "name": "默认助手",
            "workspace": default_workspace,
            "skills": [skill_id]
        });

        if config.get("agents").is_none() {
            config["agents"] = serde_json::json!({});
        }
        if config["agents"].get("list").is_none() {
            config["agents"]["list"] = serde_json::json!([]);
        }

        let agents_list = config["agents"]["list"].as_array_mut();
        if let Some(list) = agents_list {
            list.push(new_agent);
        }
    }

    save_openclaw_config(&config)?;

    info!("[内置智能体管理] ✓ 技能 {} 已分配给智能体 {}", skill_id, agent_id);
    Ok(format!("技能 {} 已分配给智能体 {}", skill_id, agent_id))
}

#[command]
pub async fn remove_skill_from_builtin_agent(skill_id: String, agent_id: String) -> Result<String, String> {
    info!(
        "[内置智能体管理] 从内置智能体 {} 移除技能 {}",
        agent_id, skill_id
    );

    if !is_builtin_agent(&agent_id) {
        return Err(format!("智能体 \"{}\" 不是内置智能体", agent_id));
    }

    let mut config = load_openclaw_config()?;

    let agents = parse_agent_list(&config);
    let index = agents
        .iter()
        .position(|a| a.id == agent_id);

    if let Some(idx) = index {
        let agents_list = config["agents"]["list"].as_array_mut();
        if let Some(list) = agents_list {
            let agent = &mut list[idx];
            let skills = agent["skills"].as_array_mut();

            if let Some(skills_arr) = skills {
                skills_arr.retain(|s| s != &serde_json::Value::String(skill_id.clone()));
            }
        }
    }

    save_openclaw_config(&config)?;

    info!(
        "[内置智能体管理] ✓ 技能 {} 已从智能体 {} 移除",
        skill_id, agent_id
    );
    Ok(format!("技能 {} 已从智能体 {} 移除", skill_id, agent_id))
}