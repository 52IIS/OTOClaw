use crate::models::{
    SandboxConfig, SandboxContainerInfo, SandboxMode, SandboxSecurityValidation, SandboxStatus,
    BLOCKED_ENV_PATTERNS, BLOCKED_HOST_PATHS,
};
use crate::utils::platform;
use std::process::Command;

/// 获取沙箱状态
#[tauri::command]
pub async fn get_sandbox_status() -> Result<SandboxStatus, String> {
    let docker_available = check_docker_available();
    let docker_version = if docker_available {
        get_docker_version().await.ok()
    } else {
        None
    };

    let containers = if docker_available {
        list_sandbox_containers().await.unwrap_or_default()
    } else {
        vec![]
    };

    let running_containers = containers.iter().filter(|c| c.status == "running").count() as u32;

    let config = get_sandbox_config().await?;

    Ok(SandboxStatus {
        enabled: config.mode != SandboxMode::Off,
        mode: config.mode,
        docker_available,
        docker_version,
        running_containers,
        containers,
    })
}

/// 获取沙箱配置
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

/// 保存沙箱配置
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

/// 验证沙箱配置
#[tauri::command]
pub async fn validate_sandbox_config_cmd(config: SandboxConfig) -> Result<SandboxSecurityValidation, String> {
    Ok(validate_sandbox_config(&config))
}

/// 列出沙箱容器
#[tauri::command]
pub async fn list_sandbox_containers() -> Result<Vec<SandboxContainerInfo>, String> {
    if !check_docker_available() {
        return Err("Docker 不可用".to_string());
    }

    let output = Command::new("docker")
        .args([
            "ps",
            "-a",
            "--filter",
            "label=openclaw.sandbox=1",
            "--format",
            "{{.Names}}|{{.Image}}|{{.Status}}|{{.CreatedAt}}",
        ])
        .output()
        .map_err(|e| format!("执行 docker ps 失败: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("获取容器列表失败: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut containers = Vec::new();

    for line in stdout.lines() {
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() >= 4 {
            let name = parts[0].to_string();
            let session_key = get_container_label(&name, "openclaw.sessionKey")
                .await
                .unwrap_or_default();
            let config_hash = get_container_label(&name, "openclaw.configHash")
                .await
                .ok();

            let status = if parts[2].starts_with("Up") {
                "running".to_string()
            } else {
                "stopped".to_string()
            };

            containers.push(SandboxContainerInfo {
                name: name.clone(),
                session_key,
                image: parts[1].to_string(),
                status,
                created_at: parts[3].to_string(),
                last_used_at: parts[3].to_string(),
                config_hash,
            });
        }
    }

    Ok(containers)
}

/// 停止沙箱容器
#[tauri::command]
pub async fn stop_sandbox_container(container_name: String) -> Result<(), String> {
    if !check_docker_available() {
        return Err("Docker 不可用".to_string());
    }

    let output = Command::new("docker")
        .args(["stop", &container_name])
        .output()
        .map_err(|e| format!("执行 docker stop 失败: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("停止容器失败: {}", stderr));
    }

    Ok(())
}

/// 删除沙箱容器
#[tauri::command]
pub async fn remove_sandbox_container(container_name: String) -> Result<(), String> {
    if !check_docker_available() {
        return Err("Docker 不可用".to_string());
    }

    let output = Command::new("docker")
        .args(["rm", "-f", &container_name])
        .output()
        .map_err(|e| format!("执行 docker rm 失败: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("删除容器失败: {}", stderr));
    }

    Ok(())
}

/// 清理所有沙箱容器
#[tauri::command]
pub async fn prune_sandbox_containers() -> Result<u32, String> {
    if !check_docker_available() {
        return Err("Docker 不可用".to_string());
    }

    let containers = list_sandbox_containers().await?;
    let mut removed = 0u32;

    for container in containers {
        if remove_sandbox_container(container.name).await.is_ok() {
            removed += 1;
        }
    }

    Ok(removed)
}

/// 重建沙箱容器
#[tauri::command]
pub async fn recreate_sandbox_container(container_name: String) -> Result<(), String> {
    if !check_docker_available() {
        return Err("Docker 不可用".to_string());
    }

    remove_sandbox_container(container_name.clone()).await?;

    Ok(())
}

/// 检查 Docker 是否可用
#[tauri::command]
pub async fn check_docker_available_cmd() -> Result<bool, String> {
    Ok(check_docker_available())
}

/// 获取 Docker 版本
#[tauri::command]
pub async fn get_docker_version_cmd() -> Result<String, String> {
    get_docker_version().await
}

/// 拉取沙箱镜像
#[tauri::command]
pub async fn pull_sandbox_image(image: String) -> Result<(), String> {
    if !check_docker_available() {
        return Err("Docker 不可用".to_string());
    }

    let output = Command::new("docker")
        .args(["pull", &image])
        .output()
        .map_err(|e| format!("执行 docker pull 失败: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("拉取镜像失败: {}", stderr));
    }

    Ok(())
}

/// 构建沙箱镜像
#[tauri::command]
pub async fn build_sandbox_image() -> Result<String, String> {
    if !check_docker_available() {
        return Err("Docker 不可用".to_string());
    }

    let openclaw_dir = platform::get_config_dir();
    let openclaw_path = std::path::Path::new(&openclaw_dir);
    let dockerfile_path = openclaw_path.join("Dockerfile.sandbox");

    if !dockerfile_path.exists() {
        return Err("找不到 Dockerfile.sandbox 文件".to_string());
    }

    let output = Command::new("docker")
        .args(["build", "-t", "openclaw-sandbox:bookworm-slim", "-f", "Dockerfile.sandbox", "."])
        .current_dir(openclaw_path)
        .output()
        .map_err(|e| format!("执行 docker build 失败: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("构建镜像失败: {}", stderr));
    }

    Ok("openclaw-sandbox:bookworm-slim".to_string())
}

// ============ 内部辅助函数 ============

fn check_docker_available() -> bool {
    Command::new("docker")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

async fn get_docker_version() -> Result<String, String> {
    let output = Command::new("docker")
        .args(["--version"])
        .output()
        .map_err(|e| format!("执行 docker --version 失败: {}", e))?;

    if !output.status.success() {
        return Err("获取 Docker 版本失败".to_string());
    }

    let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(version)
}

async fn get_container_label(container_name: &str, label: &str) -> Result<String, String> {
    let output = Command::new("docker")
        .args([
            "inspect",
            "-f",
            &format!("{{{{ index .Config.Labels \"{}\" }}}}", label),
            container_name,
        ])
        .output()
        .map_err(|e| format!("执行 docker inspect 失败: {}", e))?;

    if !output.status.success() {
        return Err("获取容器标签失败".to_string());
    }

    let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if value == "<no value>" || value.is_empty() {
        return Err("标签不存在".to_string());
    }

    Ok(value)
}

fn get_sandbox_config_path() -> Result<std::path::PathBuf, String> {
    let openclaw_dir = platform::get_config_dir();
    Ok(std::path::PathBuf::from(openclaw_dir).join("sandbox.json"))
}

fn validate_sandbox_config(config: &SandboxConfig) -> SandboxSecurityValidation {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // 验证绑定挂载
    for bind in &config.docker.binds {
        let parts: Vec<&str> = bind.split(':').collect();
        if parts.is_empty() {
            errors.push(format!("无效的绑定挂载格式: {}", bind));
            continue;
        }

        let source = parts[0];

        // 检查是否为绝对路径
        if !source.starts_with('/') && cfg!(not(target_os = "windows")) {
            errors.push(format!("绑定挂载源路径必须是绝对路径: {}", source));
            continue;
        }

        // 检查是否为被阻止的路径
        for blocked in BLOCKED_HOST_PATHS {
            if source == *blocked || source.starts_with(&format!("{}/", blocked)) {
                errors.push(format!(
                    "绑定挂载源路径 '{}' 指向被阻止的系统目录 '{}'",
                    source, blocked
                ));
            }
        }
    }

    // 验证网络模式
    if let Some(ref network) = config.docker.network {
        if network == "host" {
            errors.push(
                "网络模式 'host' 会绕过容器网络隔离，请使用 'bridge' 或 'none'".to_string(),
            );
        }
        if network.starts_with("container:") && !config.docker.dangerously_allow_container_namespace_join {
            warnings.push(
                "网络模式 'container:*' 会加入另一个容器的命名空间，可能绕过沙箱隔离".to_string(),
            );
        }
    }

    // 验证 seccomp 配置
    if let Some(ref profile) = config.docker.seccomp_profile {
        if profile.to_lowercase() == "unconfined" {
            errors.push("seccomp 配置 'unconfined' 会禁用系统调用过滤，削弱沙箱隔离".to_string());
        }
    }

    // 验证 AppArmor 配置
    if let Some(ref profile) = config.docker.apparmor_profile {
        if profile.to_lowercase() == "unconfined" {
            errors.push("AppArmor 配置 'unconfined' 会禁用强制访问控制，削弱沙箱隔离".to_string());
        }
    }

    // 验证环境变量
    for (key, _) in &config.docker.env {
        let key_upper = key.to_uppercase();
        for pattern in BLOCKED_ENV_PATTERNS {
            if key_upper == *pattern || key_upper.ends_with(&format!("_{}", pattern)) {
                warnings.push(format!(
                    "环境变量 '{}' 可能包含敏感信息，建议不要在沙箱中设置",
                    key
                ));
            }
        }
    }

    // 验证危险选项
    if config.docker.dangerously_allow_reserved_container_targets {
        warnings.push("dangerously_allow_reserved_container_targets 已启用，可能覆盖沙箱挂载".to_string());
    }
    if config.docker.dangerously_allow_external_bind_sources {
        warnings.push("dangerously_allow_external_bind_sources 已启用，可能允许访问工作区外的文件".to_string());
    }
    if config.docker.dangerously_allow_container_namespace_join {
        warnings.push("dangerously_allow_container_namespace_join 已启用，可能绕过网络隔离".to_string());
    }

    SandboxSecurityValidation {
        valid: errors.is_empty(),
        errors,
        warnings,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_sandbox_config_default() {
        let config = SandboxConfig::default();
        let validation = validate_sandbox_config(&config);
        assert!(validation.valid);
    }

    #[test]
    fn test_validate_sandbox_config_host_network() {
        let mut config = SandboxConfig::default();
        config.docker.network = Some("host".to_string());
        let validation = validate_sandbox_config(&config);
        assert!(!validation.valid);
        assert!(validation.errors.iter().any(|e| e.contains("host")));
    }

    #[test]
    fn test_validate_sandbox_config_blocked_path() {
        let mut config = SandboxConfig::default();
        config.docker.binds.push("/etc:/etc:ro".to_string());
        let validation = validate_sandbox_config(&config);
        assert!(!validation.valid);
        assert!(validation.errors.iter().any(|e| e.contains("/etc")));
    }

    #[test]
    fn test_validate_sandbox_config_unconfined_seccomp() {
        let mut config = SandboxConfig::default();
        config.docker.seccomp_profile = Some("unconfined".to_string());
        let validation = validate_sandbox_config(&config);
        assert!(!validation.valid);
        assert!(validation.errors.iter().any(|e| e.contains("seccomp")));
    }
}
