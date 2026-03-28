use crate::commands::common::{get_config_dir, join_path};
use crate::commands::gateway::{get_or_create_gateway_client, is_gateway_running, SERVICE_PORT};
use crate::utils::text::now_ms;
use crate::models::{
    CreateCronJobParams, CronJob, CronJobRunResult, CronJobsListResult, CronSchedule,
    CronStats, CronStoreFile, CronValidateResult, UpdateCronJobParams, CronRunLogEntry, CronRunLogResult,
    CronRunLogFile, CronRunLogFilesResult,
};
use crate::utils::file;
use log::{info, warn};
use serde_json::json;
use std::path::Path;
use tauri::command;
use uuid::Uuid;

const CRON_DIR: &str = "cron";
const CRON_JOBS_FILENAME: &str = "jobs.json";
const CRON_RUNS_DIR: &str = "runs";

fn get_cron_store_path() -> String {
    let config_dir = get_config_dir();
    join_path(&join_path(&config_dir, CRON_DIR), CRON_JOBS_FILENAME)
}

fn load_cron_store() -> Result<CronStoreFile, String> {
    let path = get_cron_store_path();
    if !Path::new(&path).exists() {
        return Ok(CronStoreFile {
            version: 1,
            jobs: Vec::new(),
        });
    }
    let content = file::read_file(&path).map_err(|e| format!("读取定时任务存储失败: {}", e))?;
    serde_json::from_str(&content).map_err(|e| format!("解析定时任务存储失败: {}", e))
}

fn save_cron_store(store: &CronStoreFile) -> Result<(), String> {
    let path = get_cron_store_path();
    let content =
        serde_json::to_string_pretty(store).map_err(|e| format!("序列化定时任务存储失败: {}", e))?;
    file::write_file(&path, &content).map_err(|e| format!("写入定时任务存储失败: {}", e))
}

fn generate_job_id() -> String {
    format!("cron-{}", Uuid::new_v4().to_string().split('-').next().unwrap_or("unknown"))
}

fn compute_next_run_at_ms(schedule: &CronSchedule, now_ms: u64) -> Option<u64> {
    match schedule {
        CronSchedule::At { at } => {
            if let Ok(at_ms) = chrono::DateTime::parse_from_rfc3339(at) {
                let at_timestamp = at_ms.timestamp_millis() as u64;
                if at_timestamp > now_ms {
                    return Some(at_timestamp);
                }
            }
            if let Ok(at_dt) = chrono::NaiveDateTime::parse_from_str(at, "%Y-%m-%d %H:%M:%S") {
                let at_timestamp = at_dt.and_utc().timestamp_millis() as u64;
                if at_timestamp > now_ms {
                    return Some(at_timestamp);
                }
            }
            None
        }
        CronSchedule::Every { every_ms, anchor_ms, .. } => {
            let every = every_ms.max(&1);
            let anchor = anchor_ms.unwrap_or(now_ms);
            if now_ms < anchor {
                return Some(anchor);
            }
            let elapsed = now_ms - anchor;
            let steps = ((elapsed / every) + 1) * every;
            Some(anchor + steps)
        }
        CronSchedule::Cron { expr, tz, .. } => {
            let cron_expr = expr.trim();
            if cron_expr.is_empty() {
                return None;
            }
            match cron::Schedule::try_from(cron_expr) {
                Ok(schedule) => {
                    let timezone: chrono_tz::Tz = tz
                        .as_ref()
                        .and_then(|t| t.parse().ok())
                        .unwrap_or(chrono_tz::UTC);
                    let now = chrono::Utc::now().with_timezone(&timezone);
                    if let Some(next) = schedule.after(&now).next() {
                        return Some(next.timestamp_millis() as u64);
                    }
                }
                Err(_) => {}
            }
            None
        }
    }
}

#[command]
pub async fn get_cron_jobs() -> Result<CronJobsListResult, String> {
    info!("[Cron] Getting cron jobs list...");

    let store = load_cron_store()?;
    let total = store.jobs.len();
    let enabled_count = store.jobs.iter().filter(|j| j.enabled).count();
    let running_count = store.jobs.iter().filter(|j| j.state.running_at_ms.is_some()).count();

    info!("[Cron] Returning {} jobs, {} enabled, {} running", total, enabled_count, running_count);

    Ok(CronJobsListResult {
        jobs: store.jobs,
        total,
        enabled_count,
        running_count,
    })
}

#[command]
pub async fn get_cron_job(job_id: String) -> Result<CronJob, String> {
    info!("[Cron] Getting cron job: {}", job_id);

    let store = load_cron_store()?;
    store
        .jobs
        .into_iter()
        .find(|j| j.id == job_id)
        .ok_or_else(|| format!("定时任务 '{}' 不存在", job_id))
}

#[command]
pub async fn create_cron_job(params: CreateCronJobParams) -> Result<CronJob, String> {
    info!("[Cron] Creating cron job: {}", params.name);

    let mut store = load_cron_store()?;

    let job_id = generate_job_id();
    let now = now_ms();
    let next_run_at_ms = compute_next_run_at_ms(&params.schedule, now);

    let job = CronJob {
        id: job_id.clone(),
        name: params.name,
        enabled: params.enabled.unwrap_or(true),
        schedule: params.schedule,
        agent_id: params.agent_id,
        session_target: params.session_target.unwrap_or_else(|| "main".to_string()),
        wake_mode: params.wake_mode.unwrap_or_else(|| "next-heartbeat".to_string()),
        payload: params.payload,
        delivery: params.delivery,
        failure_alert: params.failure_alert,
        delete_after_run: params.delete_after_run.unwrap_or(false),
        session_key: params.session_key,
        state: crate::models::CronJobState {
            next_run_at_ms,
            ..Default::default()
        },
        created_at_ms: now,
        updated_at_ms: now,
    };

    store.jobs.push(job.clone());
    save_cron_store(&store)?;

    info!("[Cron] Created cron job: {}", job_id);
    Ok(job)
}

#[command]
pub async fn update_cron_job(params: UpdateCronJobParams) -> Result<CronJob, String> {
    info!("[Cron] Updating cron job: {}", params.job_id);

    let mut store = load_cron_store()?;
    let job_index = store
        .jobs
        .iter()
        .position(|j| j.id == params.job_id)
        .ok_or_else(|| format!("定时任务 '{}' 不存在", params.job_id))?;

    let job = &mut store.jobs[job_index];
    let now = now_ms();

    if let Some(name) = params.name {
        job.name = name;
    }
    if let Some(enabled) = params.enabled {
        job.enabled = enabled;
    }
    if let Some(schedule) = params.schedule {
        job.schedule = schedule;
    }
    if let Some(ref payload) = params.payload {
        job.payload = payload.clone();
    }
    if let Some(ref agent_id) = params.agent_id {
        job.agent_id = Some(agent_id.clone());
    }
    if let Some(ref session_target) = params.session_target {
        job.session_target = session_target.clone();
    }
    if let Some(ref wake_mode) = params.wake_mode {
        job.wake_mode = wake_mode.clone();
    }
    if let Some(ref delivery) = params.delivery {
        job.delivery = Some(delivery.clone());
    }
    if let Some(ref failure_alert) = params.failure_alert {
        job.failure_alert = Some(failure_alert.clone());
    }
    if let Some(delete_after_run) = params.delete_after_run {
        job.delete_after_run = delete_after_run;
    }
    if let Some(ref session_key) = params.session_key {
        job.session_key = Some(session_key.clone());
    }

    job.updated_at_ms = now;
    job.state.next_run_at_ms = compute_next_run_at_ms(&job.schedule, now);

    let updated_job = job.clone();
    save_cron_store(&store)?;

    info!("[Cron] Updated cron job: {}", params.job_id);
    Ok(updated_job)
}

#[command]
pub async fn delete_cron_job(job_id: String) -> Result<String, String> {
    info!("[Cron] Deleting cron job: {}", job_id);

    let mut store = load_cron_store()?;
    let initial_len = store.jobs.len();
    store.jobs.retain(|j| j.id != job_id);

    if store.jobs.len() == initial_len {
        return Err(format!("定时任务 '{}' 不存在", job_id));
    }

    save_cron_store(&store)?;

    info!("[Cron] Deleted cron job: {}", job_id);
    Ok(format!("定时任务 '{}' 已删除", job_id))
}

#[command]
pub async fn toggle_cron_job(job_id: String, enabled: bool) -> Result<CronJob, String> {
    info!("[Cron] Toggling cron job {}: enabled={}", job_id, enabled);

    let mut store = load_cron_store()?;
    let job = store
        .jobs
        .iter_mut()
        .find(|j| j.id == job_id)
        .ok_or_else(|| format!("定时任务 '{}' 不存在", job_id))?;

    job.enabled = enabled;
    job.updated_at_ms = now_ms();

    if enabled {
        job.state.next_run_at_ms = compute_next_run_at_ms(&job.schedule, now_ms());
    } else {
        job.state.next_run_at_ms = None;
    }

    let updated_job = job.clone();
    save_cron_store(&store)?;

    info!("[Cron] Toggled cron job: {}", job_id);
    Ok(updated_job)
}

#[command]
pub async fn run_cron_job(job_id: String) -> Result<CronJobRunResult, String> {
    info!("[Cron] Running cron job via Gateway API: {}", job_id);

    if !is_gateway_running() {
        warn!("[Cron] Gateway 服务未运行 (端口 {} 未监听)", SERVICE_PORT);
        return Err("Gateway 服务未运行，请先启动 OpenClaw 服务".to_string());
    }

    // 获取或创建 Gateway 客户端（未连接时会尝试自动连接一次）
    let mut client = get_or_create_gateway_client().await?;
    
    // 发送 cron.run 请求，将任务加入队列
    info!("[Cron] 发送 cron.run 请求...");
    let run_payload = client.send_request("cron.run", json!({
        "id": job_id,
        "mode": "force"
    })).await?;
    
    // 检查是否成功加入队列
    let enqueued = run_payload.get("enqueued")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    
    let run_id = run_payload.get("runId")
        .and_then(|v| v.as_str().map(|s| s.to_string()));
    
    if enqueued {
        info!("[Cron] 任务已加入队列，runId: {:?}", run_id);
        return Ok(CronJobRunResult {
            job_id: job_id.clone(),
            status: "enqueued".to_string(),
            error: None,
            summary: Some("已加入任务运行队列，运行结果请查看执行日志".to_string()),
            session_id: run_id,
            duration_ms: None,
        });
    }
    
    // 如果未成功加入队列
    let error_msg = run_payload.get("error")
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| "未知错误".to_string());
    
    warn!("[Cron] 任务加入队列失败: {}", error_msg);
    Ok(CronJobRunResult {
        job_id: job_id.clone(),
        status: "error".to_string(),
        error: Some(error_msg),
        summary: None,
        session_id: run_id,
        duration_ms: None,
    })
}

#[command]
pub async fn get_cron_stats() -> Result<CronStats, String> {
    info!("[Cron] Getting cron stats...");

    let store = load_cron_store()?;
    let now = now_ms();
    let day_ago = now - 24 * 60 * 60 * 1000;

    let total = store.jobs.len();
    let enabled = store.jobs.iter().filter(|j| j.enabled).count();
    let disabled = total - enabled;
    let running = store.jobs.iter().filter(|j| j.state.running_at_ms.is_some()).count();
    let pending = store.jobs.iter().filter(|j| {
        j.enabled && j.state.next_run_at_ms.map(|n| n > now).unwrap_or(false)
    }).count();

    let recent_success = store.jobs.iter().filter(|j| {
        j.state.last_run_at_ms.map(|t| t > day_ago && j.state.last_run_status.as_deref() == Some("ok")).unwrap_or(false)
    }).count();

    let recent_error = store.jobs.iter().filter(|j| {
        j.state.last_run_at_ms.map(|t| t > day_ago && j.state.last_run_status.as_deref() == Some("error")).unwrap_or(false)
    }).count();

    let runs_last_24h = store.jobs.iter()
        .filter_map(|j| j.state.last_run_at_ms)
        .filter(|&t| t > day_ago)
        .count();

    let stats = CronStats {
        total,
        enabled,
        disabled,
        running,
        pending,
        recent_success,
        recent_error,
        runs_last_24h,
    };

    info!("[Cron] Stats: {:?}", stats);
    Ok(stats)
}

#[command]
pub async fn validate_cron_expression(expr: String) -> Result<CronValidateResult, String> {
    info!("[Cron] Validating cron expression: {}", expr);

    let expr = expr.trim();
    if expr.is_empty() {
        return Ok(CronValidateResult {
            valid: false,
            error: Some("Cron 表达式不能为空".to_string()),
            next_runs: vec![],
            description: None,
        });
    }

    match cron::Schedule::try_from(expr) {
        Ok(schedule) => {
            let now = chrono::Utc::now();
            let next_runs: Vec<u64> = schedule
                .after(&now)
                .take(5)
                .map(|dt| dt.timestamp_millis() as u64)
                .collect();

            let description = describe_cron_expression(expr);

            Ok(CronValidateResult {
                valid: true,
                error: None,
                next_runs,
                description: Some(description),
            })
        }
        Err(e) => Ok(CronValidateResult {
            valid: false,
            error: Some(format!("无效的 Cron 表达式: {}", e)),
            next_runs: vec![],
            description: None,
        }),
    }
}

fn describe_cron_expression(expr: &str) -> String {
    let parts: Vec<&str> = expr.split_whitespace().collect();
    if parts.len() < 5 {
        return expr.to_string();
    }

    let (sec, min, hour, dom, month, dow) = if parts.len() == 6 {
        (parts[0], parts[1], parts[2], parts[3], parts[4], parts[5])
    } else {
        ("0", parts[0], parts[1], parts[2], parts[3], parts[4])
    };

    let mut desc_parts = Vec::new();

    if min == "*" {
        desc_parts.push("每分钟".to_string());
    } else if min.contains('/') {
        desc_parts.push(format!("每 {} 分钟", min.split('/').nth(1).unwrap_or(min)));
    } else if min != "0" {
        desc_parts.push(format!("第 {} 分钟", min));
    }

    if hour == "*" {
        if min != "*" {
            desc_parts.push("每小时".to_string());
        }
    } else if hour.contains('/') {
        desc_parts.push(format!("每 {} 小时", hour.split('/').nth(1).unwrap_or(hour)));
    } else if hour != "0" {
        desc_parts.push(format!("{} 点", hour));
    }

    if dom != "*" && dom != "?" {
        desc_parts.push(format!("每月 {} 号", dom));
    }

    if dow != "*" && dow != "?" {
        let day_name = match dow {
            "0" | "7" => "周日",
            "1" => "周一",
            "2" => "周二",
            "3" => "周三",
            "4" => "周四",
            "5" => "周五",
            "6" => "周六",
            _ => dow,
        };
        desc_parts.push(format!("每{}", day_name));
    }

    if desc_parts.is_empty() {
        format!("Cron: {}", expr)
    } else {
        desc_parts.join(", ")
    }
}

#[command]
pub async fn duplicate_cron_job(job_id: String) -> Result<CronJob, String> {
    info!("[Cron] Duplicating cron job: {}", job_id);

    let mut store = load_cron_store()?;
    let original = store
        .jobs
        .iter()
        .find(|j| j.id == job_id)
        .ok_or_else(|| format!("定时任务 '{}' 不存在", job_id))?;

    let new_id = generate_job_id();
    let now = now_ms();

    let mut new_job = original.clone();
    new_job.id = new_id.clone();
    new_job.name = format!("{} (副本)", original.name);
    new_job.created_at_ms = now;
    new_job.updated_at_ms = now;
    new_job.state = crate::models::CronJobState {
        next_run_at_ms: compute_next_run_at_ms(&new_job.schedule, now),
        ..Default::default()
    };

    store.jobs.push(new_job.clone());
    save_cron_store(&store)?;

    info!("[Cron] Duplicated cron job: {} -> {}", job_id, new_id);
    Ok(new_job)
}

#[command]
pub async fn get_cron_job_history(job_id: String, limit: Option<usize>) -> Result<Vec<serde_json::Value>, String> {
    info!("[Cron] Getting history for job: {}", job_id);

    let store = load_cron_store()?;
    let job = store
        .jobs
        .iter()
        .find(|j| j.id == job_id)
        .ok_or_else(|| format!("定时任务 '{}' 不存在", job_id))?;

    let mut history = Vec::new();

    if let Some(last_run) = job.state.last_run_at_ms {
        history.push(json!({
            "timestamp": last_run,
            "status": job.state.last_run_status,
            "error": job.state.last_error,
            "durationMs": job.state.last_duration_ms,
        }));
    }

    let limit = limit.unwrap_or(10);
    history.truncate(limit);

    Ok(history)
}

#[command]
pub async fn import_cron_jobs(jobs_json: String) -> Result<CronJobsListResult, String> {
    info!("[Cron] Importing cron jobs...");

    let imported: Vec<CronJob> = serde_json::from_str(&jobs_json)
        .map_err(|e| format!("解析导入数据失败: {}", e))?;

    let mut store = load_cron_store()?;
    let now = now_ms();

    for mut job in imported {
        job.id = generate_job_id();
        job.created_at_ms = now;
        job.updated_at_ms = now;
        job.state.next_run_at_ms = compute_next_run_at_ms(&job.schedule, now);
        store.jobs.push(job);
    }

    let total = store.jobs.len();
    let enabled_count = store.jobs.iter().filter(|j| j.enabled).count();
    let running_count = store.jobs.iter().filter(|j| j.state.running_at_ms.is_some()).count();

    save_cron_store(&store)?;

    info!("[Cron] Imported {} jobs", total);
    Ok(CronJobsListResult {
        jobs: store.jobs,
        total,
        enabled_count,
        running_count,
    })
}

#[command]
pub async fn export_cron_jobs() -> Result<String, String> {
    info!("[Cron] Exporting cron jobs...");

    let store = load_cron_store()?;
    let json = serde_json::to_string_pretty(&store.jobs)
        .map_err(|e| format!("导出失败: {}", e))?;

    info!("[Cron] Exported {} jobs", store.jobs.len());
    Ok(json)
}

pub fn cleanup_session_cron_jobs(session_key: &str) -> Result<usize, String> {
    let mut store = load_cron_store()?;
    let initial_len = store.jobs.len();

    store.jobs.retain(|j| j.session_key.as_ref() != Some(&session_key.to_string()));

    let removed_count = initial_len - store.jobs.len();

    if removed_count > 0 {
        save_cron_store(&store)?;
        info!("[Cron] 已清理关联到会话 {} 的 {} 个定时任务", session_key, removed_count);
    }

    Ok(removed_count)
}

// 获取 OpenClaw 执行日志目录路径 (~/.openclaw/cron/runs)
fn get_openclaw_cron_runs_dir() -> String {
    join_path(&join_path(&get_config_dir(), CRON_DIR), CRON_RUNS_DIR)
}

// 获取指定任务的执行日志文件路径
fn get_openclaw_cron_run_log_path(job_id: &str) -> String {
    info!("[Cron] 获取 OpenClaw 执行日志文件路径 for job: {}", job_id);
    let runs_dir = get_openclaw_cron_runs_dir();
    let path = join_path(&runs_dir, &format!("{}.jsonl", job_id));
    info!("[Cron] OpenClaw 执行日志文件路径: {}", path);
    path
}

// 读取 OpenClaw 执行日志
fn read_openclaw_run_logs(job_id: &str, limit: usize) -> Result<Vec<CronRunLogEntry>, String> {
    let path = get_openclaw_cron_run_log_path(job_id);
    
    if !Path::new(&path).exists() {
        return Ok(Vec::new());
    }
    
    let content = file::read_file(&path).map_err(|e| format!("读取日志文件失败: {}", e))?;
    let mut entries = Vec::new();
    
    for line in content.lines() {
        if line.trim().is_empty() {
            continue;
        }
        
        if let Ok(entry) = serde_json::from_str::<CronRunLogEntry>(line) {
            entries.push(entry);
        }
    }
    
    // 按时间倒序排列，并限制数量
    entries.sort_by(|a, b| b.ts.cmp(&a.ts));
    entries.truncate(limit);
    
    Ok(entries)
}

#[command]
pub async fn get_cron_run_logs(job_id: String, limit: Option<usize>) -> Result<CronRunLogResult, String> {
    let limit = limit.unwrap_or(50).min(200);
    
    info!("[Cron] Reading OpenClaw run logs for job: {}, limit: {}", job_id, limit);
    
    let entries = read_openclaw_run_logs(&job_id, limit)?;
    let total = entries.len();
    
    Ok(CronRunLogResult {
        entries,
        total,
        has_more: false,
    })
}

/// 获取 OpenClaw 执行日志目录下所有 .jsonl 文件列表
#[command]
pub async fn get_cron_run_log_files() -> Result<CronRunLogFilesResult, String> {
    info!("[Cron] Getting OpenClaw run log files list");
    
    let runs_dir = get_openclaw_cron_runs_dir();
    let runs_path = Path::new(&runs_dir);
    
    if !runs_path.exists() {
        return Ok(CronRunLogFilesResult {
            files: Vec::new(),
            total: 0,
        });
    }
    
    let mut files = Vec::new();
    
    let entries = std::fs::read_dir(runs_path)
        .map_err(|e| format!("读取日志目录失败: {}", e))?;
    
    for entry in entries {
        if let Ok(entry) = entry {
            let path = entry.path();
            
            // 只处理 .jsonl 文件
            if path.extension().map(|e| e == "jsonl").unwrap_or(false) {
                if let Some(name) = path.file_stem().and_then(|n| n.to_str()) {
                    let metadata = entry.metadata().ok();
                    let size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
                    let modified_ms = metadata
                        .and_then(|m| m.modified().ok())
                        .map(|t| {
                            t.duration_since(std::time::UNIX_EPOCH)
                                .map(|d| d.as_millis() as u64)
                                .unwrap_or(0)
                        })
                        .unwrap_or(0);
                    
                    // 计算日志条目数量
                    let entry_count = count_jsonl_entries(path.to_str().unwrap_or(""));
                    
                    files.push(CronRunLogFile {
                        name: name.to_string(),
                        path: path.to_str().unwrap_or("").to_string(),
                        size,
                        modified_ms,
                        entry_count,
                    });
                }
            }
        }
    }
    
    // 按修改时间倒序排列
    files.sort_by(|a, b| b.modified_ms.cmp(&a.modified_ms));
    
    let total = files.len();
    
    info!("[Cron] Found {} run log files", total);
    
    Ok(CronRunLogFilesResult {
        files,
        total,
    })
}

/// 计算 .jsonl 文件的条目数量
fn count_jsonl_entries(path: &str) -> usize {
    if path.is_empty() {
        return 0;
    }
    
    let content = match file::read_file(path) {
        Ok(c) => c,
        Err(_) => return 0,
    };
    
    content.lines().filter(|l| !l.trim().is_empty()).count()
}

/// 读取指定 .jsonl 文件的所有内容
#[command]
pub async fn get_cron_run_log_file_content(file_name: String) -> Result<CronRunLogResult, String> {
    info!("[Cron] Reading run log file content: {}", file_name);
    
    let runs_dir = get_openclaw_cron_runs_dir();
    let file_path = join_path(&runs_dir, &format!("{}.jsonl", file_name));
    
    if !Path::new(&file_path).exists() {
        return Err(format!("日志文件 '{}' 不存在", file_name));
    }
    
    let content = file::read_file(&file_path)
        .map_err(|e| format!("读取日志文件失败: {}", e))?;
    
    let mut entries = Vec::new();
    
    for line in content.lines() {
        if line.trim().is_empty() {
            continue;
        }
        
        if let Ok(entry) = serde_json::from_str::<CronRunLogEntry>(line) {
            entries.push(entry);
        }
    }
    
    // 按时间倒序排列
    entries.sort_by(|a, b| b.ts.cmp(&a.ts));
    
    let total = entries.len();
    
    info!("[Cron] Read {} entries from file: {}", total, file_name);
    
    Ok(CronRunLogResult {
        entries,
        total,
        has_more: false,
    })
}
