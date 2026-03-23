use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// OpenClaw 完整配置 - 对应 openclaw.json 结构
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OpenClawConfig {
    /// Agent 配置
    #[serde(default)]
    pub agents: AgentsConfig,
    /// 模型配置
    #[serde(default)]
    pub models: ModelsConfig,
    /// 网关配置
    #[serde(default)]
    pub gateway: GatewayConfig,
    /// 渠道配置
    #[serde(default)]
    pub channels: HashMap<String, serde_json::Value>,
    /// 插件配置
    #[serde(default)]
    pub plugins: PluginsConfig,
    /// 元数据
    #[serde(default)]
    pub meta: MetaConfig,
}

/// Agent 配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentsConfig {
    /// 默认配置
    #[serde(default)]
    pub defaults: AgentDefaults,
}

/// Agent 默认配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentDefaults {
    /// 模型配置
    #[serde(default)]
    pub model: AgentModelConfig,
    /// 可用模型列表 (provider/model -> {})
    #[serde(default)]
    pub models: HashMap<String, serde_json::Value>,
    /// 压缩配置
    #[serde(default)]
    pub compaction: Option<serde_json::Value>,
    /// 上下文裁剪
    #[serde(rename = "contextPruning", default)]
    pub context_pruning: Option<serde_json::Value>,
    /// 心跳配置
    #[serde(default)]
    pub heartbeat: Option<serde_json::Value>,
    /// 最大并发数
    #[serde(rename = "maxConcurrent", default)]
    pub max_concurrent: Option<u32>,
    /// 子代理配置
    #[serde(default)]
    pub subagents: Option<serde_json::Value>,
}

/// Agent 模型配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentModelConfig {
    /// 主模型 (格式: provider/model-id)
    #[serde(default)]
    pub primary: Option<String>,
}

/// 模型配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModelsConfig {
    /// Provider 配置映射
    #[serde(default)]
    pub providers: HashMap<String, ProviderConfig>,
}

/// Provider 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// API 地址
    #[serde(rename = "baseUrl")]
    pub base_url: String,
    /// API Key
    #[serde(rename = "apiKey")]
    pub api_key: Option<String>,
    /// 模型列表
    #[serde(default)]
    pub models: Vec<ModelConfig>,
}

/// 模型配置详情
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    /// 模型 ID
    pub id: String,
    /// 显示名称
    pub name: String,
    /// API 类型 (anthropic-messages / openai-completions)
    #[serde(default)]
    pub api: Option<String>,
    /// 支持的输入类型
    #[serde(default)]
    pub input: Vec<String>,
    /// 上下文窗口大小
    #[serde(rename = "contextWindow", default)]
    pub context_window: Option<u32>,
    /// 最大输出 Token
    #[serde(rename = "maxTokens", default)]
    pub max_tokens: Option<u32>,
    /// 是否支持推理模式
    #[serde(default)]
    pub reasoning: Option<bool>,
    /// 成本配置
    #[serde(default)]
    pub cost: Option<ModelCostConfig>,
}

/// 模型成本配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModelCostConfig {
    #[serde(default)]
    pub input: f64,
    #[serde(default)]
    pub output: f64,
    #[serde(rename = "cacheRead", default)]
    pub cache_read: f64,
    #[serde(rename = "cacheWrite", default)]
    pub cache_write: f64,
}

/// 网关配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GatewayConfig {
    /// 模式：local 或 cloud
    #[serde(default)]
    pub mode: Option<String>,
    /// 认证配置
    #[serde(default)]
    pub auth: Option<GatewayAuthConfig>,
}

/// 网关认证配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GatewayAuthConfig {
    #[serde(default)]
    pub mode: Option<String>,
    #[serde(default)]
    pub token: Option<String>,
}

/// 插件配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PluginsConfig {
    #[serde(default)]
    pub allow: Vec<String>,
    #[serde(default)]
    pub entries: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub installs: HashMap<String, serde_json::Value>,
}

/// 元数据配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MetaConfig {
    #[serde(rename = "lastTouchedAt", default)]
    pub last_touched_at: Option<String>,
    #[serde(rename = "lastTouchedVersion", default)]
    pub last_touched_version: Option<String>,
}

// ============ 前端展示用数据结构 ============

/// 官方 Provider 预设（用于前端展示）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfficialProvider {
    /// Provider ID (用于配置中)
    pub id: String,
    /// 显示名称
    pub name: String,
    /// 图标（emoji）
    pub icon: String,
    /// 官方 API 地址
    pub default_base_url: Option<String>,
    /// API 类型
    pub api_type: String,
    /// 推荐模型列表
    pub suggested_models: Vec<SuggestedModel>,
    /// 是否需要 API Key
    pub requires_api_key: bool,
    /// 文档链接
    pub docs_url: Option<String>,
}

/// 推荐模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestedModel {
    /// 模型 ID
    pub id: String,
    /// 显示名称
    pub name: String,
    /// 描述
    pub description: Option<String>,
    /// 上下文窗口
    pub context_window: Option<u32>,
    /// 最大输出
    pub max_tokens: Option<u32>,
    /// 是否推荐
    pub recommended: bool,
}

/// 已配置的 Provider（从配置文件读取）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfiguredProvider {
    /// Provider 名称 (配置中的 key)
    pub name: String,
    /// API 地址
    pub base_url: String,
    /// API Key (脱敏显示)
    pub api_key_masked: Option<String>,
    /// 是否有 API Key
    pub has_api_key: bool,
    /// 配置的模型列表
    pub models: Vec<ConfiguredModel>,
}

/// 已配置的模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfiguredModel {
    /// 完整模型 ID (provider/model-id)
    pub full_id: String,
    /// 模型 ID
    pub id: String,
    /// 显示名称
    pub name: String,
    /// API 类型
    pub api_type: Option<String>,
    /// 上下文窗口
    pub context_window: Option<u32>,
    /// 最大输出
    pub max_tokens: Option<u32>,
    /// 是否为主模型
    pub is_primary: bool,
}

/// AI 配置概览（返回给前端）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIConfigOverview {
    /// 主模型
    pub primary_model: Option<String>,
    /// 已配置的 Provider 列表
    pub configured_providers: Vec<ConfiguredProvider>,
    /// 可用模型列表
    pub available_models: Vec<String>,
}

// ============ 旧数据结构保持兼容 ============

/// AI Provider 选项（用于前端展示）- 旧版兼容
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIProviderOption {
    /// Provider ID
    pub id: String,
    /// 显示名称
    pub name: String,
    /// 图标（emoji）
    pub icon: String,
    /// 官方 API 地址
    pub default_base_url: Option<String>,
    /// 推荐模型列表
    pub models: Vec<AIModelOption>,
    /// 是否需要 API Key
    pub requires_api_key: bool,
}

/// AI 模型选项 - 旧版兼容
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIModelOption {
    /// 模型 ID
    pub id: String,
    /// 显示名称
    pub name: String,
    /// 描述
    pub description: Option<String>,
    /// 是否推荐
    pub recommended: bool,
}

/// 渠道配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelConfig {
    /// 渠道 ID
    pub id: String,
    /// 渠道类型
    pub channel_type: String,
    /// 是否启用
    pub enabled: bool,
    /// 配置详情
    pub config: HashMap<String, serde_json::Value>,
}

/// 环境变量配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvConfig {
    pub key: String,
    pub value: String,
}

// ============ 智能体管理数据结构 ============

/// 智能体配置项 - 对应 openclaw.json 中 agents.list 的每一项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// 智能体 ID
    pub id: String,
    /// 是否为默认智能体
    #[serde(default, skip_serializing_if = "is_false")]
    pub default: bool,
    /// 智能体名称
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// 工作区目录
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace: Option<String>,
    /// 智能体目录
    #[serde(rename = "agentDir", skip_serializing_if = "Option::is_none")]
    pub agent_dir: Option<String>,
    /// 模型配置
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<AgentModelConfigFull>,
    /// 技能白名单
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub skills: Vec<String>,
    /// 身份配置
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identity: Option<AgentIdentityConfig>,
}

fn is_false(value: &bool) -> bool {
    !*value
}

/// 智能体模型配置（完整版）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentModelConfigFull {
    /// 主模型
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary: Option<String>,
    /// 备用模型列表
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fallbacks: Vec<String>,
}

/// 智能体身份配置 - 仅包含 OpenClaw 支持的字段
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentIdentityConfig {
    /// 名称
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// 主题
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme: Option<String>,
    /// 表情符号
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emoji: Option<String>,
    /// 头像 URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<String>,
}

/// 智能体信息（返回给前端）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    /// 智能体 ID
    pub id: String,
    /// 显示名称
    pub name: String,
    /// 描述
    pub description: Option<String>,
    /// 头像（emoji 或 URL）
    pub avatar: Option<String>,
    /// 是否为默认智能体
    #[serde(rename = "isDefault")]
    pub is_default: bool,
    /// 是否为内置智能体（不可删除）
    #[serde(rename = "isBuiltin", default)]
    pub is_builtin: bool,
    /// 工作区目录
    pub workspace: Option<String>,
    /// 主模型
    pub model: Option<String>,
    /// 技能列表
    #[serde(default)]
    pub skills: Vec<String>,
    /// 创建时间
    #[serde(rename = "createdAt")]
    pub created_at: Option<String>,
    /// 更新时间
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<String>,
}

/// 智能体列表结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentsListResult {
    /// 智能体列表
    pub agents: Vec<AgentInfo>,
    /// 默认智能体 ID
    #[serde(rename = "defaultId")]
    pub default_id: Option<String>,
}

/// 创建智能体参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAgentParams {
    /// 智能体名称
    pub name: String,
    /// 工作区目录（可选，默认自动生成）
    pub workspace: Option<String>,
    /// 主模型（可选）
    pub model: Option<String>,
    /// 描述（可选）
    pub description: Option<String>,
    /// 头像/表情（可选）
    pub avatar: Option<String>,
    /// 技能列表（可选）
    #[serde(default)]
    pub skills: Vec<String>,
}

/// 更新智能体参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAgentParams {
    /// 智能体 ID
    #[serde(rename = "agentId")]
    pub agent_id: String,
    /// 名称（可选）
    pub name: Option<String>,
    /// 工作区目录（可选）
    pub workspace: Option<String>,
    /// 主模型（可选）
    pub model: Option<String>,
    /// 描述（可选）
    pub description: Option<String>,
    /// 头像/表情（可选）
    pub avatar: Option<String>,
    /// 技能列表（可选）
    pub skills: Option<Vec<String>>,
}

/// 删除智能体参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteAgentParams {
    /// 智能体 ID
    #[serde(rename = "agentId")]
    pub agent_id: String,
    /// 是否删除相关文件
    #[serde(rename = "deleteFiles", default = "default_delete_files")]
    pub delete_files: bool,
}

fn default_delete_files() -> bool {
    true
}

/// 设置默认智能体参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetDefaultAgentParams {
    /// 智能体 ID
    #[serde(rename = "agentId")]
    pub agent_id: String,
}

/// 智能体渠道关联配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentBinding {
    /// 绑定类型
    #[serde(default)]
    pub r#type: Option<String>,
    /// 智能体 ID
    pub agent_id: String,
    /// 注释
    #[serde(default)]
    pub comment: Option<String>,
    /// 匹配规则
    pub r#match: AgentBindingMatch,
}

/// 渠道绑定匹配规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentBindingMatch {
    /// 渠道类型
    pub channel: String,
    /// 账户 ID
    #[serde(default)]
    pub account_id: Option<String>,
    /// 对端类型和 ID
    #[serde(default)]
    pub peer: Option<AgentBindingPeer>,
    /// Discord 服务器 ID
    #[serde(default)]
    pub guild_id: Option<String>,
    /// Team ID
    #[serde(default)]
    pub team_id: Option<String>,
    /// Discord 角色 ID 列表
    #[serde(default)]
    pub roles: Option<Vec<String>>,
}

/// 绑定对端信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentBindingPeer {
    /// 对端类型
    pub kind: String,
    /// 对端 ID
    pub id: String,
}

/// 智能体关联的渠道信息（返回给前端）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentChannelBinding {
    /// 渠道类型
    pub channel: String,
    /// 账户 ID
    pub account_id: Option<String>,
}

/// 智能体渠道关联结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentBindingsResult {
    /// 智能体 ID
    pub agent_id: String,
    /// 渠道关联列表
    pub bindings: Vec<AgentChannelBinding>,
}

/// 设置渠道关联参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetAgentBindingsParams {
    /// 智能体 ID
    #[serde(rename = "agentId")]
    pub agent_id: String,
    /// 渠道关联列表
    pub bindings: Vec<AgentChannelBinding>,
}

// ============ 技能管理数据结构 ============

/// 技能信息（返回给前端）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillInfo {
    /// 技能 ID（唯一标识）
    pub id: String,
    /// 技能名称
    pub name: String,
    /// 技能描述
    pub description: String,
    /// 图标（emoji）
    pub emoji: Option<String>,
    /// 主页 URL
    pub homepage: Option<String>,
    /// 来源位置
    pub source: String,
    /// 是否为内置技能
    pub bundled: bool,
    /// 是否可用
    pub eligible: bool,
    /// 是否禁用
    pub disabled: bool,
    /// 标签列表
    #[serde(default)]
    pub tags: Vec<String>,
    /// 版本信息
    pub version: Option<String>,
    /// 作者信息
    pub author: Option<String>,
    /// 所需环境变量
    #[serde(default)]
    pub required_env: Vec<String>,
    /// 所需二进制文件
    #[serde(default)]
    pub required_bins: Vec<String>,
    /// 安装选项
    #[serde(default)]
    pub install_options: Vec<SkillInstallOption>,
}

/// 技能安装选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillInstallOption {
    /// 安装 ID
    pub id: String,
    /// 安装类型
    pub kind: String,
    /// 显示标签
    pub label: String,
    /// 安装后的二进制文件
    #[serde(default)]
    pub bins: Vec<String>,
}

/// 技能列表结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillsListResult {
    /// 技能列表
    pub skills: Vec<SkillInfo>,
    /// 总数
    pub total: usize,
    /// 可用数量
    pub eligible_count: usize,
}

/// 技能配置项 - 对应 openclaw.json 中 skills.entries 的每一项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillConfigEntry {
    /// 是否启用
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// API 密钥
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    /// 环境变量覆盖
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub env: std::collections::HashMap<String, String>,
    /// 其他配置
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub config: serde_json::Value,
}

fn default_true() -> bool {
    true
}

/// 技能详细配置 - 用于前端编辑
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDetail {
    /// 技能基本信息
    #[serde(flatten)]
    pub info: SkillInfo,
    /// 技能配置
    pub config: Option<SkillConfigEntry>,
    /// 技能目录路径
    pub path: String,
    /// SKILL.md 内容
    pub skill_md_content: Option<String>,
    /// 是否需要 API 密钥
    pub requires_api_key: bool,
    /// 主环境变量名（如 GEMINI_API_KEY）
    pub primary_env: Option<String>,
}

/// 创建技能参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSkillParams {
    /// 技能名称
    pub name: String,
    /// 技能描述
    pub description: String,
    /// 图标（emoji）
    pub emoji: Option<String>,
    /// 主页 URL
    pub homepage: Option<String>,
    /// 所需环境变量
    #[serde(default)]
    pub required_env: Vec<String>,
    /// 所需二进制文件
    #[serde(default)]
    pub required_bins: Vec<String>,
    /// SKILL.md 内容（可选，默认生成模板）
    pub skill_md_content: Option<String>,
    /// 安装到指定智能体（可选）
    pub agent_id: Option<String>,
}

/// 更新技能配置参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSkillConfigParams {
    /// 技能 ID
    #[serde(rename = "skillId")]
    pub skill_id: String,
    /// 是否启用
    pub enabled: Option<bool>,
    /// API 密钥
    #[serde(rename = "apiKey")]
    pub api_key: Option<String>,
    /// 环境变量
    pub env: Option<std::collections::HashMap<String, String>>,
    /// 其他配置
    pub config: Option<serde_json::Value>,
}

/// 安装技能参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallSkillParams {
    /// ZIP 文件路径
    #[serde(rename = "zipPath")]
    pub zip_path: String,
    /// 安装到指定智能体（可选，默认安装到全局）
    #[serde(rename = "agentId")]
    pub agent_id: Option<String>,
}

/// 安装技能结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallSkillResult {
    /// 是否成功
    pub success: bool,
    /// 技能 ID
    #[serde(rename = "skillId")]
    pub skill_id: Option<String>,
    /// 技能名称
    pub name: Option<String>,
    /// 错误信息
    pub error: Option<String>,
    /// 安全警告
    pub warnings: Vec<String>,
}

/// 导出技能参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportSkillParams {
    /// 技能 ID
    #[serde(rename = "skillId")]
    pub skill_id: String,
    /// 导出目录
    #[serde(rename = "outputDir")]
    pub output_dir: Option<String>,
}

/// 导出技能结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportSkillResult {
    /// 是否成功
    pub success: bool,
    /// 导出文件路径
    #[serde(rename = "outputPath")]
    pub output_path: Option<String>,
    /// 错误信息
    pub error: Option<String>,
}

/// 技能安装结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillInstallResult {
    /// 是否成功
    pub success: bool,
    /// 消息
    pub message: String,
    /// 安装选项 ID
    #[serde(rename = "installId")]
    pub install_id: Option<String>,
}
