use crate::commands::gateway::{
    create_gateway_client, set_gateway_connection, clear_gateway_connection,
    is_gateway_running, is_gateway_connected, get_session_store_path, GatewayConnectConfig,
    SERVICE_PORT,
};
use crate::utils::platform;
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use tauri::{command, AppHandle, Emitter};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSession {
    pub key: String,
    #[serde(rename = "id")]
    pub id: String,
    pub title: String,
    #[serde(rename = "agentId")]
    pub agent_id: Option<String>,
    #[serde(rename = "modelId")]
    pub model_id: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: u64,
    #[serde(rename = "updatedAt")]
    pub updated_at: u64,
    #[serde(rename = "messageCount")]
    pub message_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub role: String,
    pub content: String,
    pub timestamp: u64,
    #[serde(rename = "isStreaming")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_streaming: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachments: Option<Vec<ChatAttachment>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatAttachment {
    pub id: String,
    #[serde(rename = "type")]
    pub attachment_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GatewayAttachment {
    #[serde(rename = "type")]
    type_: String,
    mime_type: Option<String>,
    file_name: String,
    size: Option<u64>,
    content: Option<String>,
}

impl From<ChatAttachment> for GatewayAttachment {
    fn from(att: ChatAttachment) -> Self {
        GatewayAttachment {
            type_: att.attachment_type,
            mime_type: att.mime_type,
            file_name: att.name,
            size: att.size,
            content: att.content.or(att.url),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<String>,
    #[serde(rename = "runId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
    pub status: String,
}

fn get_sessions_dir() -> Result<PathBuf, String> {
    let config_dir = platform::get_config_dir();
    let sessions_dir = PathBuf::from(&config_dir)
        .join("agents")
        .join("main")
        .join("sessions");

    if !sessions_dir.exists() {
        fs::create_dir_all(&sessions_dir)
            .map_err(|e| format!("创建会话目录失败: {}", e))?;
    }

    Ok(sessions_dir)
}

fn get_session_file(session_key: &str) -> Result<PathBuf, String> {
    let sessions_dir = get_sessions_dir()?;
    let safe_key = session_key.replace("/", "-").replace("\\", "-");
    Ok(sessions_dir.join(format!("{}.jsonl", safe_key)))
}

fn extract_text_from_content(content: &serde_json::Value) -> String {
    if let Some(text) = content.as_str() {
        return text.to_string();
    }
    
    if let Some(arr) = content.as_array() {
        let texts: Vec<String> = arr
            .iter()
            .filter_map(|item| {
                if let Some(obj) = item.as_object() {
                    if obj.get("type")?.as_str()? == "text" {
                        return obj.get("text")?.as_str().map(|s| s.to_string());
                    }
                }
                None
            })
            .collect();
        return texts.join("\n");
    }
    
    content.to_string()
}

#[command]
pub async fn get_sessions() -> Result<HashMap<String, Vec<ChatSession>>, String> {
    info!("[聊天] 获取会话列表...");
    
    if !is_gateway_running() {
        warn!("[聊天] Gateway 服务未运行，返回空列表");
        let mut result = HashMap::new();
        result.insert("sessions".to_string(), Vec::new());
        return Ok(result);
    }
    
    if !is_gateway_connected().await {
        warn!("[聊天] 未连接 Gateway，返回空列表");
        let mut result = HashMap::new();
        result.insert("sessions".to_string(), Vec::new());
        return Ok(result);
    }

    let mut client = match create_gateway_client().await {
        Ok(c) => c,
        Err(e) => {
            warn!("[聊天] 创建 Gateway 客户端失败: {}，返回空列表", e);
            let mut result = HashMap::new();
            result.insert("sessions".to_string(), Vec::new());
            return Ok(result);
        }
    };
    
    let payload = client.send_request("sessions.list", serde_json::json!({
        "includeGlobal": true,
        "includeUnknown": false,
        "limit": 50
    })).await;
    
    let payload = match payload {
        Ok(p) => p,
        Err(e) => {
            warn!("[聊天] 获取会话列表失败: {}，返回空列表", e);
            let mut result = HashMap::new();
            result.insert("sessions".to_string(), Vec::new());
            return Ok(result);
        }
    };
    
    let sessions_array = payload.get("sessions")
        .and_then(|s| s.as_array())
        .cloned()
        .unwrap_or_default();
    
    let mut sessions: Vec<ChatSession> = Vec::new();
    
    for item in sessions_array {
        if let Some(obj) = item.as_object() {
            let key = obj.get("key").and_then(|k| k.as_str()).unwrap_or("").to_string();
            if key.is_empty() {
                continue;
            }
            
            // 从 Gateway 返回的对象中提取字段，兼容不同的字段名
            let session_id = obj.get("sessionId").or_else(|| obj.get("id")).and_then(|i| i.as_str()).unwrap_or(&key).to_string();
            let title = obj.get("displayName").or_else(|| obj.get("title")).and_then(|t| t.as_str()).unwrap_or(&key).to_string();
            let agent_id = obj.get("agentId").and_then(|a| a.as_str()).map(|s| s.to_string());
            let model_id = obj.get("modelId").or_else(|| obj.get("model")).and_then(|m| m.as_str()).map(|s| s.to_string());
            let created_at = obj.get("createdAt").or_else(|| obj.get("updatedAt")).and_then(|t| t.as_u64()).unwrap_or(0);
            let updated_at = obj.get("updatedAt").and_then(|t| t.as_u64()).unwrap_or(created_at);
            let message_count = obj.get("messageCount").and_then(|c| c.as_u64()).unwrap_or(0) as u32;
            
            let session = ChatSession {
                key,
                id: session_id,
                title,
                agent_id,
                model_id,
                created_at,
                updated_at,
                message_count,
            };
            sessions.push(session);
        }
    }
    
    sessions.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    
    info!("[聊天] ✓ 返回 {} 个会话", sessions.len());
    
    let mut result = HashMap::new();
    result.insert("sessions".to_string(), sessions);
    Ok(result)
}

#[command]
pub async fn get_session_messages(session_key: String) -> Result<HashMap<String, Vec<ChatMessage>>, String> {
    info!("[聊天] 获取会话消息: {}", session_key);
    
    if !is_gateway_running() {
        warn!("[聊天] Gateway 服务未运行，返回空列表");
        let mut result = HashMap::new();
        result.insert("messages".to_string(), Vec::new());
        return Ok(result);
    }
    
    if !is_gateway_connected().await {
        warn!("[聊天] 未连接 Gateway，返回空列表");
        let mut result = HashMap::new();
        result.insert("messages".to_string(), Vec::new());
        return Ok(result);
    }

    let mut client = create_gateway_client().await?;
    
    let payload = client.send_request("chat.history", serde_json::json!({
        "sessionKey": session_key
    })).await?;
    
    let messages_array = payload.get("messages")
        .and_then(|m| m.as_array())
        .cloned()
        .unwrap_or_default();
    
    let mut messages: Vec<ChatMessage> = Vec::new();
    
    for item in messages_array {
        if let Some(obj) = item.as_object() {
            let role = obj.get("role").and_then(|r| r.as_str()).unwrap_or("").to_string();
            let content = obj.get("content");
            let content_text = extract_text_from_content(content.unwrap_or(&serde_json::json!("")));
            let timestamp = obj.get("timestamp").and_then(|t| t.as_u64())
                .or_else(|| obj.get("timestampMs").and_then(|t| t.as_u64()))
                .unwrap_or(0);
            
            let message = ChatMessage {
                id: obj.get("id").and_then(|i| i.as_str()).unwrap_or(&format!("msg-{}", timestamp)).to_string(),
                role,
                content: content_text,
                timestamp,
                is_streaming: None,
                thinking: None,
                attachments: None,
            };
            messages.push(message);
        }
    }
    
    info!("[聊天] ✓ 返回 {} 条消息", messages.len());
    
    let mut result = HashMap::new();
    result.insert("messages".to_string(), messages);
    Ok(result)
}

#[command]
pub async fn connect_gateway(config: GatewayConnectConfig) -> Result<HashMap<String, bool>, String> {
    info!("[聊天] 连接 Gateway: {}", config.url);
    
    if !is_gateway_running() {
        warn!("[聊天] Gateway 服务未运行 (端口 {} 未监听)", SERVICE_PORT);
        let mut result = HashMap::new();
        result.insert("success".to_string(), false);
        return Ok(result);
    }
    
    set_gateway_connection(config.url.clone(), config.token.clone()).await;
    
    info!("[聊天] ✓ Gateway 连接配置已保存");
    
    let mut result = HashMap::new();
    result.insert("success".to_string(), true);
    Ok(result)
}

#[command]
pub async fn disconnect_gateway() -> Result<(), String> {
    info!("[聊天] 断开 Gateway 连接");
    
    clear_gateway_connection().await;
    
    Ok(())
}

#[command]
pub async fn create_session(
    agent_id: Option<String>,
    model_id: Option<String>,
) -> Result<HashMap<String, ChatSession>, String> {
    info!("[聊天] 创建新会话: agent={:?}, model={:?}", agent_id, model_id);
    
    if !is_gateway_running() {
        warn!("[聊天] Gateway 服务未运行，使用本地创建");
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let session_key = format!("session-{}", timestamp);
        
        let session = ChatSession {
                key: session_key.clone(),
                id: session_key.clone(),
                title: "新会话".to_string(),
                agent_id,
                model_id,
                created_at: timestamp,
                updated_at: timestamp,
                message_count: 0,
            };
        
        let mut result = HashMap::new();
        result.insert("session".to_string(), session);
        return Ok(result);
    }
    
    if !is_gateway_connected().await {
        warn!("[聊天] 未连接 Gateway，使用本地创建");
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let session_key = format!("session-{}", timestamp);
        
        let session = ChatSession {
            key: session_key.clone(),
            id: session_key.clone(),
            title: "新会话".to_string(),
            agent_id,
            model_id,
            created_at: timestamp,
            updated_at: timestamp,
            message_count: 0,
        };
        
        let mut result = HashMap::new();
        result.insert("session".to_string(), session);
        return Ok(result);
    }

    let mut client = create_gateway_client().await?;

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    let agent_id_str = agent_id.clone().unwrap_or_else(|| "main".to_string());
    let session_key_format = format!("agent:{}:session-{}", agent_id_str, timestamp);

    let create_params = serde_json::json!({
        "agentId": agent_id_str,
        "model": model_id.clone(),
        "key": session_key_format,
    });

    info!("[聊天] 发送 sessions.create 请求");

    let payload = client.send_request("sessions.create", create_params).await?;

    let entry = payload.get("entry").cloned().unwrap_or(serde_json::json!({}));

    let session_key = entry.get("key").and_then(|k| k.as_str()).unwrap_or(&format!("session-{}", timestamp)).to_string();

    let session = ChatSession {
        key: session_key.clone(),
        id: entry.get("sessionId").and_then(|i| i.as_str()).unwrap_or(&session_key).to_string(),
        title: entry.get("title").and_then(|t| t.as_str()).unwrap_or("新会话").to_string(),
        agent_id: entry.get("agentId").and_then(|a| a.as_str().map(|s| s.to_string())).or(agent_id),
        model_id: entry.get("model").and_then(|m| m.as_str().map(|s| s.to_string())).or(model_id),
        created_at: entry.get("createdAt").and_then(|t| t.as_u64()).unwrap_or(timestamp / 1000),
        updated_at: entry.get("updatedAt").and_then(|t| t.as_u64()).unwrap_or(timestamp / 1000),
        message_count: entry.get("messageCount").and_then(|c| c.as_u64()).unwrap_or(0) as u32,
    };

    info!("[聊天] ✓ 会话创建成功: {}", session.key);

    let mut result = HashMap::new();
    result.insert("session".to_string(), session);
    Ok(result)
}

#[command]
pub async fn delete_session(session_key: String) -> Result<(), String> {
    info!("[聊天] 删除会话: {}", session_key);

    let mut deleted_via_gateway = false;
    let mut gateway_error = String::new();

    if is_gateway_running() && is_gateway_connected().await {
        match delete_session_via_gateway(session_key.clone()).await {
            Ok(_) => {
                info!("[聊天] ✓ 通过 Gateway 删除会话成功: {}", session_key);
                deleted_via_gateway = true;
            }
            Err(e) => {
                gateway_error = e;
                warn!("[聊天] 通过 Gateway 删除失败: {}，将尝试本地删除", gateway_error);
            }
        }
    }

    delete_session_locally(&session_key)?;
    info!("[聊天] ✓ 本地删除会话成功: {}", session_key);

    if !deleted_via_gateway && !gateway_error.is_empty() {
        warn!("[聊天] Gateway删除失败原因: {}", gateway_error);
    }

    Ok(())
}

async fn delete_session_via_gateway(session_key: String) -> Result<(), String> {
    let mut client = create_gateway_client().await?;
    
    client.send_request("sessions.delete", serde_json::json!({
        "key": session_key,
        "deleteTranscript": true
    })).await?;
    
    Ok(())
}

fn delete_session_locally(session_key: &str) -> Result<(), String> {
    let store_path = get_session_store_path();
    let mut deleted_count = 0;

    if store_path.exists() {
        let content = fs::read_to_string(&store_path)
            .map_err(|e| format!("读取 sessions.json 失败: {}", e))?;

        let mut store: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| format!("解析 sessions.json 失败: {}", e))?;

        let sessions = match store.as_object_mut() {
            Some(obj) => obj,
            None => return Err("sessions.json 格式无效".to_string()),
        };

        let keys_to_remove: Vec<String> = sessions.keys()
            .filter(|k| {
                k.to_lowercase() == session_key.to_lowercase() ||
                k.starts_with(&format!("{}:", session_key)) ||
                session_key.starts_with(&format!("{}:", k))
            })
            .cloned()
            .collect();

        deleted_count = keys_to_remove.len();
        for key in &keys_to_remove {
            sessions.remove(key);
        }

        if !keys_to_remove.is_empty() {
            let new_content = serde_json::to_string_pretty(&store)
                .map_err(|e| format!("序列化 sessions.json 失败: {}", e))?;

            fs::write(&store_path, new_content)
                .map_err(|e| format!("写入 sessions.json 失败: {}", e))?;
        }
    }

    let session_file = get_session_file(session_key)?;
    if session_file.exists() {
        fs::remove_file(&session_file)
            .map_err(|e| format!("删除会话脚本文件失败: {}", e))?;
    }

    let sessions_dir = get_sessions_dir()?;
    if let Ok(entries) = fs::read_dir(&sessions_dir) {
        let session_id = session_key.split(':').last().unwrap_or(session_key);
        for entry in entries.flatten() {
            let file_name = entry.file_name();
            if let Some(name) = file_name.to_str() {
                if name.starts_with(session_id) && (name.ends_with(".jsonl") || name.ends_with(".jsonl.archived")) {
                    let _ = fs::remove_file(entry.path());
                }
            }
        }
    }

    if let Err(e) = crate::commands::cron::cleanup_session_cron_jobs(session_key) {
        warn!("[聊天] 清理关联定时任务失败: {}", e);
    }

    info!("[聊天] 本地删除会话完成: {} ({} 条记录)", session_key, deleted_count);
    Ok(())
}

#[command]
pub async fn send_chat_message(
    app: AppHandle,
    session_key: Option<String>,
    message: String,
    attachments: Option<Vec<ChatAttachment>>,
    agent_id: Option<String>,
    model_id: Option<String>,
) -> Result<ChatResponse, String> {
    info!(
        "[聊天] 发送消息: session={:?}, agent={:?}, model={:?}",
        session_key, agent_id, model_id
    );
    
    debug!("[聊天] 消息内容: {}", message);
    if let Some(ref atts) = attachments {
        debug!("[聊天] 附件数量: {}", atts.len());
    }
    
    if !is_gateway_running() {
        error!("[聊天] Gateway 服务未运行");
        return Err("Gateway 服务未运行，请先启动服务".to_string());
    }
    
    if !is_gateway_connected().await {
        error!("[聊天] 未连接到 Gateway");
        return Err("未连接到 Gateway，请先配置并连接".to_string());
    }
    
    let session = session_key.unwrap_or_else(|| "main".to_string());
    let run_id = uuid::Uuid::new_v4().to_string();

    let mut client = create_gateway_client().await?;
    
    let mut params = serde_json::json!({
        "sessionKey": session,
        "message": message,
        "idempotencyKey": run_id,
        "timeoutMs": 30000
    });
    
    if let Some(atts) = attachments {
        if !atts.is_empty() {
            let gateway_atts: Vec<GatewayAttachment> = atts.into_iter().map(|a| a.into()).collect();
            params["attachments"] = serde_json::to_value(gateway_atts)
                .map_err(|e| format!("序列化附件失败: {}", e))?;
        }
    }
    
    let payload = client.send_request("chat.send", params).await?;
    
    let actual_run_id = payload.get("runId").and_then(|r| r.as_str()).map(|s| s.to_string());
    
    let mut assistant_text = String::new();
    let mut final_received = false;
    
    while !final_received {
        let event_opt = tokio::time::timeout(
            Duration::from_secs(120),
            client.read_event()
        )
        .await
        .map_err(|_| "等待事件超时".to_string())??;
        
        let event = match event_opt {
            Some(e) => e,
            None => continue,
        };
        
        if event["type"] == "event" && event["event"] == "chat" {
            let event_payload = &event["payload"];
            let state = event_payload.get("state").and_then(|s| s.as_str()).unwrap_or("");
            
            match state {
                "thinking_delta" => {
                    let thinking_text = event_payload.get("thinking")
                        .and_then(|t: &serde_json::Value| t.as_str())
                        .unwrap_or("");
                    
                    let _ = app.emit("chat-event", serde_json::json!({
                        "state": "thinking_delta",
                        "thinking": thinking_text
                    }));
                }
                "delta" => {
                    let mut delta_text = String::new();
                    if let Some(msg) = event_payload.get("message") {
                        if let Some(content) = msg.get("content").and_then(|c: &serde_json::Value| c.as_array()) {
                            for item in content {
                                if let Some(text) = item.get("text").and_then(|t: &serde_json::Value| t.as_str()) {
                                    delta_text.push_str(text);
                                    assistant_text.push_str(text);
                                }
                            }
                        }
                    }
                    
                    let _ = app.emit("chat-event", serde_json::json!({
                        "state": "delta",
                        "message": {
                            "content": delta_text
                        }
                    }));
                }
                "final" | "error" => {
                    final_received = true;
                    
                    if state == "error" {
                        let error_msg = event_payload.get("errorMessage")
                            .and_then(|m: &serde_json::Value| m.as_str())
                            .unwrap_or("聊天失败")
                            .to_string();
                        
                        let _ = app.emit("chat-event", serde_json::json!({
                            "state": "error",
                            "errorMessage": error_msg
                        }));
                        
                        return Err(error_msg);
                    }
                    
                    if assistant_text.is_empty() {
                        if let Some(msg) = event_payload.get("message") {
                            if let Some(content) = msg.get("content").and_then(|c: &serde_json::Value| c.as_array()) {
                                for item in content {
                                    if let Some(text) = item.get("text").and_then(|t: &serde_json::Value| t.as_str()) {
                                        assistant_text.push_str(text);
                                    }
                                }
                            }
                        }
                    }
                    
                    let _ = app.emit("chat-event", serde_json::json!({
                        "state": "final",
                        "message": {
                            "content": assistant_text
                        }
                    }));
                }
                _ => {}
            }
        }
    }
    
    info!("[聊天] ✓ 消息已处理: runId={:?}", actual_run_id);
    
    Ok(ChatResponse {
        content: assistant_text,
        thinking: None,
        run_id: actual_run_id.or(Some(run_id)),
        status: "completed".to_string(),
    })
}

#[command]
pub async fn check_gateway_status() -> Result<HashMap<String, bool>, String> {
    info!("[聊天] 检查 Gateway 状态...");

    let is_running = is_gateway_running();
    let is_connected = is_gateway_connected().await;

    info!("[聊天] Gateway 状态: running={}, connected={}", is_running, is_connected);

    let mut result = HashMap::new();
    result.insert("running".to_string(), is_running);
    result.insert("connected".to_string(), is_connected);
    Ok(result)
}

#[command]
pub async fn patch_session(
    session_key: String,
    model_id: Option<String>,
    label: Option<String>,
) -> Result<HashMap<String, ChatSession>, String> {
    info!("[聊天] 修改会话: key={}, model={:?}, label={:?}", session_key, model_id, label);

    if !is_gateway_running() {
        warn!("[聊天] Gateway 服务未运行，无法修改会话");
        return Err("Gateway 服务未运行".to_string());
    }

    if !is_gateway_connected().await {
        warn!("[聊天] 未连接到 Gateway");
        return Err("未连接到 Gateway".to_string());
    }

    let mut client = create_gateway_client().await?;

    let mut patch_params = serde_json::json!({
        "key": session_key
    });

    if let Some(ref model) = model_id {
        patch_params["model"] = serde_json::json!(model);
    }

    if let Some(ref lbl) = label {
        patch_params["label"] = serde_json::json!(lbl);
    }

    info!("[聊天] 发送 sessions.patch 请求");

    let payload = client.send_request("sessions.patch", patch_params).await?;

    let entry = payload.get("entry").cloned().unwrap_or(serde_json::json!({}));

    let resolved = payload.get("resolved").cloned();

    let resolved_model = resolved
        .and_then(|r| r.get("model").and_then(|m| m.as_str().map(|s| s.to_string())));

    let session = ChatSession {
        key: entry.get("key").and_then(|k| k.as_str()).unwrap_or(&session_key).to_string(),
        id: entry.get("sessionId").and_then(|i| i.as_str()).unwrap_or(&session_key).to_string(),
        title: entry.get("title").and_then(|t| t.as_str()).unwrap_or("会话").to_string(),
        agent_id: entry.get("agentId").and_then(|a| a.as_str().map(|s| s.to_string())),
        model_id: resolved_model.or_else(|| entry.get("model").and_then(|m| m.as_str().map(|s| s.to_string()))),
        created_at: entry.get("createdAt").and_then(|t| t.as_u64()).unwrap_or(0),
        updated_at: entry.get("updatedAt").and_then(|t| t.as_u64()).unwrap_or(0),
        message_count: entry.get("messageCount").and_then(|c| c.as_u64()).unwrap_or(0) as u32,
    };

    info!("[聊天] ✓ 会话修改成功: {}", session.key);

    let mut result = HashMap::new();
    result.insert("session".to_string(), session);
    Ok(result)
}
