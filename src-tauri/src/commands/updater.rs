use crate::models::updater::*;
use crate::utils::platform;
use chrono::Utc;
use dirs::home_dir;
use log::{error, info, warn};
use reqwest::Client;
use semver::Version;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tauri::{command, AppHandle, Emitter};

const GITHUB_REPO: &str = "52IIS/OTOClaw";
const GITHUB_API_URL: &str = "https://api.github.com/repos";
const CONFIG_FILE_NAME: &str = "otoclaw_config.json";

static UPDATE_IN_PROGRESS: AtomicBool = AtomicBool::new(false);

fn get_config_path() -> PathBuf {
    home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".openclaw")
        .join(CONFIG_FILE_NAME)
}

fn load_update_config() -> UpdateConfig {
    let config_path = get_config_path();
    
    if config_path.exists() {
        match fs::read_to_string(&config_path) {
            Ok(content) => {
                match serde_json::from_str::<UpdateConfig>(&content) {
                    Ok(config) => {
                        info!("[更新配置] 加载成功: {:?}", config.mode);
                        return config;
                    }
                    Err(e) => {
                        warn!("[更新配置] 解析失败，使用默认配置: {}", e);
                    }
                }
            }
            Err(e) => {
                warn!("[更新配置] 读取失败，使用默认配置: {}", e);
            }
        }
    }
    
    UpdateConfig::default()
}

fn save_update_config(config: &UpdateConfig) -> Result<(), String> {
    let config_path = get_config_path();
    
    if let Some(parent) = config_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(|e| format!("创建配置目录失败: {}", e))?;
        }
    }
    
    let content = serde_json::to_string_pretty(config).map_err(|e| format!("序列化配置失败: {}", e))?;
    fs::write(&config_path, content).map_err(|e| format!("写入配置文件失败: {}", e))?;
    
    info!("[更新配置] 保存成功");
    Ok(())
}

#[command]
pub fn get_update_config() -> Result<UpdateConfig, String> {
    Ok(load_update_config())
}

#[command]
pub fn save_update_config_cmd(config: UpdateConfig) -> Result<String, String> {
    save_update_config(&config)?;
    Ok("配置保存成功".to_string())
}

fn get_current_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

async fn fetch_latest_release() -> Result<GitHubRelease, String> {
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .user_agent(format!("OTOClaw/{}", get_current_version()))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;
    
    let url = format!("{}/{}/releases/latest", GITHUB_API_URL, GITHUB_REPO);
    info!("[版本检查] 请求 URL: {}", url);
    
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("请求 GitHub API 失败: {}", e))?;
    
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("GitHub API 返回错误: {} - {}", status, body));
    }
    
    let release: GitHubRelease = response
        .json()
        .await
        .map_err(|e| format!("解析 GitHub 响应失败: {}", e))?;
    
    Ok(release)
}

fn get_platform_asset_patterns() -> Vec<String> {
    let os = platform::get_os();
    let arch = if cfg!(target_arch = "x86_64") {
        "x64"
    } else if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else {
        "unknown"
    };
    
    match os.as_str() {
        "windows" => vec![
            format!("*_{}-setup.exe", arch),
            format!("*_{}_*.msi", arch),
        ],
        "macos" => vec![
            "*_universal.dmg".to_string(),
            format!("*_{}.dmg", arch),
        ],
        "linux" => vec![
            "*_amd64.AppImage".to_string(),
            "*_x64.AppImage".to_string(),
        ],
        _ => vec![format!("*_{}*", arch)],
    }
}

fn matches_pattern(name: &str, pattern: &str) -> bool {
    let pattern = pattern.trim_start_matches('*');
    if pattern.starts_with('*') {
        name.contains(&pattern[1..])
    } else if pattern.ends_with('*') {
        name.starts_with(pattern.trim_end_matches('*'))
    } else {
        name.ends_with(pattern)
    }
}

fn compare_versions(current: &str, latest: &str) -> bool {
    let current = current.trim().trim_start_matches('v');
    let latest = latest.trim().trim_start_matches('v');
    
    match (Version::parse(current), Version::parse(latest)) {
        (Ok(c), Ok(l)) => l > c,
        _ => {
            let current_parts: Vec<u32> = current
                .split('.')
                .filter_map(|s| s.parse().ok())
                .collect();
            let latest_parts: Vec<u32> = latest
                .split('.')
                .filter_map(|s| s.parse().ok())
                .collect();
            
            for i in 0..3 {
                let c = current_parts.get(i).unwrap_or(&0);
                let l = latest_parts.get(i).unwrap_or(&0);
                if l > c {
                    return true;
                } else if l < c {
                    return false;
                }
            }
            
            false
        }
    }
}

#[command]
pub async fn check_otoclaw_update() -> Result<OTOClawUpdateInfo, String> {
    info!("[版本检查] 开始检查 OTOClaw 更新...");
    
    let current_version = get_current_version();
    info!("[版本检查] 当前版本: {}", current_version);
    
    let release = match fetch_latest_release().await {
        Ok(r) => r,
        Err(e) => {
            error!("[版本检查] 获取最新版本失败: {}", e);
            return Ok(OTOClawUpdateInfo {
                update_available: false,
                current_version,
                latest_version: String::new(),
                release_notes: None,
                download_url: None,
                file_size: None,
                error: Some(e),
            });
        }
    };
    
    let latest_version = release.tag_name.trim_start_matches('v').to_string();
    info!("[版本检查] 最新版本: {}", latest_version);
    
    let update_available = compare_versions(&current_version, &latest_version);
    info!("[版本检查] 是否有更新: {}", update_available);
    
    let patterns = get_platform_asset_patterns();
    info!("[版本检查] 平台资源匹配模式: {:?}", patterns);
    
    let asset = release.assets.iter().find(|a| {
        patterns.iter().any(|p| matches_pattern(&a.name, p))
    });
    
    let (download_url, file_size) = match asset {
        Some(a) => {
            info!("[版本检查] 找到匹配的资源: {}", a.name);
            (Some(a.browser_download_url.clone()), Some(a.size))
        }
        None => {
            warn!("[版本检查] 未找到匹配的资源文件，可用资源: {:?}", release.assets.iter().map(|a| &a.name).collect::<Vec<_>>());
            (None, None)
        }
    };
    
    let mut config = load_update_config();
    config.last_check_time = Some(Utc::now());
    let _ = save_update_config(&config);
    
    Ok(OTOClawUpdateInfo {
        update_available,
        current_version,
        latest_version,
        release_notes: release.body,
        download_url,
        file_size,
        error: None,
    })
}

#[command]
pub async fn download_update(
    app: AppHandle,
    download_url: String,
) -> Result<String, String> {
    if UPDATE_IN_PROGRESS.load(Ordering::SeqCst) {
        return Err("已有更新任务在进行中".to_string());
    }
    
    UPDATE_IN_PROGRESS.store(true, Ordering::SeqCst);
    
    info!("[下载更新] 开始下载: {}", download_url);
    
    let client = Client::builder()
        .timeout(Duration::from_secs(300))
        .user_agent(format!("OTOClaw/{}", get_current_version()))
        .build()
        .map_err(|e| {
            UPDATE_IN_PROGRESS.store(false, Ordering::SeqCst);
            format!("创建 HTTP 客户端失败: {}", e)
        })?;
    
    let response = client
        .get(&download_url)
        .send()
        .await
        .map_err(|e| {
            UPDATE_IN_PROGRESS.store(false, Ordering::SeqCst);
            format!("请求下载失败: {}", e)
        })?;
    
    if !response.status().is_success() {
        UPDATE_IN_PROGRESS.store(false, Ordering::SeqCst);
        return Err(format!("下载失败: HTTP {}", response.status()));
    }
    
    let total_size = response.content_length().unwrap_or(0);
    info!("[下载更新] 文件大小: {} bytes", total_size);
    
    let temp_dir = tempfile::tempdir().map_err(|e| {
        UPDATE_IN_PROGRESS.store(false, Ordering::SeqCst);
        format!("创建临时目录失败: {}", e)
    })?;
    
    let file_name = download_url.split('/').last().unwrap_or("update");
    let file_path = temp_dir.path().join(file_name);
    
    let mut file = fs::File::create(&file_path).map_err(|e| {
        UPDATE_IN_PROGRESS.store(false, Ordering::SeqCst);
        format!("创建文件失败: {}", e)
    })?;
    
    use futures_util::StreamExt;
    use std::io::Write;
    
    let mut stream = response.bytes_stream();
    let mut downloaded: u64 = 0;
    let start_time = std::time::Instant::now();
    
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| {
            UPDATE_IN_PROGRESS.store(false, Ordering::SeqCst);
            format!("读取数据失败: {}", e)
        })?;
        
        file.write_all(&chunk).map_err(|e| {
            UPDATE_IN_PROGRESS.store(false, Ordering::SeqCst);
            format!("写入文件失败: {}", e)
        })?;
        
        downloaded += chunk.len() as u64;
        
        let percentage = if total_size > 0 {
            (downloaded as f64 / total_size as f64) * 100.0
        } else {
            0.0
        };
        
        let elapsed = start_time.elapsed().as_secs_f64();
        let speed = if elapsed > 0.0 {
            let bytes_per_sec = downloaded as f64 / elapsed;
            format!("{:.1} MB/s", bytes_per_sec / 1024.0 / 1024.0)
        } else {
            "计算中...".to_string()
        };
        
        let progress = DownloadProgress {
            downloaded,
            total: total_size,
            percentage,
            speed,
        };
        
        let _ = app.emit("update-download-progress", &progress);
    }
    
    info!("[下载更新] 下载完成: {:?}", file_path);
    
    UPDATE_IN_PROGRESS.store(false, Ordering::SeqCst);
    
    Ok(file_path.to_string_lossy().to_string())
}

#[command]
pub async fn install_update(file_path: String) -> Result<UpdateResult, String> {
    info!("[安装更新] 开始安装: {}", file_path);
    
    let path = PathBuf::from(&file_path);
    if !path.exists() {
        return Err("更新文件不存在".to_string());
    }
    
    let os = platform::get_os();
    
    match os.as_str() {
        "windows" => {
            let _ = std::process::Command::new(&file_path)
                .spawn()
                .map_err(|e| format!("启动安装程序失败: {}", e))?;
            
            std::process::exit(0);
        }
        "macos" => {
            let _ = std::process::Command::new("open")
                .arg(&file_path)
                .spawn()
                .map_err(|e| format!("打开 DMG 失败: {}", e))?;
            
            Ok(UpdateResult {
                success: true,
                message: "DMG 已打开，请手动完成安装".to_string(),
                error: None,
            })
        }
        "linux" => {
            let _ = std::process::Command::new("chmod")
                .args(["+x", &file_path])
                .spawn()
                .map_err(|e| format!("设置执行权限失败: {}", e))?;
            
            let _ = std::process::Command::new(&file_path)
                .spawn()
                .map_err(|e| format!("启动 AppImage 失败: {}", e))?;
            
            std::process::exit(0);
        }
        _ => Err(format!("不支持的操作系统: {}", os)),
    }
}

#[command]
pub fn get_app_version() -> Result<VersionInfo, String> {
    Ok(VersionInfo {
        version: get_current_version(),
        build_number: 0,
        release_date: String::new(),
        release_notes: None,
    })
}

#[command]
pub fn skip_version(version: String) -> Result<String, String> {
    let mut config = load_update_config();
    config.skipped_version = Some(version);
    save_update_config(&config)?;
    Ok("已跳过此版本".to_string())
}

#[command]
pub fn cancel_update() -> Result<String, String> {
    UPDATE_IN_PROGRESS.store(false, Ordering::SeqCst);
    Ok("已取消更新".to_string())
}
