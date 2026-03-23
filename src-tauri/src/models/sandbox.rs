use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 沙箱模式
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SandboxMode {
    /// 不使用沙箱
    #[default]
    Off,
    /// 仅沙箱非主会话
    #[serde(rename = "non-main")]
    NonMain,
    /// 所有会话都在沙箱中运行
    All,
}

/// 沙箱作用域
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SandboxScope {
    /// 每个会话一个容器
    #[default]
    Session,
    /// 每个智能体一个容器
    Agent,
    /// 所有会话共享一个容器
    Shared,
}

/// 工作区访问权限
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SandboxWorkspaceAccess {
    /// 工具看到沙箱工作区
    #[default]
    None,
    /// 只读挂载智能体工作区到 /agent
    #[serde(rename = "ro")]
    ReadOnly,
    /// 读写挂载智能体工作区到 /workspace
    #[serde(rename = "rw")]
    ReadWrite,
}

/// Docker 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxDockerConfig {
    /// Docker 镜像
    #[serde(default = "default_sandbox_image")]
    pub image: String,
    /// 容器名前缀
    #[serde(default = "default_container_prefix")]
    pub container_prefix: String,
    /// 容器工作目录
    #[serde(default = "default_workdir")]
    pub workdir: String,
    /// 只读根文件系统
    #[serde(default = "default_true")]
    pub read_only_root: bool,
    /// tmpfs 挂载点
    #[serde(default = "default_tmpfs")]
    pub tmpfs: Vec<String>,
    /// 网络模式
    #[serde(default)]
    pub network: Option<String>,
    /// 运行用户 (uid:gid)
    #[serde(default)]
    pub user: Option<String>,
    /// 丢弃的 Linux 能力
    #[serde(default = "default_cap_drop")]
    pub cap_drop: Vec<String>,
    /// 环境变量
    #[serde(default)]
    pub env: HashMap<String, String>,
    /// 初始化命令
    #[serde(default)]
    pub setup_command: Option<String>,
    /// PID 限制
    #[serde(default)]
    pub pids_limit: Option<u32>,
    /// 内存限制
    #[serde(default)]
    pub memory: Option<String>,
    /// 交换内存限制
    #[serde(default)]
    pub memory_swap: Option<String>,
    /// CPU 限制
    #[serde(default)]
    pub cpus: Option<f64>,
    /// ulimit 设置
    #[serde(default)]
    pub ulimits: HashMap<String, UlimitConfig>,
    /// seccomp 配置文件
    #[serde(default)]
    pub seccomp_profile: Option<String>,
    /// AppArmor 配置文件
    #[serde(default)]
    pub apparmor_profile: Option<String>,
    /// DNS 服务器
    #[serde(default)]
    pub dns: Vec<String>,
    /// 额外主机映射
    #[serde(default)]
    pub extra_hosts: Vec<String>,
    /// 绑定挂载
    #[serde(default)]
    pub binds: Vec<String>,
    /// 危险选项：允许保留容器目标路径
    #[serde(default)]
    pub dangerously_allow_reserved_container_targets: bool,
    /// 危险选项：允许外部绑定源
    #[serde(default)]
    pub dangerously_allow_external_bind_sources: bool,
    /// 危险选项：允许容器命名空间加入
    #[serde(default)]
    pub dangerously_allow_container_namespace_join: bool,
}

fn default_sandbox_image() -> String {
    "openclaw-sandbox:bookworm-slim".to_string()
}

fn default_container_prefix() -> String {
    "openclaw-sbx-".to_string()
}

fn default_workdir() -> String {
    "/workspace".to_string()
}

fn default_true() -> bool {
    true
}

fn default_tmpfs() -> Vec<String> {
    vec![
        "/tmp".to_string(),
        "/run".to_string(),
        "/var/run".to_string(),
    ]
}

fn default_cap_drop() -> Vec<String> {
    vec!["ALL".to_string()]
}

impl Default for SandboxDockerConfig {
    fn default() -> Self {
        Self {
            image: default_sandbox_image(),
            container_prefix: default_container_prefix(),
            workdir: default_workdir(),
            read_only_root: true,
            tmpfs: default_tmpfs(),
            network: Some("none".to_string()),
            user: None,
            cap_drop: default_cap_drop(),
            env: HashMap::new(),
            setup_command: None,
            pids_limit: Some(256),
            memory: Some("512m".to_string()),
            memory_swap: None,
            cpus: Some(1.0),
            ulimits: HashMap::new(),
            seccomp_profile: None,
            apparmor_profile: None,
            dns: vec![],
            extra_hosts: vec![],
            binds: vec![],
            dangerously_allow_reserved_container_targets: false,
            dangerously_allow_external_bind_sources: false,
            dangerously_allow_container_namespace_join: false,
        }
    }
}

/// ulimit 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UlimitConfig {
    /// 软限制
    #[serde(default)]
    pub soft: Option<u64>,
    /// 硬限制
    #[serde(default)]
    pub hard: Option<u64>,
}

/// 沙箱浏览器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxBrowserConfig {
    /// 是否启用浏览器
    #[serde(default)]
    pub enabled: bool,
    /// 浏览器镜像
    #[serde(default = "default_browser_image")]
    pub image: String,
    /// 容器名前缀
    #[serde(default = "default_browser_prefix")]
    pub container_prefix: String,
    /// 网络模式
    #[serde(default = "default_browser_network")]
    pub network: String,
    /// CDP 端口
    #[serde(default = "default_cdp_port")]
    pub cdp_port: u16,
    /// CDP 来源范围
    #[serde(default)]
    pub cdp_source_range: Option<String>,
    /// VNC 端口
    #[serde(default = "default_vnc_port")]
    pub vnc_port: u16,
    /// noVNC 端口
    #[serde(default = "default_novnc_port")]
    pub no_vnc_port: u16,
    /// 无头模式
    #[serde(default = "default_headless")]
    pub headless: bool,
    /// 启用 noVNC
    #[serde(default)]
    pub enable_no_vnc: bool,
    /// 允许主机控制
    #[serde(default)]
    pub allow_host_control: bool,
    /// 自动启动
    #[serde(default = "default_true")]
    pub auto_start: bool,
    /// 自动启动超时 (ms)
    #[serde(default = "default_autostart_timeout")]
    pub auto_start_timeout_ms: u32,
    /// 额外绑定挂载
    #[serde(default)]
    pub binds: Vec<String>,
}

fn default_browser_image() -> String {
    "openclaw-sandbox-browser:bookworm-slim".to_string()
}

fn default_browser_prefix() -> String {
    "openclaw-sbx-browser-".to_string()
}

fn default_browser_network() -> String {
    "openclaw-sandbox-browser".to_string()
}

fn default_cdp_port() -> u16 {
    9222
}

fn default_vnc_port() -> u16 {
    5900
}

fn default_novnc_port() -> u16 {
    6080
}

fn default_headless() -> bool {
    true
}

fn default_autostart_timeout() -> u32 {
    12000
}

impl Default for SandboxBrowserConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            image: default_browser_image(),
            container_prefix: default_browser_prefix(),
            network: default_browser_network(),
            cdp_port: default_cdp_port(),
            cdp_source_range: None,
            vnc_port: default_vnc_port(),
            no_vnc_port: default_novnc_port(),
            headless: true,
            enable_no_vnc: false,
            allow_host_control: false,
            auto_start: true,
            auto_start_timeout_ms: default_autostart_timeout(),
            binds: vec![],
        }
    }
}

/// 沙箱清理配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxPruneConfig {
    /// 空闲小时数后清理
    #[serde(default = "default_idle_hours")]
    pub idle_hours: u32,
    /// 最大存活天数
    #[serde(default = "default_max_age_days")]
    pub max_age_days: u32,
}

fn default_idle_hours() -> u32 {
    24
}

fn default_max_age_days() -> u32 {
    7
}

impl Default for SandboxPruneConfig {
    fn default() -> Self {
        Self {
            idle_hours: default_idle_hours(),
            max_age_days: default_max_age_days(),
        }
    }
}

/// 沙箱工具策略
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SandboxToolPolicy {
    /// 允许的工具列表
    #[serde(default)]
    pub allow: Vec<String>,
    /// 拒绝的工具列表
    #[serde(default)]
    pub deny: Vec<String>,
}

/// 沙箱完整配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    /// 沙箱模式
    #[serde(default)]
    pub mode: SandboxMode,
    /// 容器作用域
    #[serde(default)]
    pub scope: SandboxScope,
    /// 工作区访问权限
    #[serde(default)]
    pub workspace_access: SandboxWorkspaceAccess,
    /// 工作区根目录
    #[serde(default)]
    pub workspace_root: Option<String>,
    /// Docker 配置
    #[serde(default)]
    pub docker: SandboxDockerConfig,
    /// 浏览器配置
    #[serde(default)]
    pub browser: SandboxBrowserConfig,
    /// 工具策略
    #[serde(default)]
    pub tools: SandboxToolPolicy,
    /// 清理配置
    #[serde(default)]
    pub prune: SandboxPruneConfig,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            mode: SandboxMode::default(),
            scope: SandboxScope::default(),
            workspace_access: SandboxWorkspaceAccess::default(),
            workspace_root: None,
            docker: SandboxDockerConfig::default(),
            browser: SandboxBrowserConfig::default(),
            tools: SandboxToolPolicy::default(),
            prune: SandboxPruneConfig::default(),
        }
    }
}

/// 沙箱容器信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxContainerInfo {
    /// 容器名称
    pub name: String,
    /// 会话密钥
    pub session_key: String,
    /// 镜像
    pub image: String,
    /// 状态
    pub status: String,
    /// 创建时间
    pub created_at: String,
    /// 最后使用时间
    pub last_used_at: String,
    /// 配置哈希
    pub config_hash: Option<String>,
}

/// 沙箱状态概览
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxStatus {
    /// 是否启用沙箱
    pub enabled: bool,
    /// 沙箱模式
    pub mode: SandboxMode,
    /// Docker 是否可用
    pub docker_available: bool,
    /// Docker 版本
    pub docker_version: Option<String>,
    /// 运行中的容器数量
    pub running_containers: u32,
    /// 容器列表
    pub containers: Vec<SandboxContainerInfo>,
}

/// 沙箱安全验证结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxSecurityValidation {
    /// 是否通过
    pub valid: bool,
    /// 错误信息
    pub errors: Vec<String>,
    /// 警告信息
    pub warnings: Vec<String>,
}

/// 被阻止的主机路径
pub const BLOCKED_HOST_PATHS: &[&str] = &[
    "/etc",
    "/private/etc",
    "/proc",
    "/sys",
    "/dev",
    "/root",
    "/boot",
    "/run",
    "/var/run",
    "/private/var/run",
    "/var/run/docker.sock",
    "/private/var/run/docker.sock",
    "/run/docker.sock",
];

/// 被阻止的环境变量模式
pub const BLOCKED_ENV_PATTERNS: &[&str] = &[
    "ANTHROPIC_API_KEY",
    "OPENAI_API_KEY",
    "GEMINI_API_KEY",
    "OPENROUTER_API_KEY",
    "MINIMAX_API_KEY",
    "ELEVENLABS_API_KEY",
    "TELEGRAM_BOT_TOKEN",
    "DISCORD_BOT_TOKEN",
    "SLACK_BOT_TOKEN",
    "SLACK_APP_TOKEN",
    "LINE_CHANNEL_SECRET",
    "LINE_CHANNEL_ACCESS_TOKEN",
    "AWS_SECRET_ACCESS_KEY",
    "AWS_SECRET_KEY",
    "AWS_SESSION_TOKEN",
    "GH_TOKEN",
    "GITHUB_TOKEN",
];

/// 默认允许的工具
pub const DEFAULT_TOOL_ALLOW: &[&str] = &[
    "exec",
    "process",
    "read",
    "write",
    "edit",
    "apply_patch",
    "image",
    "sessions_list",
    "sessions_history",
    "sessions_send",
    "sessions_spawn",
    "sessions_yield",
    "subagents",
    "session_status",
];

/// 默认拒绝的工具
pub const DEFAULT_TOOL_DENY: &[&str] = &[
    "browser",
    "canvas",
    "nodes",
    "cron",
    "gateway",
];
