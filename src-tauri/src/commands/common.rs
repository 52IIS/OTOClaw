//! `commands` 模块公共函数库
//!
//! 该模块集中管理 commands 目录下各模块之间共享的公共函数，
//! 避免代码重复，提高代码复用率和可维护性。
//!
//! # 主要功能分类
//! - 配置管理：OpenClaw 配置文件读写
//! - 路径工具：跨平台路径拼接与目录路径获取
//! - 进程管理：端口检查、进程终止
//!
//! # 文本处理工具（已迁移至 `utils::text`）
//! - ANSI转义序列去除、JSON提取、版本比较、脱敏、时间戳等

use crate::utils::{file, platform};
use serde_json::{json, Value};
use std::path::PathBuf;

// ============================================================================
// 配置管理函数
// ============================================================================

/// 获取 OpenClaw 配置目录路径
///
/// 从平台模块获取用户配置目录，Windows 下通常为 `%APPDATA%/.openclaw`，
/// Unix-like 系统下为 `~/.openclaw`。
pub fn get_config_dir() -> String {
    platform::get_config_dir()
}

/// 获取 OpenClaw 主配置文件路径 (`openclaw.json`)
///
/// 将配置目录与文件名拼接，返回完整配置文件路径。
pub fn get_openclaw_config_path() -> PathBuf {
    PathBuf::from(platform::get_config_dir()).join("openclaw.json")
}

/// 加载 OpenClaw 配置文件
///
/// 读取 `openclaw.json` 文件并解析为 JSON Value。
/// 如果文件不存在，返回空 JSON 对象 `{}`。
///
/// # 返回值
/// * `Ok(Value)` - 解析后的配置 JSON
/// * `Err(String)` - 文件读取或解析失败
pub fn load_openclaw_config() -> Result<Value, String> {
    let config_path = get_openclaw_config_path();

    if !file::file_exists(&config_path.to_string_lossy()) {
        return Ok(json!({}));
    }

    let content =
        file::read_file(&config_path.to_string_lossy()).map_err(|e| format!("读取配置文件失败: {}", e))?;

    serde_json::from_str(&content).map_err(|e| format!("解析配置文件失败: {}", e))
}

/// 保存 OpenClaw 配置到文件
///
/// 将 JSON Value 以格式化方式写入 `openclaw.json`。
///
/// # 参数
/// * `config` - 要保存的配置 JSON 引用
///
/// # 返回值
/// * `Ok(())` - 保存成功
/// * `Err(String)` - 序列化或写入失败
pub fn save_openclaw_config(config: &Value) -> Result<(), String> {
    let config_path = get_openclaw_config_path();

    let content =
        serde_json::to_string_pretty(config).map_err(|e| format!("序列化配置失败: {}", e))?;

    file::write_file(&config_path.to_string_lossy(), &content)
        .map_err(|e| format!("写入配置文件失败: {}", e))
}

// ============================================================================
// 路径工具函数
// ============================================================================

/// 跨平台路径拼接
///
/// 根据操作系统自动选择正确的路径分隔符：
/// - Windows: `base\name`
/// - Unix: `base/name`
///
/// # 参数
/// * `base` - 基础路径
/// * `name` - 要拼接的名称
pub fn join_path(base: &str, name: &str) -> String {
    if platform::is_windows() {
        format!("{}\\{}", base, name)
    } else {
        format!("{}/{}", base, name)
    }
}

/// 在基础路径下创建子目录路径
///
/// 等同于 `join_path(base, subdir)` 的便捷包装。
pub fn get_sub_dir(base: &str, subdir: &str) -> String {
    join_path(base, subdir)
}

/// 获取配置目录下指定文件的全路径
///
/// # 参数
/// * `filename` - 文件名（不含路径）
pub fn get_store_path(filename: &str) -> String {
    let config_dir = get_config_dir();
    join_path(&config_dir, filename)
}

/// 获取智能体配置目录
///
/// 路径: `<config_dir>/agents`
pub fn get_agents_dir() -> String {
    get_sub_dir(&get_config_dir(), "agents")
}

/// 获取指定智能体的工作区目录
///
/// 每个智能体有独立的工作区，用于存储其运行时数据。
///
/// # 参数
/// * `agent_id` - 智能体唯一标识符
///
/// # 示例
/// ```
/// get_agent_workspace_dir("my-agent")
/// // -> "~/.openclaw/workspace-my-agent"
/// ```
pub fn get_agent_workspace_dir(agent_id: &str) -> String {
    get_sub_dir(&get_config_dir(), &format!("workspace-{}", agent_id))
}

/// 获取会话数据存储目录
///
/// 路径: `<config_dir>/sessions`
pub fn get_sessions_dir() -> String {
    get_sub_dir(&get_config_dir(), "sessions")
}

/// 根据类型获取技能目录
///
/// # 参数
/// * `skill_type` - 技能类型
///   - `"bundled"`: 内置/全局技能 `<config_dir>/skills`
///   - `"managed"`: 全局共享技能 `<config_dir>/skills`
pub fn get_skills_dir(skill_type: &str) -> String {
    match skill_type {
        "bundled" | "managed" => get_sub_dir(&get_config_dir(), "skills"),
        _ => get_sub_dir(&get_config_dir(), "skills"),
    }
}

/// 获取内置（捆绑）技能目录
///
/// 路径: `<config_dir>/skills`
pub fn get_bundled_skills_dir() -> String {
    get_skills_dir("bundled")
}

/// 获取全局共享技能目录
///
/// 路径: `<config_dir>/skills`
pub fn get_managed_skills_dir() -> String {
    get_skills_dir("managed")
}

/// 获取指定智能体的技能目录
///
/// 路径: `<agent_workspace>/skills`
///
/// # 参数
/// * `agent_id` - 智能体唯一标识符
pub fn get_agent_skills_dir(agent_id: &str) -> String {
    get_sub_dir(&get_agent_workspace_dir(agent_id), "skills")
}

/// 获取指定技能的工作目录
///
/// 根据技能来源和是否属于特定智能体确定目录位置。
///
/// # 参数
/// * `skill_id` - 技能唯一标识符
/// * `source` - 技能来源 (`"bundled"` 或 `"managed"`)
/// * `agent_id` - 所属智能体（可选，如果指定则在智能体工作区下查找）
pub fn get_skill_dir(skill_id: &str, source: &str, agent_id: Option<&str>) -> String {
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

/// 获取日志存储目录
///
/// 路径: `<config_dir>/logs`
pub fn get_logs_dir() -> String {
    get_sub_dir(&get_config_dir(), "logs")
}

/// 获取指定日志文件的完整路径
///
/// # 参数
/// * `filename` - 日志文件名
pub fn get_log_file_path(filename: &str) -> String {
    get_sub_dir(&get_logs_dir(), filename)
}

// ============================================================================
// 平台相关常量
// ============================================================================

/// Windows 路径分隔符
#[cfg(windows)]
pub const PATH_SEPARATOR: &str = "\\";

/// Unix 路径分隔符
#[cfg(not(windows))]
pub const PATH_SEPARATOR: &str = "/";

/// 获取当前平台路径分隔符
///
/// 返回静态字符串引用，避免每次调用都分配内存。
pub fn path_separator() -> &'static str {
    PATH_SEPARATOR
}

// ============================================================================
// 进程管理函数
// ============================================================================

/// Windows API 标志：创建进程时不创建窗口
#[cfg(windows)]
use std::os::windows::process::CommandExt;

/// Windows 隐藏窗口标志
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

/// 获取指定端口上的进程 PID 列表
pub fn get_pids_on_port(port: u16) -> Vec<u32> {
    #[cfg(windows)]
    {
        let output = std::process::Command::new("netstat")
            .args(["-ano"])
            .creation_flags(CREATE_NO_WINDOW)
            .output();
        if let Ok(out) = output {
            let stdout = String::from_utf8_lossy(&out.stdout);
            return stdout
                .lines()
                .filter(|line| line.contains(&format!(":{}", port)))
                .filter_map(|line| line.split_whitespace().last())
                .filter_map(|pid| pid.parse::<u32>().ok())
                .collect();
        }
        Vec::new()
    }

    #[cfg(unix)]
    {
        let output = std::process::Command::new("sh")
            .args(["-c", &format!("lsof -t -i:{}", port)])
            .output();
        if let Ok(out) = output {
            let stdout = String::from_utf8_lossy(&out.stdout);
            return stdout
                .lines()
                .filter_map(|pid| pid.trim().parse::<u32>().ok())
                .collect();
        }
        Vec::new()
    }
}

/// 终止指定 PID 的进程
pub fn kill_process(pid: u32, force: bool) {
    #[cfg(windows)]
    {
        let mut command = std::process::Command::new("taskkill");
        command.args(["/PID", &pid.to_string()]);
        if force {
            command.arg("/F");
        }
        let _ = command.creation_flags(CREATE_NO_WINDOW).output();
    }

    #[cfg(unix)]
    {
        let signal = if force { "-9" } else { "-15" };
        let _ = std::process::Command::new("kill")
            .args([signal, &pid.to_string()])
            .output();
    }
}

/// 检查指定端口是否有服务在监听
///
/// 使用 `lsof` (Unix) 或 `netstat` (Windows) 检查端口状态。
///
/// # 参数
/// * `port` - 要检查的端口号
///
/// # 返回值
/// * `true` - 端口正在被监听
/// * `false` - 端口空闲或查询失败
pub fn check_port_listening(port: u16) -> bool {
    #[cfg(unix)]
    {
        let output = std::process::Command::new("lsof")
            .args(["-i", &format!(":{}", port)])
            .output();
        matches!(output, Ok(o) if o.status.success())
    }

    #[cfg(windows)]
    {
        let output = std::process::Command::new("netstat")
            .args(["-ano"])
            .creation_flags(CREATE_NO_WINDOW)
            .output();

        if let Ok(out) = output {
            let stdout = String::from_utf8_lossy(&out.stdout);
            stdout.contains(&format!(":{}", port))
        } else {
            false
        }
    }
}

/// 终止指定端口占用的进程
///
/// # 参数
/// * `port` - 端口号
pub fn kill_port_process(port: u16) -> Result<(), String> {
    #[cfg(unix)]
    {
        let output = std::process::Command::new("sh")
            .args(["-c", &format!("lsof -t -i:{} | xargs kill -9", port)])
            .output()
            .map_err(|e| format!("执行 kill 失败: {}", e))?;

        if output.status.success() {
            Ok(())
        } else {
            Err(format!("终止端口进程失败: {}", String::from_utf8_lossy(&output.stderr)))
        }
    }

    #[cfg(windows)]
    {
        let find_pid_cmd = format!("for /f \"tokens=5\" %a in ('netstat -ano ^| findstr :{}') do taskkill /F /PID %a", port);
        let output = std::process::Command::new("cmd")
            .args(["/C", &find_pid_cmd])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map_err(|e| format!("执行 taskkill 失败: {}", e))?;

        if output.status.success() {
            Ok(())
        } else {
            Err(format!("终止端口进程失败: {}", String::from_utf8_lossy(&output.stderr)))
        }
    }
}

/// 检查命令是否存在于系统 PATH 中
pub fn command_exists(command: &str) -> bool {
    #[cfg(unix)]
    {
        let output = std::process::Command::new("which").arg(command).output();
        matches!(output, Ok(o) if o.status.success())
    }

    #[cfg(windows)]
    {
        let output = std::process::Command::new("where")
            .arg(command)
            .creation_flags(CREATE_NO_WINDOW)
            .output();
        matches!(output, Ok(o) if o.status.success())
    }
}

/// 运行命令并返回 stdout 文本
pub fn run_command_output(command: &str, args: &[&str]) -> Result<String, String> {
    let output = {
        #[cfg(windows)]
        {
            std::process::Command::new(command)
                .args(args)
                .creation_flags(CREATE_NO_WINDOW)
                .output()
        }
        #[cfg(not(windows))]
        {
            std::process::Command::new(command).args(args).output()
        }
    }
    .map_err(|e| format!("执行命令失败: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

/// 运行脚本并返回 stdout 文本
pub fn run_script_output(script: &str) -> Result<String, String> {
    #[cfg(unix)]
    {
        run_command_output("sh", &["-c", script])
    }

    #[cfg(windows)]
    {
        run_command_output("cmd", &["/C", script])
    }
}

/// 在后台运行脚本（不等待结果）
pub fn spawn_background(script: &str) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        std::process::Command::new("sh")
            .args(["-c", script])
            .spawn()?;
    }

    #[cfg(windows)]
    {
        std::process::Command::new("cmd")
            .args(["/C", script])
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()?;
    }

    Ok(())
}

/// 获取用户主目录
pub fn get_home_dir() -> String {
    dirs::home_dir()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string()
}

/// 获取临时目录
pub fn get_temp_dir() -> String {
    std::env::temp_dir().to_string_lossy().to_string()
}

/// 获取当前工作目录
pub fn get_current_dir() -> String {
    std::env::current_dir()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string()
}

/// 确保目录存在，不存在则创建
pub fn ensure_dir(path: &str) -> Result<(), String> {
    std::fs::create_dir_all(path).map_err(|e| format!("创建目录失败: {}", e))
}

/// 将相对路径转换为绝对路径
pub fn resolve_path(path: &str) -> String {
    let path_buf = PathBuf::from(path);
    if path_buf.is_absolute() {
        path.to_string()
    } else {
        std::env::current_dir()
            .unwrap_or_default()
            .join(path_buf)
            .to_string_lossy()
            .to_string()
    }
}

/// 归一化路径分隔符
pub fn normalize_path(path: &str) -> String {
    if path_separator() == "/" {
        path.replace('\\', "/")
    } else {
        path.replace('/', "\\")
    }
}

/// 获取文件名（不含路径）
pub fn get_filename(path: &str) -> Option<String> {
    PathBuf::from(path)
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
}

/// 获取文件扩展名
pub fn get_extension(path: &str) -> Option<String> {
    PathBuf::from(path)
        .extension()
        .map(|s| s.to_string_lossy().to_string())
}

/// 判断路径是否存在
pub fn path_exists(path: &str) -> bool {
    PathBuf::from(path).exists()
}

/// 判断是否为目录
pub fn is_dir(path: &str) -> bool {
    PathBuf::from(path).is_dir()
}

/// 判断是否为文件
pub fn is_file(path: &str) -> bool {
    PathBuf::from(path).is_file()
}

/// 删除文件
pub fn remove_file(path: &str) -> Result<(), String> {
    std::fs::remove_file(path).map_err(|e| format!("删除文件失败: {}", e))
}

/// 删除目录（递归）
pub fn remove_dir_all(path: &str) -> Result<(), String> {
    std::fs::remove_dir_all(path).map_err(|e| format!("删除目录失败: {}", e))
}

/// 重命名路径
pub fn rename_path(from: &str, to: &str) -> Result<(), String> {
    std::fs::rename(from, to).map_err(|e| format!("重命名失败: {}", e))
}

/// 复制文件
pub fn copy_file(from: &str, to: &str) -> Result<u64, String> {
    std::fs::copy(from, to).map_err(|e| format!("复制文件失败: {}", e))
}

/// 获取文件元数据
pub fn metadata(path: &str) -> Result<std::fs::Metadata, String> {
    std::fs::metadata(path).map_err(|e| format!("读取元数据失败: {}", e))
}

/// 创建空文件
pub fn touch_file(path: &str) -> Result<(), String> {
    std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(path)
        .map(|_| ())
        .map_err(|e| format!("创建文件失败: {}", e))
}

/// 获取文件大小
pub fn file_size(path: &str) -> Result<u64, String> {
    metadata(path).map(|m| m.len())
}

/// 获取目录大小（浅层）
pub fn dir_entry_count(path: &str) -> Result<usize, String> {
    std::fs::read_dir(path)
        .map_err(|e| format!("读取目录失败: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map(|entries| entries.len())
        .map_err(|e| format!("读取目录项失败: {}", e))
}

/// 判断路径是否为空目录
pub fn is_empty_dir(path: &str) -> Result<bool, String> {
    let mut iter = std::fs::read_dir(path).map_err(|e| format!("读取目录失败: {}", e))?;
    Ok(iter.next().is_none())
}

/// 拼接多个路径片段
pub fn join_paths(parts: &[&str]) -> String {
    let mut iter = parts.iter();
    let Some(first) = iter.next() else {
        return String::new();
    };
    let mut path = PathBuf::from(first);
    for part in iter {
        path.push(part);
    }
    path.to_string_lossy().to_string()
}

/// 获取父目录
pub fn parent_dir(path: &str) -> Option<String> {
    PathBuf::from(path)
        .parent()
        .map(|p| p.to_string_lossy().to_string())
}

/// 获取规范化绝对路径
pub fn canonicalize_path(path: &str) -> Result<String, String> {
    std::fs::canonicalize(path)
        .map(|p| p.to_string_lossy().to_string())
        .map_err(|e| format!("规范化路径失败: {}", e))
}

/// 获取配置目录下的子目录
pub fn get_config_sub_dir(name: &str) -> String {
    get_sub_dir(&get_config_dir(), name)
}

/// 判断字符串是否为空白
pub fn is_blank(value: &str) -> bool {
    value.trim().is_empty()
}

/// 对路径做波浪线展开
pub fn expand_tilde(path: &str) -> String {
    if path == "~" {
        return get_home_dir();
    }
    if let Some(rest) = path.strip_prefix("~/") {
        return join_path(&get_home_dir(), rest);
    }
    path.to_string()
}

/// 对路径做最基础清洗
pub fn sanitize_path(path: &str) -> String {
    normalize_path(path).trim().to_string()
}

/// 构建 `<config_dir>/workspace-<agent_id>`
pub fn build_agent_workspace_path(agent_id: &str) -> String {
    get_agent_workspace_dir(agent_id)
}

/// 构建 `<config_dir>/skills/<skill_id>`
pub fn build_global_skill_path(skill_id: &str) -> String {
    join_path(&get_managed_skills_dir(), skill_id)
}

/// 构建 `<workspace>/skills/<skill_id>`
pub fn build_agent_skill_path(agent_id: &str, skill_id: &str) -> String {
    join_path(&get_agent_skills_dir(agent_id), skill_id)
}

/// 配置文件是否存在
pub fn openclaw_config_exists() -> bool {
    get_openclaw_config_path().exists()
}

/// 获取配置文件路径字符串
pub fn get_openclaw_config_path_string() -> String {
    get_openclaw_config_path().to_string_lossy().to_string()
}

/// 读取配置目录中的 JSON 文件
pub fn load_json_from_store(filename: &str) -> Result<Value, String> {
    let path = get_store_path(filename);
    if !file::file_exists(&path) {
        return Ok(json!({}));
    }
    let content = file::read_file(&path).map_err(|e| format!("读取 JSON 文件失败: {}", e))?;
    serde_json::from_str(&content).map_err(|e| format!("解析 JSON 文件失败: {}", e))
}

/// 保存 JSON 到配置目录
pub fn save_json_to_store(filename: &str, value: &Value) -> Result<(), String> {
    let path = get_store_path(filename);
    let content = serde_json::to_string_pretty(value).map_err(|e| format!("序列化 JSON 失败: {}", e))?;
    file::write_file(&path, &content).map_err(|e| format!("写入 JSON 文件失败: {}", e))
}

/// 读取文本文件，若不存在返回空字符串
pub fn read_text_or_default(path: &str) -> Result<String, String> {
    if !file::file_exists(path) {
        return Ok(String::new());
    }
    file::read_file(path).map_err(|e| format!("读取文本文件失败: {}", e))
}

/// 写入文本文件
pub fn write_text(path: &str, content: &str) -> Result<(), String> {
    file::write_file(path, content).map_err(|e| format!("写入文本文件失败: {}", e))
}

/// 读取环境变量，不存在返回默认值
pub fn env_or_default(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

/// 判断环境变量是否存在且非空
pub fn has_env(key: &str) -> bool {
    std::env::var(key).map(|v| !v.trim().is_empty()).unwrap_or(false)
}

/// 读取当前用户名
pub fn get_username() -> String {
    env_or_default("USERNAME", &env_or_default("USER", ""))
}

/// 获取主机名
pub fn get_hostname() -> String {
    env_or_default("COMPUTERNAME", &env_or_default("HOSTNAME", ""))
}

/// 读取 PATH
pub fn get_path_env() -> String {
    env_or_default("PATH", "")
}

/// 拆分 PATH
pub fn split_path_env() -> Vec<String> {
    std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default())
        .map(|p| p.to_string_lossy().to_string())
        .collect()
}

/// 追加 PATH 项
pub fn append_path_env(path: &str) -> String {
    let current = std::env::var_os("PATH").unwrap_or_default();
    let mut paths: Vec<_> = std::env::split_paths(&current).collect();
    paths.push(PathBuf::from(path));
    std::env::join_paths(paths)
        .unwrap_or_default()
        .to_string_lossy()
        .to_string()
}

/// 获取系统分隔符
pub fn line_separator() -> &'static str {
    if cfg!(windows) { "\r\n" } else { "\n" }
}

/// 判断是否为绝对路径
pub fn is_absolute_path(path: &str) -> bool {
    PathBuf::from(path).is_absolute()
}

/// 创建父目录
pub fn ensure_parent_dir(path: &str) -> Result<(), String> {
    let Some(parent) = PathBuf::from(path).parent().map(|p| p.to_path_buf()) else {
        return Ok(());
    };
    std::fs::create_dir_all(parent).map_err(|e| format!("创建父目录失败: {}", e))
}

/// 清理末尾路径分隔符
pub fn trim_trailing_separator(path: &str) -> String {
    let mut value = path.to_string();
    while value.ends_with('/') || value.ends_with('\\') {
        value.pop();
    }
    value
}

/// 获取进程 ID
pub fn current_pid() -> u32 {
    std::process::id()
}

/// 简单 sleep 毫秒
pub fn sleep_ms(ms: u64) {
    std::thread::sleep(std::time::Duration::from_millis(ms));
}

/// 获取当前时间戳秒
pub fn unix_timestamp_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// 获取当前时间戳毫秒
pub fn unix_timestamp_millis() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

/// 生成临时文件路径
pub fn temp_file_path(filename: &str) -> String {
    join_path(&get_temp_dir(), filename)
}

/// 生成日志文件路径
pub fn config_log_file(filename: &str) -> String {
    join_path(&get_logs_dir(), filename)
}

/// 判断字符串是否包含任一子串
pub fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| haystack.contains(needle))
}

/// 大小写不敏感包含判断
pub fn contains_ignore_case(haystack: &str, needle: &str) -> bool {
    haystack.to_lowercase().contains(&needle.to_lowercase())
}

/// 安全截断字符串
pub fn truncate_string(value: &str, max_len: usize) -> String {
    if value.chars().count() <= max_len {
        value.to_string()
    } else {
        value.chars().take(max_len).collect()
    }
}

/// 掩码显示敏感值
pub fn mask_secret(value: &str) -> String {
    if value.len() <= 8 {
        return "****".to_string();
    }
    format!("{}****{}", &value[..4], &value[value.len() - 4..])
}

/// 尝试解析 JSON 字符串
pub fn try_parse_json(content: &str) -> Option<Value> {
    serde_json::from_str(content).ok()
}

/// 将 Value 转为紧凑 JSON 文本
pub fn to_compact_json(value: &Value) -> Result<String, String> {
    serde_json::to_string(value).map_err(|e| format!("序列化 JSON 失败: {}", e))
}

/// 将 Value 转为格式化 JSON 文本
pub fn to_pretty_json(value: &Value) -> Result<String, String> {
    serde_json::to_string_pretty(value).map_err(|e| format!("序列化 JSON 失败: {}", e))
}

/// 合并两个 JSON Object（浅合并）
pub fn merge_json_object(base: &mut Value, patch: &Value) {
    if let (Some(base_obj), Some(patch_obj)) = (base.as_object_mut(), patch.as_object()) {
        for (key, value) in patch_obj {
            base_obj.insert(key.clone(), value.clone());
        }
    }
}

/// 是否为 Windows
pub fn is_windows_platform() -> bool {
    cfg!(windows)
}

/// 是否为 macOS
pub fn is_macos_platform() -> bool {
    cfg!(target_os = "macos")
}

/// 是否为 Linux
pub fn is_linux_platform() -> bool {
    cfg!(target_os = "linux")
}

/// 读取当前可执行文件目录
pub fn current_exe_dir() -> String {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|dir| dir.to_path_buf()))
        .unwrap_or_default()
        .to_string_lossy()
        .to_string()
}

/// 获取配置目录下 skills 根目录
pub fn get_global_skills_root() -> String {
    get_managed_skills_dir()
}

/// 获取 agent skills 根目录
pub fn get_agent_skills_root(agent_id: &str) -> String {
    get_agent_skills_dir(agent_id)
}

/// 构造配置下 exports 目录
pub fn get_exports_dir() -> String {
    get_sub_dir(&get_config_dir(), "exports")
}

/// 构造配置下 credentials 目录
pub fn get_credentials_dir() -> String {
    get_sub_dir(&get_config_dir(), "credentials")
}

/// 构造配置下 sessions 目录
pub fn get_sessions_root() -> String {
    get_sessions_dir()
}

/// 构造配置下 agents 目录
pub fn get_agents_root() -> String {
    get_agents_dir()
}

/// 构造配置下 logs 目录
pub fn get_logs_root() -> String {
    get_logs_dir()
}

/// 统一返回 OpenClaw 全局技能根目录
pub fn get_openclaw_skills_root() -> String {
    get_managed_skills_dir()
}

/// 统一返回 OpenClaw 主配置路径
pub fn get_openclaw_config_string() -> String {
    get_openclaw_config_path().to_string_lossy().to_string()
}

/// 统一拼接 skill 文件路径
pub fn get_skill_markdown_path(skill_dir: &str) -> String {
    join_path(skill_dir, "SKILL.md")
}

/// 简单路径相等判断（归一化分隔符后）
pub fn path_eq(left: &str, right: &str) -> bool {
    normalize_path(left) == normalize_path(right)
}

/// 读取目录名
pub fn get_dir_name(path: &str) -> Option<String> {
    PathBuf::from(path)
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
}

/// 获取 stem
pub fn get_file_stem(path: &str) -> Option<String> {
    PathBuf::from(path)
        .file_stem()
        .map(|name| name.to_string_lossy().to_string())
}

/// 判断扩展名是否匹配
pub fn has_extension(path: &str, ext: &str) -> bool {
    get_extension(path)
        .map(|value| value.eq_ignore_ascii_case(ext))
        .unwrap_or(false)
}

/// 返回空 JSON object
pub fn empty_json_object() -> Value {
    json!({})
}

/// 返回空 JSON array
pub fn empty_json_array() -> Value {
    json!([])
}

/// 返回 JSON null
pub fn json_null() -> Value {
    Value::Null
}

/// 判断 Value 是否为空对象
pub fn is_empty_object(value: &Value) -> bool {
    value.as_object().map(|obj| obj.is_empty()).unwrap_or(false)
}

/// 判断 Value 是否为空数组
pub fn is_empty_array(value: &Value) -> bool {
    value.as_array().map(|arr| arr.is_empty()).unwrap_or(false)
}

/// 克隆 Value 或返回空对象
pub fn clone_or_empty_object(value: Option<&Value>) -> Value {
    value.cloned().unwrap_or_else(|| json!({}))
}

/// 克隆 Value 或返回 Null
pub fn clone_or_null(value: Option<&Value>) -> Value {
    value.cloned().unwrap_or(Value::Null)
}

/// 读取布尔值或默认值
pub fn bool_or(value: Option<bool>, default: bool) -> bool {
    value.unwrap_or(default)
}

/// 读取字符串或默认值
pub fn string_or(value: Option<&str>, default: &str) -> String {
    value.unwrap_or(default).to_string()
}

/// 读取 Vec 或空 Vec
pub fn vec_or_empty<T>(value: Option<Vec<T>>) -> Vec<T> {
    value.unwrap_or_default()
}

/// 读取 HashMap 或空 HashMap
pub fn map_or_empty<K, V>(value: Option<std::collections::HashMap<K, V>>) -> std::collections::HashMap<K, V>
where
    K: std::cmp::Eq + std::hash::Hash,
{
    value.unwrap_or_default()
}

/// 当前目录下拼接子路径
pub fn join_current_dir(name: &str) -> String {
    join_path(&get_current_dir(), name)
}

/// 主目录下拼接子路径
pub fn join_home_dir(name: &str) -> String {
    join_path(&get_home_dir(), name)
}

/// 临时目录下拼接子路径
pub fn join_temp_dir(name: &str) -> String {
    join_path(&get_temp_dir(), name)
}

/// 配置目录下拼接子路径
pub fn join_config_dir(name: &str) -> String {
    join_path(&get_config_dir(), name)
}

/// agent 工作区下拼接子路径
pub fn join_agent_workspace_dir(agent_id: &str, name: &str) -> String {
    join_path(&get_agent_workspace_dir(agent_id), name)
}

/// agent skills 下拼接子路径
pub fn join_agent_skill_dir(agent_id: &str, name: &str) -> String {
    join_path(&get_agent_skills_dir(agent_id), name)
}

/// global skills 下拼接子路径
pub fn join_global_skill_dir(name: &str) -> String {
    join_path(&get_managed_skills_dir(), name)
}

/// 日志目录下拼接子路径
pub fn join_logs_dir(name: &str) -> String {
    join_path(&get_logs_dir(), name)
}

/// exports 目录下拼接子路径
pub fn join_exports_dir(name: &str) -> String {
    join_path(&get_exports_dir(), name)
}

/// credentials 目录下拼接子路径
pub fn join_credentials_dir(name: &str) -> String {
    join_path(&get_credentials_dir(), name)
}

/// sessions 目录下拼接子路径
pub fn join_sessions_dir(name: &str) -> String {
    join_path(&get_sessions_dir(), name)
}

/// agents 目录下拼接子路径
pub fn join_agents_dir(name: &str) -> String {
    join_path(&get_agents_dir(), name)
}

/// config 文件同级路径
pub fn join_config_file_dir(name: &str) -> String {
    get_openclaw_config_path()
        .parent()
        .map(|dir| join_path(&dir.to_string_lossy(), name))
        .unwrap_or_else(|| name.to_string())
}

/// 将 PathBuf 转字符串
pub fn pathbuf_to_string(path: &PathBuf) -> String {
    path.to_string_lossy().to_string()
}

/// 读取布尔环境变量
pub fn env_bool(key: &str) -> bool {
    std::env::var(key)
        .map(|value| matches!(value.to_lowercase().as_str(), "1" | "true" | "yes" | "on"))
        .unwrap_or(false)
}

/// 获取进程架构
pub fn current_arch() -> String {
    std::env::consts::ARCH.to_string()
}

/// 获取当前操作系统
pub fn current_os() -> String {
    std::env::consts::OS.to_string()
}

/// 获取 Rust 临时目录别名
pub fn tmp_dir() -> String {
    get_temp_dir()
}

/// 获取系统路径分隔符字符
pub fn path_separator_char() -> char {
    if cfg!(windows) { '\\' } else { '/' }
}

/// 判断是否为隐藏文件名
pub fn is_hidden_name(name: &str) -> bool {
    name.starts_with('.')
}

/// 安全拼接 base 与可选名称
pub fn join_path_optional(base: &str, name: Option<&str>) -> String {
    name.map(|value| join_path(base, value)).unwrap_or_else(|| base.to_string())
}

/// 读取 UTF-8 文件内容
pub fn read_utf8_file(path: &str) -> Result<String, String> {
    file::read_file(path).map_err(|e| format!("读取 UTF-8 文件失败: {}", e))
}

/// 写入 UTF-8 文件内容
pub fn write_utf8_file(path: &str, content: &str) -> Result<(), String> {
    file::write_file(path, content).map_err(|e| format!("写入 UTF-8 文件失败: {}", e))
}

/// 读取 JSON 文件
pub fn read_json_file(path: &str) -> Result<Value, String> {
    let content = read_utf8_file(path)?;
    serde_json::from_str(&content).map_err(|e| format!("解析 JSON 文件失败: {}", e))
}

/// 写入 JSON 文件
pub fn write_json_file(path: &str, value: &Value) -> Result<(), String> {
    let content = serde_json::to_string_pretty(value).map_err(|e| format!("序列化 JSON 失败: {}", e))?;
    write_utf8_file(path, &content)
}

/// 读取 YAML 文件
pub fn read_yaml_file(path: &str) -> Result<Value, String> {
    let content = read_utf8_file(path)?;
    serde_yaml::from_str(&content).map_err(|e| format!("解析 YAML 文件失败: {}", e))
}

/// 获取默认 shell
pub fn default_shell() -> String {
    #[cfg(windows)]
    {
        env_or_default("COMSPEC", "cmd.exe")
    }
    #[cfg(not(windows))]
    {
        env_or_default("SHELL", "/bin/sh")
    }
}

/// 判断 shell 是否可用
pub fn shell_available() -> bool {
    command_exists(&default_shell())
}

/// 获取系统名称（简短）
pub fn system_name() -> String {
    format!("{}-{}", current_os(), current_arch())
}

/// 获取配置目录 basename
pub fn config_dir_name() -> Option<String> {
    get_dir_name(&get_config_dir())
}

/// 获取工作目录 basename
pub fn current_dir_name() -> Option<String> {
    get_dir_name(&get_current_dir())
}

/// 判断路径是否在配置目录内（字符串前缀级别）
pub fn is_under_config_dir(path: &str) -> bool {
    normalize_path(path).starts_with(&normalize_path(&get_config_dir()))
}

/// 返回规范化后的配置目录
pub fn normalized_config_dir() -> String {
    normalize_path(&get_config_dir())
}

/// 返回规范化后的全局 skills 目录
pub fn normalized_global_skills_dir() -> String {
    normalize_path(&get_managed_skills_dir())
}

/// 返回规范化后的 agent skills 目录
pub fn normalized_agent_skills_dir(agent_id: &str) -> String {
    normalize_path(&get_agent_skills_dir(agent_id))
}

/// 返回规范化后的配置文件路径
pub fn normalized_config_file_path() -> String {
    normalize_path(&get_openclaw_config_path().to_string_lossy())
}

/// 获取程序数据目录
pub fn data_dir() -> String {
    get_config_dir()
}

/// 获取应用名称
pub fn app_name() -> &'static str {
    "OpenClaw"
}

/// 获取 CLI 名称
pub fn cli_name() -> &'static str {
    "openclaw"
}

/// 获取技能默认文件名
pub fn skill_filename() -> &'static str {
    "SKILL.md"
}

/// 获取配置默认文件名
pub fn config_filename() -> &'static str {
    "openclaw.json"
}

/// 获取日志默认文件名
pub fn default_log_filename() -> &'static str {
    "openclaw.log"
}

/// 获取导出目录名
pub fn exports_dir_name() -> &'static str {
    "exports"
}

/// 获取会话目录名
pub fn sessions_dir_name() -> &'static str {
    "sessions"
}

/// 获取 agents 目录名
pub fn agents_dir_name() -> &'static str {
    "agents"
}

/// 获取 skills 目录名
pub fn skills_dir_name() -> &'static str {
    "skills"
}

/// 获取 logs 目录名
pub fn logs_dir_name() -> &'static str {
    "logs"
}

/// 获取 credentials 目录名
pub fn credentials_dir_name() -> &'static str {
    "credentials"
}

/// 获取默认 workspace 前缀
pub fn workspace_prefix() -> &'static str {
    "workspace-"
}

/// 生成 workspace 名称
pub fn workspace_name(agent_id: &str) -> String {
    format!("{}{}", workspace_prefix(), agent_id)
}

/// 从 workspace 名称提取 agent id
pub fn parse_workspace_agent_id(name: &str) -> Option<String> {
    name.strip_prefix(workspace_prefix()).map(|value| value.to_string())
}

/// 返回配置目录中的 workspace 目录
pub fn get_workspace_dir(agent_id: &str) -> String {
    get_agent_workspace_dir(agent_id)
}

/// 返回配置目录中的 skill 目录
pub fn get_skill_root_dir() -> String {
    get_managed_skills_dir()
}

/// 返回配置目录中的导出目录
pub fn get_export_root_dir() -> String {
    get_exports_dir()
}

/// 返回配置目录中的日志目录
pub fn get_log_root_dir() -> String {
    get_logs_dir()
}

/// 返回配置目录中的凭证目录
pub fn get_credential_root_dir() -> String {
    get_credentials_dir()
}

/// 返回配置目录中的会话目录
pub fn get_session_root_dir() -> String {
    get_sessions_dir()
}

/// 返回配置目录中的 agents 目录
pub fn get_agent_root_dir() -> String {
    get_agents_dir()
}

/// 读取主配置 JSON
pub fn read_openclaw_config_json() -> Result<Value, String> {
    load_openclaw_config()
}

/// 写入主配置 JSON
pub fn write_openclaw_config_json(value: &Value) -> Result<(), String> {
    save_openclaw_config(value)
}

/// 检查主配置 JSON 是否为空对象
pub fn is_openclaw_config_empty() -> Result<bool, String> {
    let value = load_openclaw_config()?;
    Ok(is_empty_object(&value))
}

/// 获取配置目录路径对象
pub fn get_config_dir_pathbuf() -> PathBuf {
    PathBuf::from(get_config_dir())
}

/// 获取 skills 目录路径对象
pub fn get_skills_dir_pathbuf() -> PathBuf {
    PathBuf::from(get_managed_skills_dir())
}

/// 获取日志目录路径对象
pub fn get_logs_dir_pathbuf() -> PathBuf {
    PathBuf::from(get_logs_dir())
}

/// 获取 agents 目录路径对象
pub fn get_agents_dir_pathbuf() -> PathBuf {
    PathBuf::from(get_agents_dir())
}

/// 获取 sessions 目录路径对象
pub fn get_sessions_dir_pathbuf() -> PathBuf {
    PathBuf::from(get_sessions_dir())
}

/// 获取 credentials 目录路径对象
pub fn get_credentials_dir_pathbuf() -> PathBuf {
    PathBuf::from(get_credentials_dir())
}

/// 获取 exports 目录路径对象
pub fn get_exports_dir_pathbuf() -> PathBuf {
    PathBuf::from(get_exports_dir())
}

/// 获取 agent workspace 路径对象
pub fn get_agent_workspace_pathbuf(agent_id: &str) -> PathBuf {
    PathBuf::from(get_agent_workspace_dir(agent_id))
}

/// 获取 agent skills 路径对象
pub fn get_agent_skills_pathbuf(agent_id: &str) -> PathBuf {
    PathBuf::from(get_agent_skills_dir(agent_id))
}

/// 获取 skill 路径对象
pub fn get_skill_pathbuf(skill_id: &str, source: &str, agent_id: Option<&str>) -> PathBuf {
    PathBuf::from(get_skill_dir(skill_id, source, agent_id))
}

/// 获取日志文件路径对象
pub fn get_log_file_pathbuf(filename: &str) -> PathBuf {
    PathBuf::from(get_log_file_path(filename))
}

/// 获取配置文件路径对象
pub fn get_openclaw_config_pathbuf() -> PathBuf {
    get_openclaw_config_path()
}

/// 获取当前目录路径对象
pub fn get_current_dir_pathbuf() -> PathBuf {
    PathBuf::from(get_current_dir())
}

/// 获取主目录路径对象
pub fn get_home_dir_pathbuf() -> PathBuf {
    PathBuf::from(get_home_dir())
}

/// 获取临时目录路径对象
pub fn get_temp_dir_pathbuf() -> PathBuf {
    PathBuf::from(get_temp_dir())
}

/// 获取可执行文件目录路径对象
pub fn get_current_exe_dir_pathbuf() -> PathBuf {
    PathBuf::from(current_exe_dir())
}

/// 判断两个 PathBuf 是否相等
pub fn pathbuf_eq(left: &PathBuf, right: &PathBuf) -> bool {
    left == right
}

/// PathBuf 转规范化字符串
pub fn normalize_pathbuf(path: &PathBuf) -> String {
    normalize_path(&path.to_string_lossy())
}

/// 返回空 PathBuf
pub fn empty_pathbuf() -> PathBuf {
    PathBuf::new()
}

/// 检查 PathBuf 是否存在
pub fn pathbuf_exists(path: &PathBuf) -> bool {
    path.exists()
}

/// 检查 PathBuf 是否为文件
pub fn pathbuf_is_file(path: &PathBuf) -> bool {
    path.is_file()
}

/// 检查 PathBuf 是否为目录
pub fn pathbuf_is_dir(path: &PathBuf) -> bool {
    path.is_dir()
}

/// 连接两个 PathBuf
pub fn join_pathbuf(base: &PathBuf, name: &str) -> PathBuf {
    base.join(name)
}

/// 读取 PathBuf 文件名
pub fn pathbuf_file_name(path: &PathBuf) -> Option<String> {
    path.file_name().map(|name| name.to_string_lossy().to_string())
}

/// 读取 PathBuf 扩展名
pub fn pathbuf_extension(path: &PathBuf) -> Option<String> {
    path.extension().map(|name| name.to_string_lossy().to_string())
}

/// 读取 PathBuf 父目录
pub fn pathbuf_parent(path: &PathBuf) -> Option<String> {
    path.parent().map(|p| p.to_string_lossy().to_string())
}

/// 将字符串 Path 转 PathBuf
pub fn to_pathbuf(path: &str) -> PathBuf {
    PathBuf::from(path)
}

/// 将多个路径拼成 PathBuf
pub fn join_many_pathbuf(parts: &[&str]) -> PathBuf {
    let mut path = PathBuf::new();
    for part in parts {
        path.push(part);
    }
    path
}

/// 获取系统默认编码名称
pub fn default_encoding() -> &'static str {
    "utf-8"
}

/// 恒等函数，便于调用点统一
pub fn identity_string(value: &str) -> String {
    value.to_string()
}

/// 恒等 Value
pub fn identity_value(value: Value) -> Value {
    value
}

/// 复制字符串切片数组
pub fn clone_str_slice(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_string()).collect()
}

/// 将 Option<&str> 转 String
pub fn option_str_to_string(value: Option<&str>) -> Option<String> {
    value.map(|s| s.to_string())
}

/// 将 bool 转 yes/no
pub fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}

/// 将 bool 转 enabled/disabled
pub fn enabled_disabled(value: bool) -> &'static str {
    if value { "enabled" } else { "disabled" }
}

/// 构造简单状态字符串
pub fn build_status(label: &str, ok: bool) -> String {
    format!("{}: {}", label, yes_no(ok))
}

/// 读取文件并 trim
pub fn read_trimmed(path: &str) -> Result<String, String> {
    read_utf8_file(path).map(|content| content.trim().to_string())
}

/// 写入 trim 后文本
pub fn write_trimmed(path: &str, content: &str) -> Result<(), String> {
    write_utf8_file(path, content.trim())
}

/// 将换行统一为 LF
pub fn normalize_newlines(content: &str) -> String {
    content.replace("\r\n", "\n")
}

/// 将换行转换为当前平台格式
pub fn to_platform_newlines(content: &str) -> String {
    if cfg!(windows) {
        normalize_newlines(content).replace("\n", "\r\n")
    } else {
        normalize_newlines(content)
    }
}

/// 判断文本是否包含多行
pub fn is_multiline(content: &str) -> bool {
    content.contains('\n') || content.contains("\r\n")
}

/// 返回空字符串
pub fn empty_string() -> String {
    String::new()
}

/// 返回 true 常量
pub fn always_true() -> bool {
    true
}

/// 返回 false 常量
pub fn always_false() -> bool {
    false
}

/// 生成简单 key/value JSON
pub fn kv_json(key: &str, value: Value) -> Value {
    let mut obj = serde_json::Map::new();
    obj.insert(key.to_string(), value);
    Value::Object(obj)
}

/// 生成字符串 JSON
pub fn string_json(value: &str) -> Value {
    Value::String(value.to_string())
}

/// 生成布尔 JSON
pub fn bool_json(value: bool) -> Value {
    Value::Bool(value)
}

/// 生成数字 JSON
pub fn u64_json(value: u64) -> Value {
    Value::Number(value.into())
}

/// 判断字符串是否是 JSON object 起始
pub fn looks_like_json_object(content: &str) -> bool {
    content.trim_start().starts_with('{')
}

/// 判断字符串是否是 YAML frontmatter 起始
pub fn looks_like_frontmatter(content: &str) -> bool {
    content.trim_start().starts_with("---")
}

/// 拼接日志消息
pub fn log_message(scope: &str, message: &str) -> String {
    format!("[{}] {}", scope, message)
}

/// 生成 workspace skills 路径
pub fn workspace_skill_dir(agent_id: &str) -> String {
    get_agent_skills_dir(agent_id)
}

/// 生成 global skills 路径
pub fn global_skill_dir() -> String {
    get_managed_skills_dir()
}

/// 生成 bundled skills 路径
pub fn bundled_skill_dir() -> String {
    get_bundled_skills_dir()
}

/// 生成 openclaw config 路径
pub fn openclaw_config_path() -> String {
    get_openclaw_config_path().to_string_lossy().to_string()
}

/// 判断字符串是否相等（忽略大小写）
pub fn eq_ignore_case(left: &str, right: &str) -> bool {
    left.eq_ignore_ascii_case(right)
}

/// 返回当前平台显示名
pub fn platform_display_name() -> &'static str {
    if cfg!(windows) {
        "Windows"
    } else if cfg!(target_os = "macos") {
        "macOS"
    } else {
        "Linux"
    }
}

/// 构造配置下 `workspace-<agent>/skills/<skill>`
pub fn resolve_agent_skill_path(agent_id: &str, skill_id: &str) -> String {
    join_path(&get_agent_skills_dir(agent_id), skill_id)
}

/// 构造配置下 `skills/<skill>`
pub fn resolve_global_skill_path(skill_id: &str) -> String {
    join_path(&get_managed_skills_dir(), skill_id)
}

/// 获取配置目录中的 exports 路径对象
pub fn exports_pathbuf() -> PathBuf {
    PathBuf::from(get_exports_dir())
}

/// 获取配置目录中的 logs 路径对象
pub fn logs_pathbuf() -> PathBuf {
    PathBuf::from(get_logs_dir())
}

/// 获取配置目录中的 skills 路径对象
pub fn skills_pathbuf() -> PathBuf {
    PathBuf::from(get_managed_skills_dir())
}

/// 获取配置目录中的 agents 路径对象
pub fn agents_pathbuf() -> PathBuf {
    PathBuf::from(get_agents_dir())
}

/// 获取配置目录中的 sessions 路径对象
pub fn sessions_pathbuf() -> PathBuf {
    PathBuf::from(get_sessions_dir())
}

/// 获取配置目录中的 credentials 路径对象
pub fn credentials_pathbuf() -> PathBuf {
    PathBuf::from(get_credentials_dir())
}
