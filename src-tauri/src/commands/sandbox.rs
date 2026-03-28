use crate::models::{
    SandboxConfig, SandboxMode, SandboxSecurityValidation, SandboxStatus,
};
use std::path::PathBuf;

#[tauri::command]
pub async fn get_sandbox_status() -> Result<SandboxStatus, String> {
    Ok(SandboxStatus {
        enabled: false,
        mode: SandboxMode::Off,
        docker_available: false,
        docker_version: None,
        running_containers: 0,
    })
}

#[tauri::command]
pub async fn get_sandbox_config() -> Result<SandboxConfig, String> {
    let config_path = get_sandbox_config_path()?;

    if !config_path.exists() {
        return Ok(SandboxConfig::default());
    }

    let content = tokio::fs::read_to_string(&config_path)
        .await
        .map_err(|e| format!("读取沙箱配置失败: {}", e))?;

    let config: SandboxConfig = serde_json::from_str(&content).unwrap_or_default();

    Ok(config)
}

#[tauri::command]
pub async fn save_sandbox_config(config: SandboxConfig) -> Result<(), String> {
    let validation = validate_sandbox_config(&config);
    if !validation.valid {
        return Err(format!("沙箱配置验证失败: {}", validation.errors.join(", ")));
    }

    let config_path = get_sandbox_config_path()?;

    if let Some(parent) = config_path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| format!("创建配置目录失败: {}", e))?;
    }

    let content = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("序列化配置失败: {}", e))?;

    tokio::fs::write(&config_path, content)
        .await
        .map_err(|e| format!("保存沙箱配置失败: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn validate_sandbox_config_cmd(config: SandboxConfig) -> Result<SandboxSecurityValidation, String> {
    Ok(validate_sandbox_config(&config))
}

fn get_sandbox_config_path() -> Result<PathBuf, String> {
    let home_dir = dirs::home_dir()
        .ok_or_else(|| "无法获取用户主目录".to_string())?;
    Ok(home_dir.join(".openclaw").join("sandbox.json"))
}

fn validate_sandbox_config(config: &SandboxConfig) -> SandboxSecurityValidation {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    SandboxSecurityValidation {
        valid: errors.is_empty(),
        errors,
        warnings,
    }
}