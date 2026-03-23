use crate::utils::platform;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use ed25519_dalek::{Signer, SigningKey, VerifyingKey};
use futures_util::{SinkExt, StreamExt};
use log::{debug, error, info, warn};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tauri::command;
use tokio::sync::Mutex;
use tokio_tungstenite::{connect_async, tungstenite::Message as WsMessage};
use uuid::Uuid;

const SERVICE_PORT: u16 = 18789;
const GATEWAY_PROTOCOL_VERSION: u32 = 3;

const CLIENT_ID: &str = "cli";
const CLIENT_MODE: &str = "cli";

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
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayConnectConfig {
    pub url: String,
    pub token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WsRequest {
    #[serde(rename = "type")]
    msg_type: String,
    id: String,
    method: String,
    params: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WsResponse {
    #[serde(rename = "type")]
    msg_type: String,
    #[serde(default)]
    id: Option<String>,
    ok: Option<bool>,
    payload: Option<serde_json::Value>,
    error: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WsEvent {
    #[serde(rename = "type")]
    msg_type: String,
    event: String,
    payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ConnectChallenge {
    nonce: String,
    #[serde(rename = "ts")]
    ts: u64,
}

#[derive(Debug, Clone)]
struct DeviceIdentity {
    device_id: String,
    signing_key: SigningKey,
    verifying_key: VerifyingKey,
}

lazy_static::lazy_static! {
    static ref GATEWAY_CONNECTION: Arc<Mutex<Option<GatewayConnection>>> = Arc::new(Mutex::new(None));
    static ref DEVICE_IDENTITY: Arc<Mutex<Option<DeviceIdentity>>> = Arc::new(Mutex::new(None));
}

struct GatewayConnection {
    url: String,
    token: String,
}

fn get_sessions_dir() -> Result<PathBuf, String> {
    let config_dir = platform::get_config_dir();
    let sessions_dir = PathBuf::from(&config_dir).join("sessions");
    
    if !sessions_dir.exists() {
        fs::create_dir_all(&sessions_dir)
            .map_err(|e| format!("创建会话目录失败: {}", e))?;
    }
    
    Ok(sessions_dir)
}

fn get_session_file(session_key: &str) -> Result<PathBuf, String> {
    let sessions_dir = get_sessions_dir()?;
    Ok(sessions_dir.join(format!("{}.json", session_key)))
}

fn get_identity_path() -> PathBuf {
    let config_dir = platform::get_config_dir();
    PathBuf::from(&config_dir).join("identity").join("device.json")
}

fn base64_url_encode(data: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(data)
}

fn base64_url_decode(input: &str) -> Result<Vec<u8>, String> {
    URL_SAFE_NO_PAD.decode(input).map_err(|e| format!("Base64 解码失败: {}", e))
}

async fn load_or_create_device_identity() -> Result<DeviceIdentity, String> {
    let mut identity_guard = DEVICE_IDENTITY.lock().await;
    
    if let Some(ref identity) = *identity_guard {
        return Ok(identity.clone());
    }
    
    let identity_path = get_identity_path();
    
    if identity_path.exists() {
        if let Ok(content) = fs::read_to_string(&identity_path) {
            if let Ok(stored) = serde_json::from_str::<serde_json::Value>(&content) {
                if let (Some(private_key_b64), Some(public_key_b64)) = (
                    stored.get("privateKey").and_then(|v| v.as_str()),
                    stored.get("publicKey").and_then(|v| v.as_str()),
                ) {
                    let private_bytes = base64_url_decode(private_key_b64)?;
                    let public_bytes = base64_url_decode(public_key_b64)?;
                    
                    if private_bytes.len() == 32 && public_bytes.len() == 32 {
                        let mut seed = [0u8; 32];
                        seed.copy_from_slice(&private_bytes);
                        
                        let signing_key = SigningKey::from_bytes(&seed);
                        let verifying_key = signing_key.verifying_key();
                        
                        let device_id = {
                            let mut hasher = Sha256::new();
                            hasher.update(&public_bytes);
                            format!("{:x}", hasher.finalize())
                        };
                        
                        let identity = DeviceIdentity {
                            device_id,
                            signing_key,
                            verifying_key,
                        };
                        
                        *identity_guard = Some(identity.clone());
                        return Ok(identity);
                    }
                }
            }
        }
    }
    
    let mut csprng = OsRng;
    let signing_key = SigningKey::generate(&mut csprng);
    let verifying_key = signing_key.verifying_key();
    
    let public_bytes = verifying_key.to_bytes();
    let private_bytes = signing_key.to_bytes();
    
    let device_id = {
        let mut hasher = Sha256::new();
        hasher.update(&public_bytes);
        format!("{:x}", hasher.finalize())
    };
    
    if let Some(parent) = identity_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("创建身份目录失败: {}", e))?;
    }
    
    let stored = serde_json::json!({
        "version": 1,
        "deviceId": device_id,
        "publicKey": base64_url_encode(&public_bytes),
        "privateKey": base64_url_encode(&private_bytes),
        "createdAtMs": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    });
    
    fs::write(&identity_path, serde_json::to_string_pretty(&stored).unwrap())
        .map_err(|e| format!("保存身份文件失败: {}", e))?;
    
    let identity = DeviceIdentity {
        device_id,
        signing_key,
        verifying_key,
    };
    
    *identity_guard = Some(identity.clone());
    Ok(identity)
}

fn build_device_auth_payload(
    device_id: &str,
    client_id: &str,
    client_mode: &str,
    role: &str,
    scopes: &[&str],
    signed_at_ms: u64,
    token: Option<&str>,
    nonce: &str,
    platform: &str,
) -> String {
    let scopes_str = scopes.join(",");
    let token_str = token.unwrap_or("");
    
    format!(
        "v3|{}|{}|{}|{}|{}|{}|{}|{}|{}|",
        device_id,
        client_id,
        client_mode,
        role,
        scopes_str,
        signed_at_ms,
        token_str,
        nonce,
        platform
    )
}

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

fn check_port_listening(port: u16) -> bool {
    #[cfg(unix)]
    {
        use std::process::Command;
        let output = Command::new("lsof")
            .args(["-ti", &format!(":{}", port)])
            .output()
            .ok();
        
        output.map(|o| o.status.success()).unwrap_or(false)
    }
    
    #[cfg(windows)]
    {
        use std::process::Command;
        let mut cmd = Command::new("netstat");
        cmd.args(["-ano"]);
        cmd.creation_flags(CREATE_NO_WINDOW);
        
        let output = cmd.output().ok();
        
        output.map(|o| {
            let stdout = String::from_utf8_lossy(&o.stdout);
            stdout.lines().any(|line| {
                line.contains(&format!(":{}", port)) && line.contains("LISTENING")
            })
        }).unwrap_or(false)
    }
}

fn parse_ws_url(url: &str) -> String {
    if url.starts_with("ws://") || url.starts_with("wss://") {
        url.to_string()
    } else if url.starts_with("http://") {
        url.replace("http://", "ws://")
    } else if url.starts_with("https://") {
        url.replace("https://", "wss://")
    } else {
        format!("ws://{}", url)
    }
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
    
    if !check_port_listening(SERVICE_PORT) {
        warn!("[聊天] Gateway 服务未运行，返回空列表");
        let mut result = HashMap::new();
        result.insert("sessions".to_string(), Vec::new());
        return Ok(result);
    }
    
    let connection = GATEWAY_CONNECTION.lock().await;
    let gateway = match connection.as_ref() {
        Some(g) => g,
        None => {
            warn!("[聊天] 未连接 Gateway，返回空列表");
            let mut result = HashMap::new();
            result.insert("sessions".to_string(), Vec::new());
            return Ok(result);
        }
    };
    
    let identity = load_or_create_device_identity().await?;
    let ws_url = parse_ws_url(&gateway.url);
    
    info!("[聊天] 连接 WebSocket: {}", ws_url);
    
    let (ws_stream, _) = connect_async(&ws_url)
        .await
        .map_err(|e| format!("WebSocket 连接失败: {}", e))?;
    
    let (mut write, mut read) = ws_stream.split();
    
    let challenge_event = tokio::time::timeout(
        Duration::from_secs(5),
        read.next()
    )
    .await
    .map_err(|_| "等待 challenge 超时".to_string())?
    .ok_or("未收到 challenge 事件".to_string())?
    .map_err(|e| format!("读取 challenge 失败: {}", e))?;
    
    let challenge_text = match challenge_event {
        WsMessage::Text(text) => text,
        _ => return Err("无效的 challenge 格式".to_string()),
    };
    
    let challenge: WsEvent = serde_json::from_str(&challenge_text)
        .map_err(|e| format!("解析 challenge 失败: {}", e))?;
    
    if challenge.event != "connect.challenge" {
        return Err(format!("期望 challenge 事件，收到: {}", challenge.event));
    }
    
    let challenge_payload: ConnectChallenge = serde_json::from_value(challenge.payload)
        .map_err(|e| format!("解析 challenge payload 失败: {}", e))?;
    
    debug!("[聊天] 收到 challenge: nonce={}", challenge_payload.nonce);
    
    let request_id = Uuid::new_v4().to_string();
    let signed_at_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    
    let scopes = ["operator.read", "operator.write"];
    let platform_name = std::env::consts::OS;
    
    let payload = build_device_auth_payload(
        &identity.device_id,
        CLIENT_ID,
        CLIENT_MODE,
        "operator",
        &scopes,
        signed_at_ms,
        Some(&gateway.token),
        &challenge_payload.nonce,
        platform_name,
    );
    
    let signature = identity.signing_key.sign(payload.as_bytes());
    let signature_b64 = base64_url_encode(&signature.to_bytes());
    let public_key_b64 = base64_url_encode(&identity.verifying_key.to_bytes());
    
    let connect_params = serde_json::json!({
        "minProtocol": GATEWAY_PROTOCOL_VERSION,
        "maxProtocol": GATEWAY_PROTOCOL_VERSION,
        "client": {
            "id": CLIENT_ID,
            "version": env!("CARGO_PKG_VERSION"),
            "platform": platform_name,
            "mode": CLIENT_MODE
        },
        "role": "operator",
        "scopes": scopes,
        "caps": [],
        "auth": {
            "token": gateway.token
        },
        "device": {
            "id": identity.device_id,
            "publicKey": public_key_b64,
            "signature": signature_b64,
            "signedAt": signed_at_ms,
            "nonce": challenge_payload.nonce
        }
    });
    
    let connect_request = WsRequest {
        msg_type: "req".to_string(),
        id: request_id.clone(),
        method: "connect".to_string(),
        params: connect_params,
    };
    
    let connect_json = serde_json::to_string(&connect_request)
        .map_err(|e| format!("序列化 connect 请求失败: {}", e))?;
    
    write.send(WsMessage::Text(connect_json))
        .await
        .map_err(|e| format!("发送 connect 请求失败: {}", e))?;
    
    let connect_response = tokio::time::timeout(
        Duration::from_secs(5),
        read.next()
    )
    .await
    .map_err(|_| "等待 connect 响应超时".to_string())?
    .ok_or("未收到 connect 响应".to_string())?
    .map_err(|e| format!("读取 connect 响应失败: {}", e))?;
    
    let connect_text = match connect_response {
        WsMessage::Text(text) => text,
        _ => return Err("无效的 connect 响应格式".to_string()),
    };
    
    let connect_res: WsResponse = serde_json::from_str(&connect_text)
        .map_err(|e| format!("解析 connect 响应失败: {}", e))?;
    
    if !connect_res.ok.unwrap_or(false) {
        let error_msg = connect_res.error
            .and_then(|e| e.get("message").and_then(|m| m.as_str().map(|s| s.to_string())))
            .unwrap_or_else(|| "未知错误".to_string());
        return Err(format!("连接失败: {}", error_msg));
    }
    
    debug!("[聊天] WebSocket 握手成功");
    
    let sessions_request_id = Uuid::new_v4().to_string();
    let sessions_request = WsRequest {
        msg_type: "req".to_string(),
        id: sessions_request_id.clone(),
        method: "sessions.list".to_string(),
        params: serde_json::json!({
            "includeGlobal": true,
            "includeUnknown": false,
            "limit": 50
        }),
    };
    
    let sessions_json = serde_json::to_string(&sessions_request)
        .map_err(|e| format!("序列化请求失败: {}", e))?;

    write.send(WsMessage::Text(sessions_json))
        .await
        .map_err(|e| format!("发送请求失败: {}", e))?;

    info!("[聊天] 等待sessions.list响应，请求id={}", sessions_request_id);

    // 循环读取消息直到找到匹配的响应
    let mut matched_response: Option<String> = None;
    let start_time = std::time::Instant::now();
    let timeout_duration = Duration::from_secs(15);

    while start_time.elapsed() < timeout_duration {
        let sessions_response = tokio::time::timeout(
            Duration::from_secs(5),
            read.next()
        )
        .await;

        match sessions_response {
            Ok(Some(Ok(response))) => {
                let text = match response {
                    WsMessage::Text(t) => t,
                    _ => continue,
                };

                info!("[聊天] 检查消息: {}", text);

                // 检查是否是事件广播
                if text.contains(r#""type":"event""#) {
                    info!("[聊天] 收到事件广播，继续等待...");
                    continue;
                }

                // 尝试解析并检查id是否匹配
                if let Ok(res) = serde_json::from_str::<WsResponse>(&text) {
                    if res.msg_type == "res" {
                        if let Some(ref rid) = res.id {
                            if rid == &sessions_request_id {
                                info!("[聊天] 找到匹配的响应!");
                                matched_response = Some(text);
                                break;
                            } else {
                                info!("[聊天] id不匹配: {} != {}", rid, sessions_request_id);
                                continue;
                            }
                        }
                    }
                }
            }
            Ok(Some(Err(e))) => {
                info!("[聊天] WebSocket错误: {}", e);
                break;
            }
            Ok(None) => {
                info!("[聊天] WebSocket连接关闭");
                break;
            }
            Err(_) => {
                info!("[聊天] 等待消息超时，继续等待...");
                continue;
            }
        }
    }

    let sessions_text = matched_response
        .ok_or_else(|| "未收到匹配的响应".to_string())?;

    info!("[聊天] sessions原始响应: {}", sessions_text);

    let sessions_res: WsResponse = serde_json::from_str(&sessions_text)
        .map_err(|e| format!("解析响应失败: {}", e))?;

    info!("[聊天] sessions响应解析: msg_type={}, ok={:?}, has_payload={}",
        sessions_res.msg_type,
        sessions_res.ok,
        sessions_res.payload.is_some());

    if let Some(ref err) = sessions_res.error {
        info!("[聊天] sessions错误详情: {}", err);
    }

    if !sessions_res.ok.unwrap_or(false) {
        let error_msg = sessions_res.error
            .and_then(|e| e.get("message").and_then(|m| m.as_str().map(|s| s.to_string())))
            .unwrap_or_else(|| "获取会话列表失败".to_string());
        return Err(error_msg);
    }
    
    let payload = sessions_res.payload.unwrap_or(serde_json::json!({}));
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
            
            let session = ChatSession {
                key: key.clone(),
                id: obj.get("sessionId").and_then(|i| i.as_str()).unwrap_or(&key).to_string(),
                title: obj.get("title").and_then(|t| t.as_str()).unwrap_or(&key).to_string(),
                agent_id: obj.get("agentId").and_then(|a| a.as_str().map(|s| s.to_string())),
                model_id: obj.get("modelId").and_then(|m| m.as_str().map(|s| s.to_string())),
                created_at: obj.get("createdAt").and_then(|t| t.as_u64()).unwrap_or(0),
                updated_at: obj.get("updatedAt").and_then(|t| t.as_u64()).unwrap_or(0),
                message_count: obj.get("messageCount").and_then(|c| c.as_u64()).unwrap_or(0) as u32,
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
    
    if !check_port_listening(SERVICE_PORT) {
        warn!("[聊天] Gateway 服务未运行，返回空列表");
        let mut result = HashMap::new();
        result.insert("messages".to_string(), Vec::new());
        return Ok(result);
    }
    
    let connection = GATEWAY_CONNECTION.lock().await;
    let gateway = match connection.as_ref() {
        Some(g) => g,
        None => {
            warn!("[聊天] 未连接 Gateway，返回空列表");
            let mut result = HashMap::new();
            result.insert("messages".to_string(), Vec::new());
            return Ok(result);
        }
    };
    
    let identity = load_or_create_device_identity().await?;
    let ws_url = parse_ws_url(&gateway.url);
    
    info!("[聊天] 连接 WebSocket: {}", ws_url);
    
    let (ws_stream, _) = connect_async(&ws_url)
        .await
        .map_err(|e| format!("WebSocket 连接失败: {}", e))?;
    
    let (mut write, mut read) = ws_stream.split();
    
    let challenge_event = tokio::time::timeout(
        Duration::from_secs(5),
        read.next()
    )
    .await
    .map_err(|_| "等待 challenge 超时".to_string())?
    .ok_or("未收到 challenge 事件".to_string())?
    .map_err(|e| format!("读取 challenge 失败: {}", e))?;
    
    let challenge_text = match challenge_event {
        WsMessage::Text(text) => text,
        _ => return Err("无效的 challenge 格式".to_string()),
    };
    
    let challenge: WsEvent = serde_json::from_str(&challenge_text)
        .map_err(|e| format!("解析 challenge 失败: {}", e))?;
    
    if challenge.event != "connect.challenge" {
        return Err(format!("期望 challenge 事件，收到: {}", challenge.event));
    }
    
    let challenge_payload: ConnectChallenge = serde_json::from_value(challenge.payload)
        .map_err(|e| format!("解析 challenge payload 失败: {}", e))?;
    
    let request_id = Uuid::new_v4().to_string();
    let signed_at_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    
    let scopes = ["operator.read", "operator.write"];
    let platform_name = std::env::consts::OS;
    
    let payload = build_device_auth_payload(
        &identity.device_id,
        CLIENT_ID,
        CLIENT_MODE,
        "operator",
        &scopes,
        signed_at_ms,
        Some(&gateway.token),
        &challenge_payload.nonce,
        platform_name,
    );
    
    let signature = identity.signing_key.sign(payload.as_bytes());
    let signature_b64 = base64_url_encode(&signature.to_bytes());
    let public_key_b64 = base64_url_encode(&identity.verifying_key.to_bytes());
    
    let connect_params = serde_json::json!({
        "minProtocol": GATEWAY_PROTOCOL_VERSION,
        "maxProtocol": GATEWAY_PROTOCOL_VERSION,
        "client": {
            "id": CLIENT_ID,
            "version": env!("CARGO_PKG_VERSION"),
            "platform": platform_name,
            "mode": CLIENT_MODE
        },
        "role": "operator",
        "scopes": scopes,
        "caps": [],
        "auth": {
            "token": gateway.token
        },
        "device": {
            "id": identity.device_id,
            "publicKey": public_key_b64,
            "signature": signature_b64,
            "signedAt": signed_at_ms,
            "nonce": challenge_payload.nonce
        }
    });
    
    let connect_request = WsRequest {
        msg_type: "req".to_string(),
        id: request_id.clone(),
        method: "connect".to_string(),
        params: connect_params,
    };
    
    let connect_json = serde_json::to_string(&connect_request)
        .map_err(|e| format!("序列化 connect 请求失败: {}", e))?;
    
    write.send(WsMessage::Text(connect_json))
        .await
        .map_err(|e| format!("发送 connect 请求失败: {}", e))?;
    
    let connect_response = tokio::time::timeout(
        Duration::from_secs(5),
        read.next()
    )
    .await
    .map_err(|_| "等待 connect 响应超时".to_string())?
    .ok_or("未收到 connect 响应".to_string())?
    .map_err(|e| format!("读取 connect 响应失败: {}", e))?;
    
    let connect_text = match connect_response {
        WsMessage::Text(text) => text,
        _ => return Err("无效的 connect 响应格式".to_string()),
    };
    
    let connect_res: WsResponse = serde_json::from_str(&connect_text)
        .map_err(|e| format!("解析 connect 响应失败: {}", e))?;
    
    if !connect_res.ok.unwrap_or(false) {
        let error_msg = connect_res.error
            .and_then(|e| e.get("message").and_then(|m| m.as_str().map(|s| s.to_string())))
            .unwrap_or_else(|| "未知错误".to_string());
        return Err(format!("连接失败: {}", error_msg));
    }
    
    debug!("[聊天] WebSocket 握手成功");
    
    let history_request_id = Uuid::new_v4().to_string();
    let history_request = WsRequest {
        msg_type: "req".to_string(),
        id: history_request_id.clone(),
        method: "chat.history".to_string(),
        params: serde_json::json!({
            "sessionKey": session_key
        }),
    };
    
    let history_json = serde_json::to_string(&history_request)
        .map_err(|e| format!("序列化请求失败: {}", e))?;

    write.send(WsMessage::Text(history_json))
        .await
        .map_err(|e| format!("发送请求失败: {}", e))?;

    info!("[聊天] 等待chat.history响应，请求id={}", history_request_id);

    // 循环读取消息直到找到匹配的响应
    let mut matched_response: Option<String> = None;
    let start_time = std::time::Instant::now();
    let timeout_duration = Duration::from_secs(15);

    while start_time.elapsed() < timeout_duration {
        let history_resp = tokio::time::timeout(
            Duration::from_secs(5),
            read.next()
        )
        .await;

        match history_resp {
            Ok(Some(Ok(response))) => {
                let text = match response {
                    WsMessage::Text(t) => t,
                    _ => continue,
                };

                info!("[聊天] 检查消息: {}", text);

                // 检查是否是事件广播
                if text.contains(r#""type":"event""#) {
                    info!("[聊天] 收到事件广播，继续等待...");
                    continue;
                }

                // 尝试解析并检查id是否匹配
                if let Ok(res) = serde_json::from_str::<WsResponse>(&text) {
                    if res.msg_type == "res" {
                        if let Some(ref rid) = res.id {
                            if rid == &history_request_id {
                                info!("[聊天] 找到匹配的响应!");
                                matched_response = Some(text);
                                break;
                            } else {
                                info!("[聊天] id不匹配: {} != {}", rid, history_request_id);
                                continue;
                            }
                        }
                    }
                }
            }
            Ok(Some(Err(e))) => {
                info!("[聊天] WebSocket错误: {}", e);
                break;
            }
            Ok(None) => {
                info!("[聊天] WebSocket连接关闭");
                break;
            }
            Err(_) => {
                info!("[聊天] 等待消息超时，继续等待...");
                continue;
            }
        }
    }

    let history_text = matched_response
        .ok_or_else(|| "未收到匹配的响应".to_string())?;

    let history_res: WsResponse = serde_json::from_str(&history_text)
        .map_err(|e| format!("解析响应失败: {}", e))?;

    if !history_res.ok.unwrap_or(false) {
        let error_msg = history_res.error
            .and_then(|e| e.get("message").and_then(|m| m.as_str().map(|s| s.to_string())))
            .unwrap_or_else(|| "获取消息历史失败".to_string());
        return Err(error_msg);
    }
    
    let payload = history_res.payload.unwrap_or(serde_json::json!({}));
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
    
    if !check_port_listening(SERVICE_PORT) {
        warn!("[聊天] Gateway 服务未运行 (端口 {} 未监听)", SERVICE_PORT);
        let mut result = HashMap::new();
        result.insert("success".to_string(), false);
        return Ok(result);
    }
    
    let mut connection = GATEWAY_CONNECTION.lock().await;
    *connection = Some(GatewayConnection {
        url: config.url.clone(),
        token: config.token.clone(),
    });
    
    info!("[聊天] ✓ Gateway 连接配置已保存");
    
    let mut result = HashMap::new();
    result.insert("success".to_string(), true);
    Ok(result)
}

#[command]
pub async fn disconnect_gateway() -> Result<(), String> {
    info!("[聊天] 断开 Gateway 连接");
    
    let mut connection = GATEWAY_CONNECTION.lock().await;
    *connection = None;
    
    Ok(())
}

#[command]
pub async fn create_session(
    agent_id: Option<String>,
    model_id: Option<String>,
) -> Result<HashMap<String, ChatSession>, String> {
    info!("[聊天] 创建新会话: agent={:?}, model={:?}", agent_id, model_id);
    
    if !check_port_listening(SERVICE_PORT) {
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
    
    let connection = GATEWAY_CONNECTION.lock().await;
    let gateway = match connection.as_ref() {
        Some(g) => g,
        None => {
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
    };
    
    let identity = load_or_create_device_identity().await?;
    let ws_url = parse_ws_url(&gateway.url);
    
    info!("[聊天] 连接 WebSocket: {}", ws_url);
    
    let (ws_stream, _) = connect_async(&ws_url)
        .await
        .map_err(|e| format!("WebSocket 连接失败: {}", e))?;
    
    let (mut write, mut read) = ws_stream.split();
    
    let challenge_event = tokio::time::timeout(
        Duration::from_secs(5),
        read.next()
    )
    .await
    .map_err(|_| "等待 challenge 超时".to_string())?
    .ok_or("未收到 challenge 事件".to_string())?
    .map_err(|e| format!("读取 challenge 失败: {}", e))?;
    
    let challenge_text = match challenge_event {
        WsMessage::Text(text) => text,
        _ => return Err("无效的 challenge 格式".to_string()),
    };
    
    let challenge: WsEvent = serde_json::from_str(&challenge_text)
        .map_err(|e| format!("解析 challenge 失败: {}", e))?;
    
    if challenge.event != "connect.challenge" {
        return Err(format!("期望 challenge 事件，收到: {}", challenge.event));
    }
    
    let challenge_payload: ConnectChallenge = serde_json::from_value(challenge.payload)
        .map_err(|e| format!("解析 challenge payload 失败: {}", e))?;
    
    let request_id = Uuid::new_v4().to_string();
    let signed_at_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    
    let scopes = ["operator.read", "operator.write"];
    let platform_name = std::env::consts::OS;
    
    let payload = build_device_auth_payload(
        &identity.device_id,
        CLIENT_ID,
        CLIENT_MODE,
        "operator",
        &scopes,
        signed_at_ms,
        Some(&gateway.token),
        &challenge_payload.nonce,
        platform_name,
    );
    
    let signature = identity.signing_key.sign(payload.as_bytes());
    let signature_b64 = base64_url_encode(&signature.to_bytes());
    let public_key_b64 = base64_url_encode(&identity.verifying_key.to_bytes());
    
    let connect_params = serde_json::json!({
        "minProtocol": GATEWAY_PROTOCOL_VERSION,
        "maxProtocol": GATEWAY_PROTOCOL_VERSION,
        "client": {
            "id": CLIENT_ID,
            "version": env!("CARGO_PKG_VERSION"),
            "platform": platform_name,
            "mode": CLIENT_MODE
        },
        "role": "operator",
        "scopes": scopes,
        "caps": [],
        "auth": {
            "token": gateway.token
        },
        "device": {
            "id": identity.device_id,
            "publicKey": public_key_b64,
            "signature": signature_b64,
            "signedAt": signed_at_ms,
            "nonce": challenge_payload.nonce
        }
    });
    
    let connect_request = WsRequest {
        msg_type: "req".to_string(),
        id: request_id.clone(),
        method: "connect".to_string(),
        params: connect_params,
    };
    
    let connect_json = serde_json::to_string(&connect_request)
        .map_err(|e| format!("序列化 connect 请求失败: {}", e))?;
    
    write.send(WsMessage::Text(connect_json))
        .await
        .map_err(|e| format!("发送 connect 请求失败: {}", e))?;
    
    let connect_response = tokio::time::timeout(
        Duration::from_secs(5),
        read.next()
    )
    .await
    .map_err(|_| "等待 connect 响应超时".to_string())?
    .ok_or("未收到 connect 响应".to_string())?
    .map_err(|e| format!("读取 connect 响应失败: {}", e))?;
    
    let connect_text = match connect_response {
        WsMessage::Text(text) => text,
        _ => return Err("无效的 connect 响应格式".to_string()),
    };
    
    let connect_res: WsResponse = serde_json::from_str(&connect_text)
        .map_err(|e| format!("解析 connect 响应失败: {}", e))?;
    
    if !connect_res.ok.unwrap_or(false) {
        let error_msg = connect_res.error
            .and_then(|e| e.get("message").and_then(|m| m.as_str().map(|s| s.to_string())))
            .unwrap_or_else(|| "未知错误".to_string());
        return Err(format!("连接失败: {}", error_msg));
    }
    
    debug!("[聊天] WebSocket 握手成功");
    
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    
    let session_key = format!("session-{}", timestamp);
    
    let mut patch_params = serde_json::json!({
        "key": session_key,
        "title": "新会话"
    });
    
    if let Some(ref aid) = agent_id {
        patch_params["agentId"] = serde_json::json!(aid);
    }
    if let Some(ref mid) = model_id {
        patch_params["modelId"] = serde_json::json!(mid);
    }
    
    let patch_request_id = Uuid::new_v4().to_string();
    let patch_request = WsRequest {
        msg_type: "req".to_string(),
        id: patch_request_id.clone(),
        method: "sessions.patch".to_string(),
        params: patch_params,
    };
    
    let patch_json = serde_json::to_string(&patch_request)
        .map_err(|e| format!("序列化请求失败: {}", e))?;
    
    write.send(WsMessage::Text(patch_json))
        .await
        .map_err(|e| format!("发送请求失败: {}", e))?;
    
    let patch_response = tokio::time::timeout(
        Duration::from_secs(15),
        read.next()
    )
    .await
    .map_err(|_| "等待响应超时".to_string())?
    .ok_or("未收到响应".to_string())?
    .map_err(|e| format!("读取响应失败: {}", e))?;
    
    let patch_text = match patch_response {
        WsMessage::Text(text) => text,
        _ => return Err("无效的响应格式".to_string()),
    };
    
    let patch_res: WsResponse = serde_json::from_str(&patch_text)
        .map_err(|e| format!("解析响应失败: {}", e))?;
    
    if !patch_res.ok.unwrap_or(false) {
        let error_msg = patch_res.error
            .and_then(|e| e.get("message").and_then(|m| m.as_str().map(|s| s.to_string())))
            .unwrap_or_else(|| "创建会话失败".to_string());
        return Err(error_msg);
    }
    
    let entry = patch_res.payload
        .and_then(|p| p.get("entry").cloned())
        .unwrap_or(serde_json::json!({}));
    
    let session_key = entry.get("key").and_then(|k| k.as_str()).unwrap_or(&session_key).to_string();
    
    let session = ChatSession {
        key: session_key.clone(),
        id: entry.get("sessionId").and_then(|i| i.as_str()).unwrap_or(&session_key).to_string(),
        title: entry.get("title").and_then(|t| t.as_str()).unwrap_or("新会话").to_string(),
        agent_id: entry.get("agentId").and_then(|a| a.as_str().map(|s| s.to_string())).or(agent_id),
        model_id: entry.get("modelId").and_then(|m| m.as_str().map(|s| s.to_string())).or(model_id),
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
    
    if !check_port_listening(SERVICE_PORT) {
        warn!("[聊天] Gateway 服务未运行，使用本地删除");
        let session_file = get_session_file(&session_key)?;
        
        if session_file.exists() {
            fs::remove_file(&session_file)
                .map_err(|e| format!("删除会话文件失败: {}", e))?;
        }
        
        let messages_file = session_file.with_extension("messages.json");
        if messages_file.exists() {
            fs::remove_file(&messages_file)
                .map_err(|e| format!("删除消息文件失败: {}", e))?;
        }
        
        return Ok(());
    }
    
    let connection = GATEWAY_CONNECTION.lock().await;
    let gateway = match connection.as_ref() {
        Some(g) => g,
        None => {
            warn!("[聊天] 未连接 Gateway，使用本地删除");
            let session_file = get_session_file(&session_key)?;
            
            if session_file.exists() {
                fs::remove_file(&session_file)
                    .map_err(|e| format!("删除会话文件失败: {}", e))?;
            }
            
            let messages_file = session_file.with_extension("messages.json");
            if messages_file.exists() {
                fs::remove_file(&messages_file)
                    .map_err(|e| format!("删除消息文件失败: {}", e))?;
            }
            
            return Ok(());
        }
    };
    
    let identity = load_or_create_device_identity().await?;
    let ws_url = parse_ws_url(&gateway.url);
    
    info!("[聊天] 连接 WebSocket: {}", ws_url);
    
    let (ws_stream, _) = connect_async(&ws_url)
        .await
        .map_err(|e| format!("WebSocket 连接失败: {}", e))?;
    
    let (mut write, mut read) = ws_stream.split();
    
    let challenge_event = tokio::time::timeout(
        Duration::from_secs(5),
        read.next()
    )
    .await
    .map_err(|_| "等待 challenge 超时".to_string())?
    .ok_or("未收到 challenge 事件".to_string())?
    .map_err(|e| format!("读取 challenge 失败: {}", e))?;
    
    let challenge_text = match challenge_event {
        WsMessage::Text(text) => text,
        _ => return Err("无效的 challenge 格式".to_string()),
    };
    
    let challenge: WsEvent = serde_json::from_str(&challenge_text)
        .map_err(|e| format!("解析 challenge 失败: {}", e))?;
    
    if challenge.event != "connect.challenge" {
        return Err(format!("期望 challenge 事件，收到: {}", challenge.event));
    }
    
    let challenge_payload: ConnectChallenge = serde_json::from_value(challenge.payload)
        .map_err(|e| format!("解析 challenge payload 失败: {}", e))?;
    
    let request_id = Uuid::new_v4().to_string();
    let signed_at_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    
    let scopes = ["operator.read", "operator.write"];
    let platform_name = std::env::consts::OS;
    
    let payload = build_device_auth_payload(
        &identity.device_id,
        CLIENT_ID,
        CLIENT_MODE,
        "operator",
        &scopes,
        signed_at_ms,
        Some(&gateway.token),
        &challenge_payload.nonce,
        platform_name,
    );
    
    let signature = identity.signing_key.sign(payload.as_bytes());
    let signature_b64 = base64_url_encode(&signature.to_bytes());
    let public_key_b64 = base64_url_encode(&identity.verifying_key.to_bytes());
    
    let connect_params = serde_json::json!({
        "minProtocol": GATEWAY_PROTOCOL_VERSION,
        "maxProtocol": GATEWAY_PROTOCOL_VERSION,
        "client": {
            "id": CLIENT_ID,
            "version": env!("CARGO_PKG_VERSION"),
            "platform": platform_name,
            "mode": CLIENT_MODE
        },
        "role": "operator",
        "scopes": scopes,
        "caps": [],
        "auth": {
            "token": gateway.token
        },
        "device": {
            "id": identity.device_id,
            "publicKey": public_key_b64,
            "signature": signature_b64,
            "signedAt": signed_at_ms,
            "nonce": challenge_payload.nonce
        }
    });
    
    let connect_request = WsRequest {
        msg_type: "req".to_string(),
        id: request_id.clone(),
        method: "connect".to_string(),
        params: connect_params,
    };
    
    let connect_json = serde_json::to_string(&connect_request)
        .map_err(|e| format!("序列化 connect 请求失败: {}", e))?;
    
    write.send(WsMessage::Text(connect_json))
        .await
        .map_err(|e| format!("发送 connect 请求失败: {}", e))?;
    
    let connect_response = tokio::time::timeout(
        Duration::from_secs(5),
        read.next()
    )
    .await
    .map_err(|_| "等待 connect 响应超时".to_string())?
    .ok_or("未收到 connect 响应".to_string())?
    .map_err(|e| format!("读取 connect 响应失败: {}", e))?;
    
    let connect_text = match connect_response {
        WsMessage::Text(text) => text,
        _ => return Err("无效的 connect 响应格式".to_string()),
    };
    
    let connect_res: WsResponse = serde_json::from_str(&connect_text)
        .map_err(|e| format!("解析 connect 响应失败: {}", e))?;
    
    if !connect_res.ok.unwrap_or(false) {
        let error_msg = connect_res.error
            .and_then(|e| e.get("message").and_then(|m| m.as_str().map(|s| s.to_string())))
            .unwrap_or_else(|| "未知错误".to_string());
        return Err(format!("连接失败: {}", error_msg));
    }
    
    debug!("[聊天] WebSocket 握手成功");
    
    let delete_request_id = Uuid::new_v4().to_string();
    let delete_request = WsRequest {
        msg_type: "req".to_string(),
        id: delete_request_id.clone(),
        method: "sessions.delete".to_string(),
        params: serde_json::json!({
            "key": session_key,
            "deleteTranscript": true
        }),
    };
    
    let delete_json = serde_json::to_string(&delete_request)
        .map_err(|e| format!("序列化请求失败: {}", e))?;
    
    write.send(WsMessage::Text(delete_json))
        .await
        .map_err(|e| format!("发送请求失败: {}", e))?;
    
    let delete_response = tokio::time::timeout(
        Duration::from_secs(15),
        read.next()
    )
    .await
    .map_err(|_| "等待响应超时".to_string())?
    .ok_or("未收到响应".to_string())?
    .map_err(|e| format!("读取响应失败: {}", e))?;
    
    let delete_text = match delete_response {
        WsMessage::Text(text) => text,
        _ => return Err("无效的响应格式".to_string()),
    };
    
    let delete_res: WsResponse = serde_json::from_str(&delete_text)
        .map_err(|e| format!("解析响应失败: {}", e))?;
    
    if !delete_res.ok.unwrap_or(false) {
        let error_msg = delete_res.error
            .and_then(|e| e.get("message").and_then(|m| m.as_str().map(|s| s.to_string())))
            .unwrap_or_else(|| "删除会话失败".to_string());
        return Err(error_msg);
    }
    
    info!("[聊天] ✓ 会话已删除: {}", session_key);
    Ok(())
}

#[command]
pub async fn send_chat_message(
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
    
    if !check_port_listening(SERVICE_PORT) {
        error!("[聊天] Gateway 服务未运行");
        return Err("Gateway 服务未运行，请先启动服务".to_string());
    }
    
    let connection = GATEWAY_CONNECTION.lock().await;
    let gateway = connection.as_ref()
        .ok_or("未连接到 Gateway，请先配置并连接".to_string())?;
    
    let session = session_key.unwrap_or_else(|| "main".to_string());
    let run_id = Uuid::new_v4().to_string();
    
    let identity = load_or_create_device_identity().await?;
    let ws_url = parse_ws_url(&gateway.url);
    
    info!("[聊天] 连接 WebSocket: {}", ws_url);
    
    let (ws_stream, _) = connect_async(&ws_url)
        .await
        .map_err(|e| format!("WebSocket 连接失败: {}", e))?;
    
    let (mut write, mut read) = ws_stream.split();
    
    let challenge_event = tokio::time::timeout(
        Duration::from_secs(5),
        read.next()
    )
    .await
    .map_err(|_| "等待 challenge 超时".to_string())?
    .ok_or("未收到 challenge 事件".to_string())?
    .map_err(|e| format!("读取 challenge 失败: {}", e))?;
    
    let challenge_text = match challenge_event {
        WsMessage::Text(text) => text,
        _ => return Err("无效的 challenge 格式".to_string()),
    };
    
    let challenge: WsEvent = serde_json::from_str(&challenge_text)
        .map_err(|e| format!("解析 challenge 失败: {}", e))?;
    
    if challenge.event != "connect.challenge" {
        return Err(format!("期望 challenge 事件，收到: {}", challenge.event));
    }
    
    let challenge_payload: ConnectChallenge = serde_json::from_value(challenge.payload)
        .map_err(|e| format!("解析 challenge payload 失败: {}", e))?;
    
    debug!("[聊天] 收到 challenge: nonce={}", challenge_payload.nonce);
    
    let request_id = Uuid::new_v4().to_string();
    let signed_at_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    
    let scopes = ["operator.read", "operator.write"];
    let platform_name = std::env::consts::OS;
    
    let payload = build_device_auth_payload(
        &identity.device_id,
        CLIENT_ID,
        CLIENT_MODE,
        "operator",
        &scopes,
        signed_at_ms,
        Some(&gateway.token),
        &challenge_payload.nonce,
        platform_name,
    );
    
    let signature = identity.signing_key.sign(payload.as_bytes());
    let signature_b64 = base64_url_encode(&signature.to_bytes());
    let public_key_b64 = base64_url_encode(&identity.verifying_key.to_bytes());
    
    let connect_params = serde_json::json!({
        "minProtocol": GATEWAY_PROTOCOL_VERSION,
        "maxProtocol": GATEWAY_PROTOCOL_VERSION,
        "client": {
            "id": CLIENT_ID,
            "version": env!("CARGO_PKG_VERSION"),
            "platform": platform_name,
            "mode": CLIENT_MODE
        },
        "role": "operator",
        "scopes": scopes,
        "caps": [],
        "auth": {
            "token": gateway.token
        },
        "device": {
            "id": identity.device_id,
            "publicKey": public_key_b64,
            "signature": signature_b64,
            "signedAt": signed_at_ms,
            "nonce": challenge_payload.nonce
        }
    });
    
    let connect_request = WsRequest {
        msg_type: "req".to_string(),
        id: request_id.clone(),
        method: "connect".to_string(),
        params: connect_params,
    };
    
    let connect_json = serde_json::to_string(&connect_request)
        .map_err(|e| format!("序列化 connect 请求失败: {}", e))?;
    
    write.send(WsMessage::Text(connect_json))
        .await
        .map_err(|e| format!("发送 connect 请求失败: {}", e))?;
    
    let connect_response = tokio::time::timeout(
        Duration::from_secs(5),
        read.next()
    )
    .await
    .map_err(|_| "等待 connect 响应超时".to_string())?
    .ok_or("未收到 connect 响应".to_string())?
    .map_err(|e| format!("读取 connect 响应失败: {}", e))?;
    
    let connect_text = match connect_response {
        WsMessage::Text(text) => text,
        _ => return Err("无效的 connect 响应格式".to_string()),
    };
    
    let connect_res: WsResponse = serde_json::from_str(&connect_text)
        .map_err(|e| format!("解析 connect 响应失败: {}", e))?;
    
    if !connect_res.ok.unwrap_or(false) {
        let error_msg = connect_res.error
            .and_then(|e| e.get("message").and_then(|m| m.as_str().map(|s| s.to_string())))
            .unwrap_or_else(|| "未知错误".to_string());
        return Err(format!("连接失败: {}", error_msg));
    }
    
    debug!("[聊天] WebSocket 握手成功");
    
    let method_request_id = Uuid::new_v4().to_string();
    
    let mut params = serde_json::json!({
        "sessionKey": session,
        "message": message,
        "idempotencyKey": run_id,
        "timeoutMs": 30000
    });
    
    if let Some(atts) = attachments {
        if !atts.is_empty() {
            params["attachments"] = serde_json::to_value(atts)
                .map_err(|e| format!("序列化附件失败: {}", e))?;
        }
    }
    
    let method_request = WsRequest {
        msg_type: "req".to_string(),
        id: method_request_id.clone(),
        method: "chat.send".to_string(),
        params,
    };
    
    let method_json = serde_json::to_string(&method_request)
        .map_err(|e| format!("序列化请求失败: {}", e))?;
    
    write.send(WsMessage::Text(method_json))
        .await
        .map_err(|e| format!("发送请求失败: {}", e))?;
    
    let mut actual_run_id: Option<String> = None;
    let mut assistant_text = String::new();
    let mut final_received = false;
    
    while !final_received {
        let event = tokio::time::timeout(
            Duration::from_secs(120),
            read.next()
        )
        .await
        .map_err(|_| "等待事件超时".to_string())?
        .ok_or("连接已关闭".to_string())?
        .map_err(|e| format!("读取事件失败: {}", e))?;
        
        let event_text = match event {
            WsMessage::Text(text) => text,
            _ => continue,
        };
        
        let ws_msg: serde_json::Value = serde_json::from_str(&event_text)
            .map_err(|e| format!("解析事件失败: {}", e))?;
        
        if ws_msg["type"] == "res" && ws_msg["id"] == method_request_id {
            if !ws_msg["ok"].as_bool().unwrap_or(false) {
                let error_msg = ws_msg["error"]
                    .get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or("发送消息失败")
                    .to_string();
                return Err(error_msg);
            }
            
            if let Some(payload) = ws_msg.get("payload") {
                actual_run_id = payload.get("runId").and_then(|r| r.as_str()).map(|s| s.to_string());
            }
            continue;
        }
        
        if ws_msg["type"] == "event" && ws_msg["event"] == "chat" {
            let payload = &ws_msg["payload"];
            let state = payload.get("state").and_then(|s| s.as_str()).unwrap_or("");
            
            match state {
                "delta" => {
                    if let Some(message) = payload.get("message") {
                        if let Some(content) = message.get("content").and_then(|c| c.as_array()) {
                            for item in content {
                                if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
                                    assistant_text.push_str(text);
                                }
                            }
                        }
                    }
                }
                "final" | "error" => {
                    final_received = true;
                    if state == "error" {
                        let error_msg = payload.get("errorMessage")
                            .and_then(|m| m.as_str())
                            .unwrap_or("聊天失败")
                            .to_string();
                        return Err(error_msg);
                    }
                    if assistant_text.is_empty() {
                        if let Some(message) = payload.get("message") {
                            if let Some(content) = message.get("content").and_then(|c| c.as_array()) {
                                for item in content {
                                    if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
                                        assistant_text.push_str(text);
                                    }
                                }
                            }
                        }
                    }
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
    
    let is_running = check_port_listening(SERVICE_PORT);
    let connection = GATEWAY_CONNECTION.lock().await;
    let is_connected = connection.is_some() && is_running;
    
    info!("[聊天] Gateway 状态: running={}, connected={}", is_running, is_connected);
    
    let mut result = HashMap::new();
    result.insert("running".to_string(), is_running);
    result.insert("connected".to_string(), is_connected);
    Ok(result)
}
