use crate::commands::common::{check_port_listening, get_openclaw_config_path};
use crate::utils::platform;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use ed25519_dalek::{Signer, SigningKey, VerifyingKey};
use futures_util::{SinkExt, StreamExt};
use log::{debug, info, warn};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio_tungstenite::{connect_async, tungstenite::Message as WsMessage};
use uuid::Uuid;

pub const SERVICE_PORT: u16 = 18789;
pub const GATEWAY_PROTOCOL_VERSION: u32 = 3;

const CLIENT_ID: &str = "cli";
const CLIENT_MODE: &str = "cli";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayConnectConfig {
    pub url: String,
    pub token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
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

#[derive(Debug, Clone)]
struct GatewayConnection {
    url: String,
    token: String,
}

lazy_static::lazy_static! {
    static ref GATEWAY_CONNECTION: Arc<Mutex<Option<GatewayConnection>>> = Arc::new(Mutex::new(None));
    static ref DEVICE_IDENTITY: Arc<Mutex<Option<DeviceIdentity>>> = Arc::new(Mutex::new(None));
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

pub fn parse_ws_url(url: &str) -> String {
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

pub struct GatewayClient {
    write: futures_util::stream::SplitSink<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
        WsMessage,
    >,
    read: futures_util::stream::SplitStream<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
    >,
}

impl GatewayClient {
    pub async fn connect(url: &str, token: &str) -> Result<Self, String> {
        let identity = load_or_create_device_identity().await?;
        let ws_url = parse_ws_url(url);
        
        info!("[Gateway] 连接 WebSocket: {}", ws_url);
        
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
        
        debug!("[Gateway] 收到 challenge: nonce={}", challenge_payload.nonce);
        
        let request_id = Uuid::new_v4().to_string();
        let signed_at_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        
        let scopes = ["operator.read", "operator.write", "operator.admin"];
        let platform_name = std::env::consts::OS;
        
        let payload = build_device_auth_payload(
            &identity.device_id,
            CLIENT_ID,
            CLIENT_MODE,
            "operator",
            &scopes,
            signed_at_ms,
            Some(token),
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
                "token": token
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
        
        info!("[Gateway] WebSocket 握手成功");
        
        Ok(Self { write, read })
    }
    
    pub async fn send_request(
        &mut self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let request_id = Uuid::new_v4().to_string();
        
        let request = WsRequest {
            msg_type: "req".to_string(),
            id: request_id.clone(),
            method: method.to_string(),
            params,
        };
        
        let request_json = serde_json::to_string(&request)
            .map_err(|e| format!("序列化请求失败: {}", e))?;
        
        self.write.send(WsMessage::Text(request_json))
            .await
            .map_err(|e| format!("发送请求失败: {}", e))?;
        
        info!("[Gateway] 已发送 {} 请求，等待响应...", method);
        
        let start_time = std::time::Instant::now();
        let timeout_duration = Duration::from_secs(30);
        
        while start_time.elapsed() < timeout_duration {
            let response = tokio::time::timeout(
                Duration::from_secs(5),
                self.read.next()
            )
            .await;
            
            match response {
                Ok(Some(Ok(msg))) => {
                    let text = match msg {
                        WsMessage::Text(t) => t,
                        _ => continue,
                    };
                    
                    if text.contains(r#""type":"event""#) {
                        debug!("[Gateway] 收到事件广播，继续等待响应...");
                        continue;
                    }
                    
                    if let Ok(res) = serde_json::from_str::<WsResponse>(&text) {
                        if res.msg_type == "res" {
                            if let Some(ref rid) = res.id {
                                if rid == &request_id {
                                    if !res.ok.unwrap_or(false) {
                                        let error_msg = res.error
                                            .and_then(|e| e.get("message").and_then(|m| m.as_str().map(|s| s.to_string())))
                                            .unwrap_or_else(|| "未知错误".to_string());
                                        return Err(error_msg);
                                    }
                                    
                                    return Ok(res.payload.unwrap_or(serde_json::json!({})));
                                }
                            }
                        }
                    }
                }
                Ok(Some(Err(e))) => {
                    return Err(format!("WebSocket 错误: {}", e));
                }
                Ok(None) => {
                    return Err("WebSocket 连接已关闭".to_string());
                }
                Err(_) => {
                    continue;
                }
            }
        }
        
        Err("等待响应超时".to_string())
    }
    
    pub async fn read_event(&mut self) -> Result<Option<serde_json::Value>, String> {
        let response = tokio::time::timeout(
            Duration::from_secs(5),
            self.read.next()
        )
        .await;
        
        match response {
            Ok(Some(Ok(msg))) => {
                let text = match msg {
                    WsMessage::Text(t) => t,
                    _ => return Ok(None),
                };
                
                let ws_msg: serde_json::Value = serde_json::from_str(&text)
                    .map_err(|e| format!("解析消息失败: {}", e))?;
                
                Ok(Some(ws_msg))
            }
            Ok(Some(Err(e))) => Err(format!("WebSocket 错误: {}", e)),
            Ok(None) => Ok(None),
            Err(_) => Ok(None),
        }
    }
    
    pub async fn close(mut self) -> Result<(), String> {
        self.write.close().await
            .map_err(|e| format!("关闭连接失败: {}", e))
    }
}

pub async fn get_gateway_connection() -> Option<(String, String)> {
    let connection = GATEWAY_CONNECTION.lock().await;
    connection.as_ref().map(|g| (g.url.clone(), g.token.clone()))
}

pub async fn set_gateway_connection(url: String, token: String) {
    let mut connection = GATEWAY_CONNECTION.lock().await;
    *connection = Some(GatewayConnection { url, token });
}

pub async fn clear_gateway_connection() {
    let mut connection = GATEWAY_CONNECTION.lock().await;
    *connection = None;
}

pub fn is_gateway_running() -> bool {
    check_port_listening(SERVICE_PORT)
}

pub async fn is_gateway_connected() -> bool {
    let connection = GATEWAY_CONNECTION.lock().await;
    connection.is_some() && is_gateway_running()
}

pub async fn create_gateway_client() -> Result<GatewayClient, String> {
    let (url, token) = get_gateway_connection().await
        .ok_or("未连接到 Gateway，请先配置并连接".to_string())?;
    
    if !is_gateway_running() {
        return Err("Gateway 服务未运行，请先启动服务".to_string());
    }
    
    GatewayClient::connect(&url, &token).await
}

/// 尝试自动连接到 Gateway
/// 
/// 从 OpenClaw 配置文件中读取 Gateway 配置并尝试连接。
/// 如果连接成功，返回 GatewayClient；否则返回错误。
/// 
/// # 返回值
/// * `Ok(GatewayClient)` - 自动连接成功
/// * `Err(String)` - 自动连接失败（配置不存在、服务未运行或连接失败）
pub async fn try_auto_connect() -> Result<GatewayClient, String> {
    info!("[Gateway] 尝试自动连接...");
    
    if !is_gateway_running() {
        return Err("Gateway 服务未运行".to_string());
    }
    
    // 从配置文件读取 Gateway 配置
    let config_path = get_openclaw_config_path();
    
    if !config_path.exists() {
        return Err("配置文件不存在".to_string());
    }
    
    let content = fs::read_to_string(&config_path)
        .map_err(|e| format!("读取配置文件失败: {}", e))?;
    
    let config: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("解析配置文件失败: {}", e))?;
    
    // 读取 gateway 配置 - 参照 config.rs 中 get_gateway_config 的逻辑
    let gateway_config = config.get("gateway")
        .ok_or("配置中未找到 gateway 配置")?;
    
    // 读取端口，默认 18789
    let port = gateway_config.get("port")
        .and_then(|v| v.as_u64())
        .unwrap_or(18789) as u16;
    
    let url = format!("ws://localhost:{}", port);
    
    // 读取 Token - 路径为 /gateway/auth/token
    let token = config.pointer("/gateway/auth/token")
        .and_then(|v| v.as_str())
        .ok_or("配置中未找到 gateway token")?;
    
    info!("[Gateway] 从配置文件读取到 Gateway 配置: url={}, token已设置", url);
    
    // 尝试连接
    let client = GatewayClient::connect(&url, token).await?;
    
    // 保存连接配置
    set_gateway_connection(url.clone(), token.to_string()).await;
    
    info!("[Gateway] 自动连接成功: {}", url);
    Ok(client)
}

/// 获取或创建 Gateway 客户端
/// 
/// 如果已有连接，直接返回客户端；
/// 如果未连接，尝试自动连接一次；
/// 如果自动连接失败，返回错误。
pub async fn get_or_create_gateway_client() -> Result<GatewayClient, String> {
    // 如果已连接，直接创建客户端
    if is_gateway_connected().await {
        return create_gateway_client().await;
    }
    
    // 尝试自动连接
    match try_auto_connect().await {
        Ok(client) => Ok(client),
        Err(e) => {
            warn!("[Gateway] 自动连接失败: {}", e);
            Err("未连接到 Gateway，请先配置并连接".to_string())
        }
    }
}

pub fn get_session_store_path() -> PathBuf {
    let config_path = get_openclaw_config_path();

    if let Ok(content) = fs::read_to_string(&config_path) {
        if let Ok(config) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(store_path) = config.pointer("/session/store").and_then(|v| v.as_str()) {
                if !store_path.is_empty() {
                    return PathBuf::from(store_path);
                }
            }
        }
    }

    let config_dir = platform::get_config_dir();
    PathBuf::from(&config_dir)
        .join("agents")
        .join("main")
        .join("sessions")
        .join("sessions.json")
}
