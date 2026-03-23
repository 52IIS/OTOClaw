use crate::models::{
    CreateSkillParams, ExportSkillParams, ExportSkillResult, InstallSkillParams,
    InstallSkillResult, SkillConfigEntry, SkillDetail, SkillInfo, SkillInstallOption,
    SkillInstallResult, SkillsListResult, UpdateSkillConfigParams,
};
use crate::utils::{file, platform};
use log::info;
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use tauri::command;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

const SKILL_FILENAME: &str = "SKILL.md";
const EXCLUDE_DIRS: [&str; 6] = [".git", ".svn", ".hg", "__pycache__", "node_modules", ".DS_Store"];

fn load_openclaw_config() -> Result<Value, String> {
    let config_path = platform::get_config_file_path();
    if !file::file_exists(&config_path) {
        return Ok(json!({}));
    }
    let content = file::read_file(&config_path).map_err(|e| format!("Failed to read config: {}", e))?;
    serde_json::from_str(&content).map_err(|e| format!("Failed to parse config: {}", e))
}

fn save_openclaw_config(config: &Value) -> Result<(), String> {
    let config_path = platform::get_config_file_path();
    let content = serde_json::to_string_pretty(config).map_err(|e| format!("Failed to serialize: {}", e))?;
    file::write_file(&config_path, &content).map_err(|e| format!("Failed to write config: {}", e))
}

fn get_bundled_skills_dir() -> String {
    let config_dir = platform::get_config_dir();
    if platform::is_windows() {
        format!("{}\\skills", config_dir)
    } else {
        format!("{}/skills", config_dir)
    }
}

fn get_managed_skills_dir() -> String {
    let config_dir = platform::get_config_dir();
    if platform::is_windows() {
        format!("{}\\managed-skills", config_dir)
    } else {
        format!("{}/managed-skills", config_dir)
    }
}

fn get_agent_skills_dir(agent_id: &str) -> String {
    let config_dir = platform::get_config_dir();
    if platform::is_windows() {
        format!("{}\\workspace-{}\\skills", config_dir, agent_id)
    } else {
        format!("{}/workspace-{}/skills", config_dir, agent_id)
    }
}

fn get_skill_dir(skill_id: &str, source: &str, agent_id: Option<&str>) -> String {
    if let Some(aid) = agent_id {
        let skills_dir = get_agent_skills_dir(aid);
        return join_path(&skills_dir, skill_id);
    }
    match source {
        "bundled" => join_path(&get_bundled_skills_dir(), skill_id),
        "managed" => join_path(&get_managed_skills_dir(), skill_id),
        _ => join_path(&get_managed_skills_dir(), skill_id),
    }
}

fn join_path(base: &str, name: &str) -> String {
    if platform::is_windows() {
        format!("{}\\{}", base, name)
    } else {
        format!("{}/{}", base, name)
    }
}

fn parse_skill_frontmatter(content: &str) -> Option<Value> {
    let content = content.trim();
    if !content.starts_with("---") {
        return None;
    }
    let parts: Vec<&str> = content.splitn(3, "---").collect();
    if parts.len() < 3 {
        return None;
    }
    let frontmatter = parts[1].trim();
    serde_yaml::from_str::<Value>(frontmatter).ok()
}

fn extract_openclaw_metadata(frontmatter: &Value) -> Option<&Value> {
    frontmatter.get("metadata").and_then(|m| m.get("openclaw"))
}

fn check_bin_exists(bin: &str) -> bool {
    #[cfg(windows)]
    {
        let output = std::process::Command::new("where")
            .arg(bin)
            .creation_flags(CREATE_NO_WINDOW)
            .output();
        matches!(output, Ok(o) if o.status.success())
    }
    #[cfg(not(windows))]
    {
        let output = std::process::Command::new("which").arg(bin).output();
        matches!(output, Ok(o) if o.status.success())
    }
}

fn check_env_exists(env_key: &str) -> bool {
    std::env::var(env_key).is_ok()
}

fn get_skill_config_from_openclaw(skill_id: &str, config: &Value) -> Option<SkillConfigEntry> {
    config
        .pointer(&format!("/skills/entries/{}", skill_id))
        .and_then(|v| serde_json::from_value(v.clone()).ok())
}

fn parse_skill_file(file_path: &str, source: &str, bundled: bool, config: Option<&Value>) -> Option<SkillInfo> {
    let content = file::read_file(file_path).ok()?;
    let frontmatter = parse_skill_frontmatter(&content)?;

    let name = frontmatter.get("name").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
    let description = frontmatter.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let homepage = frontmatter.get("homepage").and_then(|v| v.as_str()).map(|s| s.to_string());
    let metadata = extract_openclaw_metadata(&frontmatter);
    let emoji = metadata.and_then(|m| m.get("emoji")).and_then(|v| v.as_str()).map(|s| s.to_string());
    let primary_env = metadata.and_then(|m| m.get("primaryEnv")).and_then(|v| v.as_str()).map(|s| s.to_string());

    let required_bins: Vec<String> = metadata
        .and_then(|m| m.get("requires"))
        .and_then(|r| r.get("bins"))
        .and_then(|b| b.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_default();

    let required_env: Vec<String> = metadata
        .and_then(|m| m.get("requires"))
        .and_then(|r| r.get("env"))
        .and_then(|e| e.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_default();

    let id = name.to_lowercase().replace(' ', "-").replace('_', "-");
    let skill_config = config.and_then(|c| get_skill_config_from_openclaw(&id, c));
    let is_disabled = skill_config.as_ref().map(|c| !c.enabled).unwrap_or(false);

    let bins_available: bool = required_bins.iter().all(|b| check_bin_exists(b));
    let env_available: bool = if let Some(ref cfg) = skill_config {
        required_env.iter().all(|e| {
            check_env_exists(e) || cfg.env.contains_key(e) || (cfg.api_key.is_some() && primary_env.as_deref() == Some(e))
        })
    } else {
        required_env.iter().all(|e| check_env_exists(e))
    };
    let eligible = bins_available && env_available && !is_disabled;

    let install_options: Vec<SkillInstallOption> = metadata
        .and_then(|m| m.get("install"))
        .and_then(|i| i.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|opt| {
                    Some(SkillInstallOption {
                        id: opt.get("id")?.as_str()?.to_string(),
                        kind: opt.get("kind")?.as_str()?.to_string(),
                        label: opt.get("label")?.as_str()?.to_string(),
                        bins: opt.get("bins").and_then(|b| b.as_array())
                            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                            .unwrap_or_default(),
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    let tags = infer_skill_tags(&name, &description);

    Some(SkillInfo {
        id,
        name,
        description,
        emoji,
        homepage,
        source: source.to_string(),
        bundled,
        eligible,
        disabled: is_disabled,
        tags,
        version: None,
        author: None,
        required_env,
        required_bins,
        install_options,
    })
}

fn infer_skill_tags(name: &str, description: &str) -> Vec<String> {
    let mut tags = HashSet::new();
    let name_lower = name.to_lowercase();
    let desc_lower = description.to_lowercase();

    let tag_keywords = [
        ("github", vec!["github", "gh", "pr", "issue", "repo"]),
        ("slack", vec!["slack", "channel", "message"]),
        ("git", vec!["git", "commit", "branch"]),
        ("api", vec!["api", "rest", "graphql"]),
        ("cli", vec!["cli", "command", "terminal"]),
        ("web", vec!["web", "browser", "http"]),
        ("ai", vec!["ai", "llm", "gpt", "claude", "gemini"]),
        ("image", vec!["image", "picture", "photo"]),
        ("weather", vec!["weather", "forecast", "temperature"]),
        ("productivity", vec!["task", "todo", "project", "trello"]),
        ("communication", vec!["message", "chat", "notify"]),
        ("automation", vec!["automate", "schedule", "workflow"]),
        ("development", vec!["code", "develop", "build", "test"]),
        ("data", vec!["data", "json", "csv", "database"]),
    ];

    for (tag, keywords) in tag_keywords {
        for keyword in keywords {
            if name_lower.contains(keyword) || desc_lower.contains(keyword) {
                tags.insert(tag.to_string());
                break;
            }
        }
    }

    tags.into_iter().collect()
}

fn scan_skills_directory(dir: &str, source: &str, bundled: bool, config: Option<&Value>) -> Vec<SkillInfo> {
    let mut skills = Vec::new();
    if !Path::new(dir).exists() {
        return skills;
    }
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let skill_file = join_path(&path.to_string_lossy(), SKILL_FILENAME);
                if Path::new(&skill_file).exists() {
                    if let Some(skill) = parse_skill_file(&skill_file, source, bundled, config) {
                        skills.push(skill);
                    }
                }
            }
        }
    }
    skills
}

fn scan_agent_skills_directory(agent_id: &str, config: Option<&Value>) -> Vec<SkillInfo> {
    let skills_dir = get_agent_skills_dir(agent_id);
    scan_skills_directory(&skills_dir, &format!("agent-{}", agent_id), false, config)
}

#[command]
pub async fn get_skills_list() -> Result<SkillsListResult, String> {
    info!("[Skills] Getting skills list...");

    let config = load_openclaw_config().ok();
    let mut all_skills: Vec<SkillInfo> = Vec::new();
    let mut seen_ids: HashSet<String> = HashSet::new();

    let bundled_dir = get_bundled_skills_dir();
    for skill in scan_skills_directory(&bundled_dir, "bundled", true, config.as_ref()) {
        if !seen_ids.contains(&skill.id) {
            seen_ids.insert(skill.id.clone());
            all_skills.push(skill);
        }
    }

    let managed_dir = get_managed_skills_dir();
    for skill in scan_skills_directory(&managed_dir, "managed", false, config.as_ref()) {
        if !seen_ids.contains(&skill.id) {
            seen_ids.insert(skill.id.clone());
            all_skills.push(skill);
        }
    }

    if let Some(ref cfg) = config {
        if let Some(extra_dirs) = cfg.pointer("/skills/load/extraDirs").and_then(|v| v.as_array()) {
            for dir_val in extra_dirs {
                if let Some(dir) = dir_val.as_str() {
                    let expanded = shellexpand::tilde(dir).to_string();
                    for skill in scan_skills_directory(&expanded, "extra", false, Some(cfg)) {
                        if !seen_ids.contains(&skill.id) {
                            seen_ids.insert(skill.id.clone());
                            all_skills.push(skill);
                        }
                    }
                }
            }
        }
    }

    all_skills.sort_by(|a, b| a.name.cmp(&b.name));
    let eligible_count = all_skills.iter().filter(|s| s.eligible).count();
    let total = all_skills.len();

    info!("[Skills] Returning {} skills, {} eligible", total, eligible_count);

    Ok(SkillsListResult {
        skills: all_skills,
        total,
        eligible_count,
    })
}

#[command]
pub async fn get_builtin_skills() -> Result<Vec<SkillInfo>, String> {
    info!("[Skills] Getting builtin skills...");
    let config = load_openclaw_config().ok();
    let bundled_dir = get_bundled_skills_dir();
    let skills = scan_skills_directory(&bundled_dir, "bundled", true, config.as_ref());
    info!("[Skills] Returning {} builtin skills", skills.len());
    Ok(skills)
}

#[command]
pub async fn check_skill_requirements(skill_id: String) -> Result<Value, String> {
    info!("[Skills] Checking requirements for: {}", skill_id);

    let skills = get_skills_list().await?;
    let skill = skills.skills.iter().find(|s| s.id == skill_id)
        .ok_or_else(|| format!("Skill '{}' not found", skill_id))?;

    let mut missing_bins: Vec<String> = Vec::new();
    let mut missing_env: Vec<String> = Vec::new();

    for bin in &skill.required_bins {
        if !check_bin_exists(bin) {
            missing_bins.push(bin.clone());
        }
    }

    for env in &skill.required_env {
        if !check_env_exists(env) {
            missing_env.push(env.clone());
        }
    }

    Ok(json!({
        "skillId": skill_id,
        "eligible": missing_bins.is_empty() && missing_env.is_empty(),
        "missingBins": missing_bins,
        "missingEnv": missing_env,
        "installOptions": skill.install_options,
    }))
}

#[command]
pub async fn get_skill_detail(skill_id: String) -> Result<SkillDetail, String> {
    info!("[Skills] Getting detail for: {}", skill_id);

    let config = load_openclaw_config()?;
    let skills = get_skills_list().await?;
    let skill = skills.skills.iter().find(|s| s.id == skill_id)
        .ok_or_else(|| format!("Skill '{}' not found", skill_id))?;

    let skill_dir = get_skill_dir(&skill.id, &skill.source, None);
    let skill_file = join_path(&skill_dir, SKILL_FILENAME);

    let skill_md_content = if Path::new(&skill_file).exists() {
        file::read_file(&skill_file).ok()
    } else {
        None
    };

    let skill_config = get_skill_config_from_openclaw(&skill_id, &config);

    let metadata = if let Some(ref content) = skill_md_content {
        parse_skill_frontmatter(content).and_then(|f| extract_openclaw_metadata(&f).cloned())
    } else {
        None
    };

    let primary_env = metadata.as_ref()
        .and_then(|m| m.get("primaryEnv"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let requires_api_key = !skill.required_env.is_empty() || primary_env.is_some();

    info!("[Skills] Returning detail for {}", skill_id);

    Ok(SkillDetail {
        info: skill.clone(),
        config: skill_config,
        path: skill_dir,
        skill_md_content,
        requires_api_key,
        primary_env,
    })
}

#[command]
pub async fn create_skill(params: CreateSkillParams) -> Result<SkillInfo, String> {
    info!("[Skills] Creating skill: {}", params.name);

    let skill_id = params.name.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .to_string();

    if skill_id.is_empty() {
        return Err("Invalid skill name".to_string());
    }

    let target_dir = if let Some(ref agent_id) = params.agent_id {
        let skills_dir = get_agent_skills_dir(agent_id);
        join_path(&skills_dir, &skill_id)
    } else {
        let managed_dir = get_managed_skills_dir();
        join_path(&managed_dir, &skill_id)
    };

    if Path::new(&target_dir).exists() {
        return Err(format!("Skill '{}' already exists", skill_id));
    }

    fs::create_dir_all(&target_dir).map_err(|e| format!("Failed to create directory: {}", e))?;

    let skill_md_content = params.skill_md_content.unwrap_or_else(|| {
        generate_skill_md_template(&params.name, &params.description, &params.emoji, &params.homepage, &params.required_env, &params.required_bins)
    });

    let skill_file = join_path(&target_dir, SKILL_FILENAME);
    file::write_file(&skill_file, &skill_md_content).map_err(|e| format!("Failed to write SKILL.md: {}", e))?;

    info!("[Skills] Skill {} created successfully", skill_id);

    let tags = infer_skill_tags(&params.name, &params.description);

    Ok(SkillInfo {
        id: skill_id,
        name: params.name,
        description: params.description,
        emoji: params.emoji,
        homepage: params.homepage,
        source: if params.agent_id.is_some() { "agent" } else { "managed" }.to_string(),
        bundled: false,
        eligible: params.required_env.is_empty() && params.required_bins.is_empty(),
        disabled: false,
        tags,
        version: None,
        author: None,
        required_env: params.required_env,
        required_bins: params.required_bins,
        install_options: vec![],
    })
}

fn generate_skill_md_template(name: &str, description: &str, emoji: &Option<String>, homepage: &Option<String>, required_env: &[String], required_bins: &[String]) -> String {
    let mut content = String::new();
    content.push_str("---\n");
    content.push_str(&format!("name: {}\n", name));
    content.push_str(&format!("description: {}\n", description));
    if let Some(hp) = homepage {
        content.push_str(&format!("homepage: {}\n", hp));
    }

    let mut metadata = serde_json::Map::new();
    if let Some(e) = emoji {
        metadata.insert("emoji".to_string(), json!(e));
    }
    if !required_env.is_empty() || !required_bins.is_empty() {
        let mut requires = serde_json::Map::new();
        if !required_env.is_empty() {
            requires.insert("env".to_string(), json!(required_env));
        }
        if !required_bins.is_empty() {
            requires.insert("bins".to_string(), json!(required_bins));
        }
        metadata.insert("requires".to_string(), json!(requires));
    }
    if !metadata.is_empty() {
        content.push_str("metadata:\n  openclaw:\n");
        for (key, value) in &metadata {
            content.push_str(&format!("    {}: {}\n", key, serde_json::to_string(value).unwrap_or_default()));
        }
    }

    content.push_str("---\n\n");
    content.push_str(&format!("# {}\n\n", name));
    content.push_str(&format!("{}\n\n## Usage\n\nDescribe how to use this skill here.\n", description));
    content
}

#[command]
pub async fn update_skill_config(params: UpdateSkillConfigParams) -> Result<String, String> {
    info!("[Skills] Updating config for: {}", params.skill_id);

    let mut config = load_openclaw_config()?;
    if config.get("skills").is_none() {
        config["skills"] = json!({});
    }
    if config["skills"].get("entries").is_none() {
        config["skills"]["entries"] = json!({});
    }

    let existing: SkillConfigEntry = config
        .pointer(&format!("/skills/entries/{}", params.skill_id))
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or(SkillConfigEntry {
            enabled: true,
            api_key: None,
            env: HashMap::new(),
            config: json!(null),
        });

    let updated = SkillConfigEntry {
        enabled: params.enabled.unwrap_or(existing.enabled),
        api_key: params.api_key.or(existing.api_key),
        env: params.env.unwrap_or(existing.env),
        config: params.config.unwrap_or(existing.config),
    };

    config["skills"]["entries"][&params.skill_id] = serde_json::to_value(&updated)
        .map_err(|e| format!("Failed to serialize: {}", e))?;

    save_openclaw_config(&config)?;
    info!("[Skills] Config updated for {}", params.skill_id);
    Ok(format!("Config updated for {}", params.skill_id))
}

#[command]
pub async fn delete_skill(skill_id: String) -> Result<String, String> {
    info!("[Skills] Deleting skill: {}", skill_id);

    let skills = get_skills_list().await?;
    let skill = skills.skills.iter().find(|s| s.id == skill_id)
        .ok_or_else(|| format!("Skill '{}' not found", skill_id))?;

    if skill.bundled {
        return Err("Cannot delete bundled skills".to_string());
    }

    let skill_dir = get_skill_dir(&skill.id, &skill.source, None);
    if Path::new(&skill_dir).exists() {
        fs::remove_dir_all(&skill_dir).map_err(|e| format!("Failed to delete directory: {}", e))?;
    }

    let mut config = load_openclaw_config().ok();
    if let Some(ref mut cfg) = config {
        if let Some(entries) = cfg.get_mut("skills").and_then(|s| s.get_mut("entries")) {
            if let Some(obj) = entries.as_object_mut() {
                obj.remove(&skill_id);
            }
        }
        let _ = save_openclaw_config(cfg);
    }

    info!("[Skills] Skill {} deleted", skill_id);
    Ok(format!("Skill {} deleted", skill_id))
}

#[command]
pub async fn install_skill_from_zip(params: InstallSkillParams) -> Result<InstallSkillResult, String> {
    info!("[Skills] Installing from ZIP: {}", params.zip_path);

    if !Path::new(&params.zip_path).exists() {
        return Err("ZIP file not found".to_string());
    }

    let temp_dir = tempfile::tempdir().map_err(|e| format!("Failed to create temp dir: {}", e))?;
    let temp_path = temp_dir.path().to_string_lossy().to_string();

    {
        let file = File::open(&params.zip_path).map_err(|e| format!("Failed to open ZIP: {}", e))?;
        let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("Failed to parse ZIP: {}", e))?;

        for i in 0..archive.len() {
            let mut file_in_zip = archive.by_index(i).map_err(|e| e.to_string())?;
            let outpath = match file_in_zip.enclosed_name() {
                Some(path) => Path::new(&temp_path).join(path),
                None => continue,
            };

            if file_in_zip.name().ends_with('/') {
                fs::create_dir_all(&outpath).map_err(|e| format!("Failed to create directory: {}", e))?;
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        fs::create_dir_all(p).map_err(|e| format!("Failed to create directory: {}", e))?;
                    }
                }
                let mut outfile = File::create(&outpath).map_err(|e| format!("Failed to create file: {}", e))?;
                std::io::copy(&mut file_in_zip, &mut outfile).map_err(|e| format!("Failed to extract: {}", e))?;
            }
        }
    }

    let mut skill_dirs: Vec<String> = Vec::new();
    find_skill_directories(&temp_path, &mut skill_dirs);

    if skill_dirs.is_empty() {
        return Err("No valid skill directory found in ZIP".to_string());
    }

    let mut warnings = Vec::new();
    let mut installed_skill_id: Option<String> = None;
    let mut installed_skill_name: Option<String> = None;

    for skill_dir in skill_dirs {
        let skill_file = join_path(&skill_dir, SKILL_FILENAME);
        if !Path::new(&skill_file).exists() {
            continue;
        }

        let content = file::read_file(&skill_file).map_err(|e| format!("Failed to read SKILL.md: {}", e))?;

        if let Some(frontmatter) = parse_skill_frontmatter(&content) {
            let name = frontmatter.get("name").and_then(|v| v.as_str()).unwrap_or("unknown");
            let skill_id = name.to_lowercase().replace(' ', "-").replace('_', "-");

            let scan_warnings = scan_skill_content(&content);
            warnings.extend(scan_warnings);

            let target_dir = if let Some(ref agent_id) = params.agent_id {
                let skills_dir = get_agent_skills_dir(agent_id);
                join_path(&skills_dir, &skill_id)
            } else {
                let managed_dir = get_managed_skills_dir();
                join_path(&managed_dir, &skill_id)
            };

            if Path::new(&target_dir).exists() {
                warnings.push(format!("Skill '{}' already exists, will be overwritten", skill_id));
                fs::remove_dir_all(&target_dir).map_err(|e| format!("Failed to remove old directory: {}", e))?;
            }

            copy_dir_all(&skill_dir, &target_dir).map_err(|e| format!("Failed to copy files: {}", e))?;

            installed_skill_id = Some(skill_id);
            installed_skill_name = Some(name.to_string());

            info!("[Skills] Skill {} installed successfully", name);
        }
    }

    Ok(InstallSkillResult {
        success: true,
        skill_id: installed_skill_id,
        name: installed_skill_name,
        error: None,
        warnings,
    })
}

fn find_skill_directories(dir: &str, results: &mut Vec<String>) {
    let root_skill_file = join_path(dir, SKILL_FILENAME);
    if Path::new(&root_skill_file).exists() {
        results.push(dir.to_string());
        return;
    }

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let skill_file = join_path(&path.to_string_lossy(), SKILL_FILENAME);
                if Path::new(&skill_file).exists() {
                    results.push(path.to_string_lossy().to_string());
                } else {
                    find_skill_directories(&path.to_string_lossy(), results);
                }
            }
        }
    }
}

fn scan_skill_content(content: &str) -> Vec<String> {
    let mut warnings = Vec::new();
    let content_lower = content.to_lowercase();

    let dangerous_patterns = [
        ("eval(", "dynamic code execution"),
        ("exec(", "command execution"),
        ("subprocess", "subprocess call"),
        ("os.system", "system command"),
        ("child_process", "child process"),
        ("crypto", "crypto currency related"),
        ("mining", "mining related"),
    ];

    for (pattern, desc) in dangerous_patterns {
        if content_lower.contains(pattern) {
            warnings.push(format!("Detected potential risk: {} ({})", desc, pattern));
        }
    }

    warnings
}

fn copy_dir_all(src: &str, dst: &str) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = Path::new(dst).join(entry.file_name());

        if ty.is_dir() {
            let dir_name = entry.file_name().to_string_lossy().to_string();
            if EXCLUDE_DIRS.contains(&dir_name.as_str()) {
                continue;
            }
            copy_dir_all(&src_path.to_string_lossy(), &dst_path.to_string_lossy())?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

#[command]
pub async fn export_skill(params: ExportSkillParams) -> Result<ExportSkillResult, String> {
    info!("[Skills] Exporting skill: {}", params.skill_id);

    let skills = get_skills_list().await?;
    let skill = skills.skills.iter().find(|s| s.id == params.skill_id)
        .ok_or_else(|| format!("Skill '{}' not found", params.skill_id))?;

    let skill_dir = get_skill_dir(&skill.id, &skill.source, None);
    if !Path::new(&skill_dir).exists() {
        return Err(format!("Skill directory not found: {}", skill_dir));
    }

    let output_dir = params.output_dir.unwrap_or_else(|| {
        let config_dir = platform::get_config_dir();
        join_path(&config_dir, "exports")
    });

    fs::create_dir_all(&output_dir).map_err(|e| format!("Failed to create export directory: {}", e))?;

    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let output_filename = format!("{}-{}.zip", skill.id, timestamp);
    let output_path = join_path(&output_dir, &output_filename);

    {
        let file = File::create(&output_path).map_err(|e| format!("Failed to create ZIP: {}", e))?;
        let mut zip = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        add_dir_to_zip(&mut zip, &skill_dir, &skill.id, &options)?;
        zip.finish().map_err(|e| format!("Failed to finalize ZIP: {}", e))?;
    }

    info!("[Skills] Skill {} exported to {}", skill.id, output_path);

    Ok(ExportSkillResult {
        success: true,
        output_path: Some(output_path),
        error: None,
    })
}

fn add_dir_to_zip<W: std::io::Write + std::io::Seek>(
    zip: &mut zip::ZipWriter<W>,
    dir: &str,
    prefix: &str,
    options: &zip::write::SimpleFileOptions,
) -> Result<(), String> {
    for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let ty = entry.file_type().map_err(|e| e.to_string())?;
        let name = entry.file_name().to_string_lossy().to_string();
        let path = entry.path();
        let zip_path = format!("{}/{}", prefix, name);

        if ty.is_dir() {
            if EXCLUDE_DIRS.contains(&name.as_str()) {
                continue;
            }
            zip.add_directory(&zip_path, *options).map_err(|e| e.to_string())?;
            add_dir_to_zip(zip, &path.to_string_lossy(), &zip_path, options)?;
        } else {
            zip.start_file(&zip_path, *options).map_err(|e| e.to_string())?;
            let mut f = File::open(&path).map_err(|e| e.to_string())?;
            let mut buffer = Vec::new();
            f.read_to_end(&mut buffer).map_err(|e| e.to_string())?;
            zip.write_all(&buffer).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

#[command]
pub async fn open_skill_directory(skill_id: String) -> Result<String, String> {
    info!("[Skills] Opening directory for: {}", skill_id);

    let skills = get_skills_list().await?;
    let skill = skills.skills.iter().find(|s| s.id == skill_id)
        .ok_or_else(|| format!("Skill '{}' not found", skill_id))?;

    let skill_dir = get_skill_dir(&skill.id, &skill.source, None);
    if !Path::new(&skill_dir).exists() {
        return Err(format!("Skill directory not found: {}", skill_dir));
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer").arg(&skill_dir).spawn().map_err(|e| format!("Failed to open directory: {}", e))?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(&skill_dir).spawn().map_err(|e| format!("Failed to open directory: {}", e))?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open").arg(&skill_dir).spawn().map_err(|e| format!("Failed to open directory: {}", e))?;
    }

    info!("[Skills] Opened directory: {}", skill_dir);
    Ok(skill_dir)
}

#[command]
pub async fn install_skill_dependency(skill_id: String, install_id: String) -> Result<SkillInstallResult, String> {
    info!("[Skills] Installing dependency for: {} - {}", skill_id, install_id);

    let skills = get_skills_list().await?;
    let skill = skills.skills.iter().find(|s| s.id == skill_id)
        .ok_or_else(|| format!("Skill '{}' not found", skill_id))?;

    let install_option = skill.install_options.iter().find(|o| o.id == install_id)
        .ok_or_else(|| format!("Install option '{}' not found", install_id))?;

    match install_option.kind.as_str() {
        "brew" => {
            #[cfg(target_os = "macos")]
            {
                let formula = install_option.label.split_whitespace().last().unwrap_or(&install_option.id);
                let output = std::process::Command::new("brew")
                    .args(["install", formula])
                    .output()
                    .map_err(|e| format!("Failed to run brew install: {}", e))?;

                if output.status.success() {
                    return Ok(SkillInstallResult {
                        success: true,
                        message: format!("Installed {} via Homebrew", formula),
                        install_id: Some(install_id),
                    });
                } else {
                    return Err(format!("Install failed: {}", String::from_utf8_lossy(&output.stderr)));
                }
            }
            #[cfg(not(target_os = "macos"))]
            {
                return Err("Homebrew only supported on macOS".to_string());
            }
        }
        "node" => {
            let package = install_option.label.split_whitespace().last().unwrap_or(&install_option.id);
            let output = std::process::Command::new("npm")
                .args(["install", "-g", package])
                .output()
                .map_err(|e| format!("Failed to run npm install: {}", e))?;

            if output.status.success() {
                return Ok(SkillInstallResult {
                    success: true,
                    message: format!("Installed {} via npm", package),
                    install_id: Some(install_id),
                });
            } else {
                return Err(format!("Install failed: {}", String::from_utf8_lossy(&output.stderr)));
            }
        }
        _ => {
            return Err(format!("Unsupported install type: {}", install_option.kind));
        }
    }
}

#[command]
pub async fn get_agent_skills(agent_id: String) -> Result<Vec<SkillInfo>, String> {
    info!("[Skills] Getting skills for agent: {}", agent_id);
    let config = load_openclaw_config().ok();
    let skills = scan_agent_skills_directory(&agent_id, config.as_ref());
    info!("[Skills] Agent {} has {} skills", agent_id, skills.len());
    Ok(skills)
}

#[command]
pub async fn assign_skill_to_agent(skill_id: String, agent_id: String) -> Result<String, String> {
    info!("[Skills] Assigning skill {} to agent {}", skill_id, agent_id);

    let skills = get_skills_list().await?;
    let skill = skills.skills.iter().find(|s| s.id == skill_id)
        .ok_or_else(|| format!("Skill '{}' not found", skill_id))?;

    let source_dir = get_skill_dir(&skill.id, &skill.source, None);
    if !Path::new(&source_dir).exists() {
        return Err(format!("Skill directory not found: {}", source_dir));
    }

    let agent_skills_dir = get_agent_skills_dir(&agent_id);
    fs::create_dir_all(&agent_skills_dir).map_err(|e| format!("Failed to create agent skills directory: {}", e))?;

    let target_dir = join_path(&agent_skills_dir, &skill_id);
    if Path::new(&target_dir).exists() {
        fs::remove_dir_all(&target_dir).map_err(|e| format!("Failed to remove old directory: {}", e))?;
    }

    copy_dir_all(&source_dir, &target_dir).map_err(|e| format!("Failed to copy files: {}", e))?;

    info!("[Skills] Skill {} assigned to agent {}", skill_id, agent_id);
    Ok(format!("Skill {} assigned to agent {}", skill_id, agent_id))
}

#[command]
pub async fn remove_skill_from_agent(skill_id: String, agent_id: String) -> Result<String, String> {
    info!("[Skills] Removing skill {} from agent {}", skill_id, agent_id);

    let agent_skills_dir = get_agent_skills_dir(&agent_id);
    let skill_dir = join_path(&agent_skills_dir, &skill_id);

    if !Path::new(&skill_dir).exists() {
        return Err(format!("Agent {} does not have skill {}", agent_id, skill_id));
    }

    fs::remove_dir_all(&skill_dir).map_err(|e| format!("Failed to delete directory: {}", e))?;

    info!("[Skills] Skill {} removed from agent {}", skill_id, agent_id);
    Ok(format!("Skill {} removed from agent {}", skill_id, agent_id))
}
