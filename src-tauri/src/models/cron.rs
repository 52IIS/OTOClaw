use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 定时任务调度类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum CronSchedule {
    /// 指定时间执行一次
    #[serde(rename = "at")]
    At { at: String },
    /// 间隔执行
    #[serde(rename = "every")]
    Every {
        #[serde(rename = "everyMs")]
        every_ms: u64,
        #[serde(rename = "anchorMs", skip_serializing_if = "Option::is_none")]
        anchor_ms: Option<u64>,
    },
    /// Cron 表达式
    #[serde(rename = "cron")]
    Cron {
        expr: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        tz: Option<String>,
        #[serde(rename = "staggerMs", skip_serializing_if = "Option::is_none")]
        stagger_ms: Option<u64>,
    },
}

/// 任务负载类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum CronPayload {
    /// 系统事件
    #[serde(rename = "systemEvent")]
    SystemEvent { text: String },
    /// Agent 轮次
    #[serde(rename = "agentTurn")]
    AgentTurn {
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        model: Option<String>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        fallbacks: Vec<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        thinking: Option<String>,
        #[serde(rename = "timeoutSeconds", skip_serializing_if = "Option::is_none")]
        timeout_seconds: Option<u32>,
        #[serde(rename = "allowUnsafeExternalContent", skip_serializing_if = "Option::is_none")]
        allow_unsafe_external_content: Option<bool>,
        #[serde(rename = "lightContext", skip_serializing_if = "Option::is_none")]
        light_context: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        deliver: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        channel: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        to: Option<String>,
        #[serde(rename = "bestEffortDeliver", skip_serializing_if = "Option::is_none")]
        best_effort_deliver: Option<bool>,
    },
}

/// 任务投递配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CronDelivery {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
    #[serde(rename = "accountId", skip_serializing_if = "Option::is_none")]
    pub account_id: Option<String>,
    #[serde(rename = "bestEffort", skip_serializing_if = "Option::is_none")]
    pub best_effort: Option<bool>,
    #[serde(rename = "failureDestination", skip_serializing_if = "Option::is_none")]
    pub failure_destination: Option<CronFailureDestination>,
}

/// 失败通知目标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronFailureDestination {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
    #[serde(rename = "accountId", skip_serializing_if = "Option::is_none")]
    pub account_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
}

/// 失败告警配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronFailureAlert {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
    #[serde(rename = "cooldownMs", skip_serializing_if = "Option::is_none")]
    pub cooldown_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    #[serde(rename = "accountId", skip_serializing_if = "Option::is_none")]
    pub account_id: Option<String>,
}

/// 任务运行状态
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CronJobState {
    #[serde(rename = "nextRunAtMs", skip_serializing_if = "Option::is_none")]
    pub next_run_at_ms: Option<u64>,
    #[serde(rename = "runningAtMs", skip_serializing_if = "Option::is_none")]
    pub running_at_ms: Option<u64>,
    #[serde(rename = "lastRunAtMs", skip_serializing_if = "Option::is_none")]
    pub last_run_at_ms: Option<u64>,
    #[serde(rename = "lastRunStatus", skip_serializing_if = "Option::is_none")]
    pub last_run_status: Option<String>,
    #[serde(rename = "lastStatus", skip_serializing_if = "Option::is_none")]
    pub last_status: Option<String>,
    #[serde(rename = "lastError", skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
    #[serde(rename = "lastErrorReason", skip_serializing_if = "Option::is_none")]
    pub last_error_reason: Option<String>,
    #[serde(rename = "lastDurationMs", skip_serializing_if = "Option::is_none")]
    pub last_duration_ms: Option<u64>,
    #[serde(rename = "consecutiveErrors", skip_serializing_if = "Option::is_none")]
    pub consecutive_errors: Option<u32>,
    #[serde(rename = "lastFailureAlertAtMs", skip_serializing_if = "Option::is_none")]
    pub last_failure_alert_at_ms: Option<u64>,
    #[serde(rename = "scheduleErrorCount", skip_serializing_if = "Option::is_none")]
    pub schedule_error_count: Option<u32>,
    #[serde(rename = "lastDeliveryStatus", skip_serializing_if = "Option::is_none")]
    pub last_delivery_status: Option<String>,
    #[serde(rename = "lastDeliveryError", skip_serializing_if = "Option::is_none")]
    pub last_delivery_error: Option<String>,
    #[serde(rename = "lastDelivered", skip_serializing_if = "Option::is_none")]
    pub last_delivered: Option<bool>,
}

/// 定时任务定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronJob {
    /// 任务 ID
    pub id: String,
    /// 任务名称
    pub name: String,
    /// 是否启用
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// 调度配置
    pub schedule: CronSchedule,
    /// Agent ID
    #[serde(rename = "agentId", skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    /// 会话目标
    #[serde(rename = "sessionTarget", default = "default_session_target")]
    pub session_target: String,
    /// 唤醒模式
    #[serde(rename = "wakeMode", default = "default_wake_mode")]
    pub wake_mode: String,
    /// 任务负载
    pub payload: CronPayload,
    /// 投递配置
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delivery: Option<CronDelivery>,
    /// 失败告警
    #[serde(rename = "failureAlert", skip_serializing_if = "Option::is_none")]
    pub failure_alert: Option<CronFailureAlert>,
    /// 执行后删除
    #[serde(rename = "deleteAfterRun", default)]
    pub delete_after_run: bool,
    /// 会话键
    #[serde(rename = "sessionKey", skip_serializing_if = "Option::is_none")]
    pub session_key: Option<String>,
    /// 任务状态
    #[serde(default)]
    pub state: CronJobState,
    /// 创建时间
    #[serde(rename = "createdAtMs")]
    pub created_at_ms: u64,
    /// 更新时间
    #[serde(rename = "updatedAtMs")]
    pub updated_at_ms: u64,
}

fn default_session_target() -> String {
    "main".to_string()
}

fn default_wake_mode() -> String {
    "next-heartbeat".to_string()
}

fn default_true() -> bool {
    true
}

/// 定时任务存储文件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronStoreFile {
    pub version: u32,
    pub jobs: Vec<CronJob>,
}

/// 创建定时任务参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCronJobParams {
    /// 任务名称
    pub name: String,
    /// 调度配置
    pub schedule: CronSchedule,
    /// 任务负载
    pub payload: CronPayload,
    /// Agent ID（可选）
    #[serde(rename = "agentId", skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    /// 会话目标
    #[serde(rename = "sessionTarget", skip_serializing_if = "Option::is_none")]
    pub session_target: Option<String>,
    /// 唤醒模式
    #[serde(rename = "wakeMode", skip_serializing_if = "Option::is_none")]
    pub wake_mode: Option<String>,
    /// 投递配置
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delivery: Option<CronDelivery>,
    /// 失败告警
    #[serde(rename = "failureAlert", skip_serializing_if = "Option::is_none")]
    pub failure_alert: Option<CronFailureAlert>,
    /// 执行后删除
    #[serde(rename = "deleteAfterRun", skip_serializing_if = "Option::is_none")]
    pub delete_after_run: Option<bool>,
    /// 会话键
    #[serde(rename = "sessionKey", skip_serializing_if = "Option::is_none")]
    pub session_key: Option<String>,
    /// 是否启用
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
}

/// 更新定时任务参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCronJobParams {
    /// 任务 ID
    #[serde(rename = "jobId")]
    pub job_id: String,
    /// 任务名称
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// 是否启用
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    /// 调度配置
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schedule: Option<CronSchedule>,
    /// 任务负载
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<CronPayload>,
    /// Agent ID
    #[serde(rename = "agentId", skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    /// 会话目标
    #[serde(rename = "sessionTarget", skip_serializing_if = "Option::is_none")]
    pub session_target: Option<String>,
    /// 唤醒模式
    #[serde(rename = "wakeMode", skip_serializing_if = "Option::is_none")]
    pub wake_mode: Option<String>,
    /// 投递配置
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delivery: Option<CronDelivery>,
    /// 失败告警
    #[serde(rename = "failureAlert", skip_serializing_if = "Option::is_none")]
    pub failure_alert: Option<CronFailureAlert>,
    /// 执行后删除
    #[serde(rename = "deleteAfterRun", skip_serializing_if = "Option::is_none")]
    pub delete_after_run: Option<bool>,
    /// 会话键
    #[serde(rename = "sessionKey", skip_serializing_if = "Option::is_none")]
    pub session_key: Option<String>,
}

/// 定时任务列表结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronJobsListResult {
    /// 任务列表
    pub jobs: Vec<CronJob>,
    /// 总数
    pub total: usize,
    /// 启用数量
    #[serde(rename = "enabledCount")]
    pub enabled_count: usize,
    /// 运行中数量
    #[serde(rename = "runningCount")]
    pub running_count: usize,
}

/// 定时任务执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronJobRunResult {
    /// 任务 ID
    #[serde(rename = "jobId")]
    pub job_id: String,
    /// 执行状态
    pub status: String,
    /// 错误信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// 执行摘要
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    /// 会话 ID
    #[serde(rename = "sessionId", skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// 执行时长（毫秒）
    #[serde(rename = "durationMs", skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
}

/// 定时任务统计信息
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CronStats {
    /// 总任务数
    pub total: usize,
    /// 启用任务数
    pub enabled: usize,
    /// 禁用任务数
    pub disabled: usize,
    /// 运行中任务数
    pub running: usize,
    /// 待执行任务数
    pub pending: usize,
    /// 最近成功数
    #[serde(rename = "recentSuccess")]
    pub recent_success: usize,
    /// 最近失败数
    #[serde(rename = "recentError")]
    pub recent_error: usize,
    /// 最近 24 小时执行次数
    #[serde(rename = "runsLast24h")]
    pub runs_last_24h: usize,
}

/// Cron 表达式验证结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronValidateResult {
    /// 是否有效
    pub valid: bool,
    /// 错误信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// 下次执行时间列表（预览）
    #[serde(rename = "nextRuns", default)]
    pub next_runs: Vec<u64>,
    /// 描述文本
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// 定时任务执行日志条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronRunLogEntry {
    /// 时间戳
    pub ts: u64,
    /// 任务 ID
    #[serde(rename = "jobId")]
    pub job_id: String,
    /// 任务名称
    #[serde(rename = "jobName", skip_serializing_if = "Option::is_none")]
    pub job_name: Option<String>,
    /// 动作类型
    pub action: String,
    /// 执行状态
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// 错误信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// 执行摘要
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    /// 执行时间（毫秒）
    #[serde(rename = "runAtMs", skip_serializing_if = "Option::is_none")]
    pub run_at_ms: Option<u64>,
    /// 执行时长（毫秒）
    #[serde(rename = "durationMs", skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
    /// 下次执行时间（毫秒）
    #[serde(rename = "nextRunAtMs", skip_serializing_if = "Option::is_none")]
    pub next_run_at_ms: Option<u64>,
    /// 投递状态
    #[serde(rename = "deliveryStatus", skip_serializing_if = "Option::is_none")]
    pub delivery_status: Option<String>,
    /// 会话 ID
    #[serde(rename = "sessionId", skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// 会话 Key
    #[serde(rename = "sessionKey", skip_serializing_if = "Option::is_none")]
    pub session_key: Option<String>,
    /// 模型
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// 提供商
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
}

/// 定时任务执行日志查询结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronRunLogResult {
    /// 日志条目列表
    pub entries: Vec<CronRunLogEntry>,
    /// 总数
    pub total: usize,
    /// 是否有更多
    #[serde(rename = "hasMore")]
    pub has_more: bool,
}

/// 执行日志文件信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronRunLogFile {
    /// 文件名（不含扩展名，即 job_id）
    pub name: String,
    /// 文件完整路径
    pub path: String,
    /// 文件大小（字节）
    pub size: u64,
    /// 修改时间（毫秒时间戳）
    #[serde(rename = "modifiedMs")]
    pub modified_ms: u64,
    /// 日志条目数量
    #[serde(rename = "entryCount")]
    pub entry_count: usize,
}

/// 执行日志文件列表结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronRunLogFilesResult {
    /// 文件列表
    pub files: Vec<CronRunLogFile>,
    /// 总数
    pub total: usize,
}
