use crate::commands::common::{load_openclaw_config, save_openclaw_config};
use crate::models::{
    AIConfigOverview, ChannelConfig, ConfiguredModel, ConfiguredProvider,
    ModelConfig, ModelCostConfig, OfficialProvider, OpenClawConfig,
    ProviderConfig, SuggestedModel,
};
use crate::utils::{file, platform, shell};
use log::{debug, error, info, warn};
use serde_json::{json, Value};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::command;

/// 获取完整配置
#[command]
pub async fn get_config() -> Result<Value, String> {
    info!("[获取配置] 读取 openclaw.json 配置...");
    let result = load_openclaw_config();
    match &result {
        Ok(_) => info!("[获取配置] ✓ 配置读取成功"),
        Err(e) => error!("[获取配置] ✗ 配置读取失败: {}", e),
    }
    result
}

/// 保存配置
#[command]
pub async fn save_config(config: Value) -> Result<String, String> {
    info!("[保存配置] 保存 openclaw.json 配置...");
    debug!(
        "[保存配置] 配置内容: {}",
        serde_json::to_string_pretty(&config).unwrap_or_default()
    );
    match save_openclaw_config(&config) {
        Ok(_) => {
            info!("[保存配置] ✓ 配置保存成功");
            Ok("配置已保存".to_string())
        }
        Err(e) => {
            error!("[保存配置] ✗ 配置保存失败: {}", e);
            Err(e)
        }
    }
}

/// 获取环境变量值
#[command]
pub async fn get_env_value(key: String) -> Result<Option<String>, String> {
    info!("[获取环境变量] 读取环境变量: {}", key);
    let env_path = platform::get_env_file_path();
    let value = file::read_env_value(&env_path, &key);
    match &value {
        Some(v) => debug!(
            "[获取环境变量] {}={} (已脱敏)",
            key,
            if v.len() > 8 { "***" } else { v }
        ),
        None => debug!("[获取环境变量] {} 不存在", key),
    }
    Ok(value)
}

/// 保存环境变量值
#[command]
pub async fn save_env_value(key: String, value: String) -> Result<String, String> {
    info!("[保存环境变量] 保存环境变量: {}", key);
    let env_path = platform::get_env_file_path();
    debug!("[保存环境变量] 环境文件路径: {}", env_path);
    
    match file::set_env_value(&env_path, &key, &value) {
        Ok(_) => {
            info!("[保存环境变量] ✓ 环境变量 {} 保存成功", key);
            Ok("环境变量已保存".to_string())
        }
        Err(e) => {
            error!("[保存环境变量] ✗ 保存失败: {}", e);
            Err(format!("保存环境变量失败: {}", e))
        }
    }
}

// ============ Gateway Token 命令 ============

/// 生成随机 token
fn generate_token() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    
    // 使用时间戳和随机数生成 token
    let random_part: u64 = (timestamp as u64) ^ 0x5DEECE66Du64;
    format!("{:016x}{:016x}{:016x}", 
        random_part, 
        random_part.wrapping_mul(0x5DEECE66Du64),
        timestamp as u64
    )
}

/// 获取或生成 Gateway Token
#[command]
pub async fn get_or_create_gateway_token() -> Result<String, String> {
    info!("[Gateway Token] 获取或创建 Gateway Token...");
    
    let mut config = load_openclaw_config()?;
    
    // 检查是否已有 token
    if let Some(token) = config
        .pointer("/gateway/auth/token")
        .and_then(|v| v.as_str())
    {
        if !token.is_empty() {
            info!("[Gateway Token] ✓ 使用现有 Token");
            return Ok(token.to_string());
        }
    }
    
    // 生成新 token
    let new_token = generate_token();
    info!("[Gateway Token] 生成新 Token: {}...", &new_token[..8]);
    
    // 确保路径存在
    if config.get("gateway").is_none() {
        config["gateway"] = json!({});
    }
    if config["gateway"].get("auth").is_none() {
        config["gateway"]["auth"] = json!({});
    }
    
    // 设置 token 和 mode（使用正确的字段名）
    config["gateway"]["auth"]["token"] = json!(new_token);
    config["gateway"]["auth"]["mode"] = json!("token");
    config["gateway"]["mode"] = json!("local");
    config["gateway"]["bind"] = json!("loopback");
    config["gateway"]["port"] = json!(18789);
    
    // 保存配置
    save_openclaw_config(&config)?;
    
    info!("[Gateway Token] ✓ Token 已保存到配置");
    Ok(new_token)
}

/// 获取 Dashboard URL（带 token）
#[command]
pub async fn get_dashboard_url() -> Result<String, String> {
    info!("[Dashboard URL] 获取 Dashboard URL...");
    
    let token = get_or_create_gateway_token().await?;
    let url = format!("http://localhost:18789?token={}", token);
    
    info!("[Dashboard URL] ✓ URL: {}...", &url[..50.min(url.len())]);
    Ok(url)
}

// ============ AI 配置相关命令 ============

/// 获取官方 Provider 列表（预设模板）
#[command]
pub async fn get_official_providers() -> Result<Vec<OfficialProvider>, String> {
    info!("[官方 Provider] 获取官方 Provider 预设列表...");

    let providers = vec![
        OfficialProvider {
            id: "anthropic".to_string(),
            name: "Anthropic Claude".to_string(),
            icon: "🟣".to_string(),
            default_base_url: Some("https://api.anthropic.com".to_string()),
            api_type: "anthropic-messages".to_string(),
            requires_api_key: true,
            docs_url: Some("https://docs.openclaw.ai/providers/anthropic".to_string()),
            suggested_models: vec![
                SuggestedModel {
                    id: "claude-opus-4-5-20251101".to_string(),
                    name: "Claude Opus 4.5".to_string(),
                    description: Some("最强大版本，适合复杂任务".to_string()),
                    context_window: Some(200000),
                    max_tokens: Some(8192),
                    recommended: true,
                },
                SuggestedModel {
                    id: "claude-sonnet-4-5-20250929".to_string(),
                    name: "Claude Sonnet 4.5".to_string(),
                    description: Some("平衡版本，性价比高".to_string()),
                    context_window: Some(200000),
                    max_tokens: Some(8192),
                    recommended: false,
                },
            ],
        },
        OfficialProvider {
            id: "openai".to_string(),
            name: "OpenAI".to_string(),
            icon: "🟢".to_string(),
            default_base_url: Some("https://api.openai.com/v1".to_string()),
            api_type: "openai-completions".to_string(),
            requires_api_key: true,
            docs_url: Some("https://docs.openclaw.ai/providers/openai".to_string()),
            suggested_models: vec![
                SuggestedModel {
                    id: "gpt-4o".to_string(),
                    name: "GPT-4o".to_string(),
                    description: Some("最新多模态模型".to_string()),
                    context_window: Some(128000),
                    max_tokens: Some(4096),
                    recommended: true,
                },
                SuggestedModel {
                    id: "gpt-4o-mini".to_string(),
                    name: "GPT-4o Mini".to_string(),
                    description: Some("快速经济版".to_string()),
                    context_window: Some(128000),
                    max_tokens: Some(4096),
                    recommended: false,
                },
            ],
        },
        OfficialProvider {
            id: "moonshot".to_string(),
            name: "Moonshot".to_string(),
            icon: "🌙".to_string(),
            default_base_url: Some("https://api.moonshot.cn/v1".to_string()),
            api_type: "openai-completions".to_string(),
            requires_api_key: true,
            docs_url: Some("https://docs.openclaw.ai/providers/moonshot".to_string()),
            suggested_models: vec![
                SuggestedModel {
                    id: "kimi-k2.5".to_string(),
                    name: "Kimi K2.5".to_string(),
                    description: Some("最新旗舰模型".to_string()),
                    context_window: Some(200000),
                    max_tokens: Some(8192),
                    recommended: true,
                },
                SuggestedModel {
                    id: "moonshot-v1-128k".to_string(),
                    name: "Moonshot 128K".to_string(),
                    description: Some("超长上下文".to_string()),
                    context_window: Some(128000),
                    max_tokens: Some(8192),
                    recommended: false,
                },
            ],
        },
        OfficialProvider {
            id: "qwen".to_string(),
            name: "Qwen (通义千问)".to_string(),
            icon: "🔮".to_string(),
            default_base_url: Some("https://dashscope.aliyuncs.com/compatible-mode/v1".to_string()),
            api_type: "openai-completions".to_string(),
            requires_api_key: true,
            docs_url: Some("https://docs.openclaw.ai/providers/qwen".to_string()),
            suggested_models: vec![
                SuggestedModel {
                    id: "qwen-max".to_string(),
                    name: "Qwen Max".to_string(),
                    description: Some("最强大版本".to_string()),
                    context_window: Some(128000),
                    max_tokens: Some(8192),
                    recommended: true,
                },
                SuggestedModel {
                    id: "qwen-plus".to_string(),
                    name: "Qwen Plus".to_string(),
                    description: Some("平衡版本".to_string()),
                    context_window: Some(128000),
                    max_tokens: Some(8192),
                    recommended: false,
                },
            ],
        },
        OfficialProvider {
            id: "deepseek".to_string(),
            name: "DeepSeek".to_string(),
            icon: "🔵".to_string(),
            default_base_url: Some("https://api.deepseek.com".to_string()),
            api_type: "openai-completions".to_string(),
            requires_api_key: true,
            docs_url: None,
            suggested_models: vec![
                SuggestedModel {
                    id: "deepseek-chat".to_string(),
                    name: "DeepSeek V3".to_string(),
                    description: Some("最新对话模型".to_string()),
                    context_window: Some(128000),
                    max_tokens: Some(8192),
                    recommended: true,
                },
                SuggestedModel {
                    id: "deepseek-reasoner".to_string(),
                    name: "DeepSeek R1".to_string(),
                    description: Some("推理增强模型".to_string()),
                    context_window: Some(128000),
                    max_tokens: Some(8192),
                    recommended: false,
                },
            ],
        },
        OfficialProvider {
            id: "glm".to_string(),
            name: "GLM (智谱)".to_string(),
            icon: "🔷".to_string(),
            default_base_url: Some("https://open.bigmodel.cn/api/paas/v4".to_string()),
            api_type: "openai-completions".to_string(),
            requires_api_key: true,
            docs_url: Some("https://docs.openclaw.ai/providers/glm".to_string()),
            suggested_models: vec![
                SuggestedModel {
                    id: "glm-4".to_string(),
                    name: "GLM-4".to_string(),
                    description: Some("最新旗舰模型".to_string()),
                    context_window: Some(128000),
                    max_tokens: Some(8192),
                    recommended: true,
                },
            ],
        },
        OfficialProvider {
            id: "minimax".to_string(),
            name: "MiniMax".to_string(),
            icon: "🟡".to_string(),
            default_base_url: Some("https://api.minimax.io/anthropic".to_string()),
            api_type: "anthropic-messages".to_string(),
            requires_api_key: true,
            docs_url: Some("https://docs.openclaw.ai/providers/minimax".to_string()),
            suggested_models: vec![
                SuggestedModel {
                    id: "minimax-m2.1".to_string(),
                    name: "MiniMax M2.1".to_string(),
                    description: Some("最新模型".to_string()),
                    context_window: Some(200000),
                    max_tokens: Some(8192),
                    recommended: true,
                },
            ],
        },
        OfficialProvider {
            id: "venice".to_string(),
            name: "Venice AI".to_string(),
            icon: "🏛️".to_string(),
            default_base_url: Some("https://api.venice.ai/api/v1".to_string()),
            api_type: "openai-completions".to_string(),
            requires_api_key: true,
            docs_url: Some("https://docs.openclaw.ai/providers/venice".to_string()),
            suggested_models: vec![
                SuggestedModel {
                    id: "llama-3.3-70b".to_string(),
                    name: "Llama 3.3 70B".to_string(),
                    description: Some("隐私优先推理".to_string()),
                    context_window: Some(128000),
                    max_tokens: Some(8192),
                    recommended: true,
                },
            ],
        },
        OfficialProvider {
            id: "openrouter".to_string(),
            name: "OpenRouter".to_string(),
            icon: "🔄".to_string(),
            default_base_url: Some("https://openrouter.ai/api/v1".to_string()),
            api_type: "openai-completions".to_string(),
            requires_api_key: true,
            docs_url: Some("https://docs.openclaw.ai/providers/openrouter".to_string()),
            suggested_models: vec![
                SuggestedModel {
                    id: "anthropic/claude-opus-4-5".to_string(),
                    name: "Claude Opus 4.5".to_string(),
                    description: Some("通过 OpenRouter 访问".to_string()),
                    context_window: Some(200000),
                    max_tokens: Some(8192),
                    recommended: true,
                },
            ],
        },
        OfficialProvider {
            id: "ollama".to_string(),
            name: "Ollama (本地)".to_string(),
            icon: "🟠".to_string(),
            default_base_url: Some("http://localhost:11434".to_string()),
            api_type: "openai-completions".to_string(),
            requires_api_key: false,
            docs_url: Some("https://docs.openclaw.ai/providers/ollama".to_string()),
            suggested_models: vec![
                SuggestedModel {
                    id: "llama3".to_string(),
                    name: "Llama 3".to_string(),
                    description: Some("本地运行".to_string()),
                    context_window: Some(8192),
                    max_tokens: Some(4096),
                    recommended: true,
                },
            ],
        },
    ];

    info!(
        "[官方 Provider] ✓ 返回 {} 个官方 Provider 预设",
        providers.len()
    );
    Ok(providers)
}

/// 获取 AI 配置概览
#[command]
pub async fn get_ai_config() -> Result<AIConfigOverview, String> {
    info!("[AI 配置] 获取 AI 配置概览...");

    let config_path = platform::get_config_file_path();
    info!("[AI 配置] 配置文件路径: {}", config_path);

    let config = load_openclaw_config()?;
    debug!("[AI 配置] 配置内容: {}", serde_json::to_string_pretty(&config).unwrap_or_default());

    // 解析主模型
    let primary_model = config
        .pointer("/agents/defaults/model/primary")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    info!("[AI 配置] 主模型: {:?}", primary_model);

    // 解析可用模型列表
    let available_models: Vec<String> = config
        .pointer("/agents/defaults/models")
        .and_then(|v| v.as_object())
        .map(|obj| obj.keys().cloned().collect())
        .unwrap_or_default();
    info!("[AI 配置] 可用模型数: {}", available_models.len());

    // 解析已配置的 Provider
    let mut configured_providers: Vec<ConfiguredProvider> = Vec::new();

    let providers_value = config.pointer("/models/providers");
    info!("[AI 配置] providers 节点存在: {}", providers_value.is_some());

    if let Some(providers) = providers_value.and_then(|v| v.as_object()) {
        info!("[AI 配置] 找到 {} 个 Provider", providers.len());
        
        for (provider_name, provider_config) in providers {
            info!("[AI 配置] 解析 Provider: {}", provider_name);
            
            let base_url = provider_config
                .get("baseUrl")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let api_key = provider_config
                .get("apiKey")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let api_key_masked = api_key.as_ref().map(|key| {
                if key.len() > 8 {
                    format!("{}...{}", &key[..4], &key[key.len() - 4..])
                } else {
                    "****".to_string()
                }
            });

            // 解析模型列表
            let models_array = provider_config.get("models").and_then(|v| v.as_array());
            info!("[AI 配置] Provider {} 的 models 数组: {:?}", provider_name, models_array.map(|a| a.len()));
            
            let models: Vec<ConfiguredModel> = models_array
                .map(|arr| {
                    arr.iter()
                        .filter_map(|m| {
                            let id = m.get("id")?.as_str()?.to_string();
                            let name = m
                                .get("name")
                                .and_then(|v| v.as_str())
                                .unwrap_or(&id)
                                .to_string();
                            let full_id = format!("{}/{}", provider_name, id);
                            let is_primary = primary_model.as_ref() == Some(&full_id);

                            info!("[AI 配置] 解析模型: {} (is_primary: {})", full_id, is_primary);

                            Some(ConfiguredModel {
                                full_id,
                                id,
                                name,
                                api_type: m.get("api").and_then(|v| v.as_str()).map(|s| s.to_string()),
                                context_window: m
                                    .get("contextWindow")
                                    .and_then(|v| v.as_u64())
                                    .map(|n| n as u32),
                                max_tokens: m
                                    .get("maxTokens")
                                    .and_then(|v| v.as_u64())
                                    .map(|n| n as u32),
                                is_primary,
                            })
                        })
                        .collect()
                })
                .unwrap_or_default();

            info!("[AI 配置] Provider {} 解析完成: {} 个模型", provider_name, models.len());

            configured_providers.push(ConfiguredProvider {
                name: provider_name.clone(),
                base_url,
                api_key_masked,
                has_api_key: api_key.is_some(),
                models,
            });
        }
    } else {
        info!("[AI 配置] 未找到 providers 配置或格式不正确");
    }

    info!(
        "[AI 配置] ✓ 最终结果 - 主模型: {:?}, {} 个 Provider, {} 个可用模型",
        primary_model,
        configured_providers.len(),
        available_models.len()
    );

    Ok(AIConfigOverview {
        primary_model,
        configured_providers,
        available_models,
    })
}

/// 添加或更新 Provider
#[command]
pub async fn save_provider(
    provider_name: String,
    base_url: String,
    api_key: Option<String>,
    api_type: String,
    models: Vec<ModelConfig>,
) -> Result<String, String> {
    info!(
        "[保存 Provider] 保存 Provider: {} ({} 个模型)",
        provider_name,
        models.len()
    );

    let mut config = load_openclaw_config()?;

    // 确保路径存在
    if config.get("models").is_none() {
        config["models"] = json!({});
    }
    if config["models"].get("providers").is_none() {
        config["models"]["providers"] = json!({});
    }
    if config.get("agents").is_none() {
        config["agents"] = json!({});
    }
    if config["agents"].get("defaults").is_none() {
        config["agents"]["defaults"] = json!({});
    }
    if config["agents"]["defaults"].get("models").is_none() {
        config["agents"]["defaults"]["models"] = json!({});
    }

    // 构建模型配置
    let models_json: Vec<Value> = models
        .iter()
        .map(|m| {
            let mut model_obj = json!({
                "id": m.id,
                "name": m.name,
                "api": m.api.clone().unwrap_or(api_type.clone()),
                "input": if m.input.is_empty() { vec!["text".to_string()] } else { m.input.clone() },
            });

            if let Some(cw) = m.context_window {
                model_obj["contextWindow"] = json!(cw);
            }
            if let Some(mt) = m.max_tokens {
                model_obj["maxTokens"] = json!(mt);
            }
            if let Some(r) = m.reasoning {
                model_obj["reasoning"] = json!(r);
            }
            if let Some(cost) = &m.cost {
                model_obj["cost"] = json!({
                    "input": cost.input,
                    "output": cost.output,
                    "cacheRead": cost.cache_read,
                    "cacheWrite": cost.cache_write,
                });
            } else {
                model_obj["cost"] = json!({
                    "input": 0,
                    "output": 0,
                    "cacheRead": 0,
                    "cacheWrite": 0,
                });
            }

            model_obj
        })
        .collect();

    // 构建 Provider 配置
    let mut provider_config = json!({
        "baseUrl": base_url,
        "models": models_json,
    });

    // 处理 API Key：如果传入了新的非空 key，使用新的；否则保留原有的
    if let Some(key) = api_key {
        if !key.is_empty() {
            // 使用新传入的 API Key
            provider_config["apiKey"] = json!(key);
            info!("[保存 Provider] 使用新的 API Key");
        } else {
            // 空字符串表示不更改，尝试保留原有的 API Key
            if let Some(existing_key) = config
                .pointer(&format!("/models/providers/{}/apiKey", provider_name))
                .and_then(|v| v.as_str())
            {
                provider_config["apiKey"] = json!(existing_key);
                info!("[保存 Provider] 保留原有的 API Key");
            }
        }
    } else {
        // None 表示不更改，尝试保留原有的 API Key
        if let Some(existing_key) = config
            .pointer(&format!("/models/providers/{}/apiKey", provider_name))
            .and_then(|v| v.as_str())
        {
            provider_config["apiKey"] = json!(existing_key);
            info!("[保存 Provider] 保留原有的 API Key");
        }
    }

    // 保存 Provider 配置
    config["models"]["providers"][&provider_name] = provider_config;

    // 将模型添加到 agents.defaults.models
    for model in &models {
        let full_id = format!("{}/{}", provider_name, model.id);
        config["agents"]["defaults"]["models"][&full_id] = json!({});
    }

    // 更新元数据
    let now = chrono::Utc::now().to_rfc3339();
    if config.get("meta").is_none() {
        config["meta"] = json!({});
    }
    config["meta"]["lastTouchedAt"] = json!(now);

    save_openclaw_config(&config)?;
    info!("[保存 Provider] ✓ Provider {} 保存成功", provider_name);

    Ok(format!("Provider {} 已保存", provider_name))
}

/// 删除 Provider
#[command]
pub async fn delete_provider(provider_name: String) -> Result<String, String> {
    info!("[删除 Provider] 删除 Provider: {}", provider_name);

    let mut config = load_openclaw_config()?;

    // 删除 Provider 配置
    if let Some(providers) = config
        .pointer_mut("/models/providers")
        .and_then(|v| v.as_object_mut())
    {
        providers.remove(&provider_name);
    }

    // 删除相关模型
    if let Some(models) = config
        .pointer_mut("/agents/defaults/models")
        .and_then(|v| v.as_object_mut())
    {
        let keys_to_remove: Vec<String> = models
            .keys()
            .filter(|k| k.starts_with(&format!("{}/", provider_name)))
            .cloned()
            .collect();

        for key in keys_to_remove {
            models.remove(&key);
        }
    }

    // 如果主模型属于该 Provider，清除主模型
    if let Some(primary) = config
        .pointer("/agents/defaults/model/primary")
        .and_then(|v| v.as_str())
    {
        if primary.starts_with(&format!("{}/", provider_name)) {
            config["agents"]["defaults"]["model"]["primary"] = json!(null);
        }
    }

    save_openclaw_config(&config)?;
    info!("[删除 Provider] ✓ Provider {} 已删除", provider_name);

    Ok(format!("Provider {} 已删除", provider_name))
}

/// 设置主模型
#[command]
pub async fn set_primary_model(model_id: String) -> Result<String, String> {
    info!("[设置主模型] 设置主模型: {}", model_id);

    let mut config = load_openclaw_config()?;

    // 确保路径存在
    if config.get("agents").is_none() {
        config["agents"] = json!({});
    }
    if config["agents"].get("defaults").is_none() {
        config["agents"]["defaults"] = json!({});
    }
    if config["agents"]["defaults"].get("model").is_none() {
        config["agents"]["defaults"]["model"] = json!({});
    }

    // 设置主模型
    config["agents"]["defaults"]["model"]["primary"] = json!(model_id);

    save_openclaw_config(&config)?;
    info!("[设置主模型] ✓ 主模型已设置为: {}", model_id);

    Ok(format!("主模型已设置为 {}", model_id))
}

/// 添加模型到可用列表
#[command]
pub async fn add_available_model(model_id: String) -> Result<String, String> {
    info!("[添加模型] 添加模型到可用列表: {}", model_id);

    let mut config = load_openclaw_config()?;

    // 确保路径存在
    if config.get("agents").is_none() {
        config["agents"] = json!({});
    }
    if config["agents"].get("defaults").is_none() {
        config["agents"]["defaults"] = json!({});
    }
    if config["agents"]["defaults"].get("models").is_none() {
        config["agents"]["defaults"]["models"] = json!({});
    }

    // 添加模型
    config["agents"]["defaults"]["models"][&model_id] = json!({});

    save_openclaw_config(&config)?;
    info!("[添加模型] ✓ 模型 {} 已添加", model_id);

    Ok(format!("模型 {} 已添加", model_id))
}

/// 从可用列表移除模型
#[command]
pub async fn remove_available_model(model_id: String) -> Result<String, String> {
    info!("[移除模型] 从可用列表移除模型: {}", model_id);

    let mut config = load_openclaw_config()?;

    if let Some(models) = config
        .pointer_mut("/agents/defaults/models")
        .and_then(|v| v.as_object_mut())
    {
        models.remove(&model_id);
    }

    save_openclaw_config(&config)?;
    info!("[移除模型] ✓ 模型 {} 已移除", model_id);

    Ok(format!("模型 {} 已移除", model_id))
}

// ============ 旧版兼容 ============

/// 获取所有支持的 AI Provider（旧版兼容）
#[command]
pub async fn get_ai_providers() -> Result<Vec<crate::models::AIProviderOption>, String> {
    info!("[AI Provider] 获取支持的 AI Provider 列表（旧版）...");

    let official = get_official_providers().await?;
    let providers: Vec<crate::models::AIProviderOption> = official
        .into_iter()
        .map(|p| crate::models::AIProviderOption {
            id: p.id,
            name: p.name,
            icon: p.icon,
            default_base_url: p.default_base_url,
            requires_api_key: p.requires_api_key,
            models: p
                .suggested_models
                .into_iter()
                .map(|m| crate::models::AIModelOption {
                    id: m.id,
                    name: m.name,
                    description: m.description,
                    recommended: m.recommended,
                })
                .collect(),
        })
        .collect();

    Ok(providers)
}

// ============ 渠道配置 ============

/// 获取渠道配置 - 从 openclaw.json 和 env 文件读取
#[command]
pub async fn get_channels_config() -> Result<Vec<ChannelConfig>, String> {
    info!("[渠道配置] 获取渠道配置列表...");
    
    let config = load_openclaw_config()?;
    let channels_obj = config.get("channels").cloned().unwrap_or(json!({}));
    let env_path = platform::get_env_file_path();
    debug!("[渠道配置] 环境文件路径: {}", env_path);
    
    let mut channels = Vec::new();
    
    // 支持的渠道类型列表及其测试字段
    let channel_types = vec![
        ("telegram", "telegram", vec!["userId"]),
        ("discord", "discord", vec!["testChannelId"]),
        ("slack", "slack", vec!["testChannelId"]),
        ("feishu", "feishu", vec!["testChatId"]),
        ("whatsapp", "whatsapp", vec![]),
        ("imessage", "imessage", vec![]),
        ("wecom", "wecom", vec!["userId"]),
        ("qq", "qq", vec!["userId"]),
        ("dingtalk", "dingtalk", vec![]),
    ];
    
    for (channel_id, channel_type, test_fields) in channel_types {
        let channel_config = channels_obj.get(channel_id);
        
        let enabled = channel_config
            .and_then(|c| c.get("enabled"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        
        // 将渠道配置转换为 HashMap
        let mut config_map: HashMap<String, Value> = if let Some(cfg) = channel_config {
            if let Some(obj) = cfg.as_object() {
                obj.iter()
                    .filter(|(k, _)| *k != "enabled") // 排除 enabled 字段
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect()
            } else {
                HashMap::new()
            }
        } else {
            HashMap::new()
        };
        
        // 从 env 文件读取测试字段
        for field in test_fields {
            let env_key = format!(
                "OPENCLAW_{}_{}",
                channel_id.to_uppercase(),
                field.to_uppercase()
            );
            if let Some(value) = file::read_env_value(&env_path, &env_key) {
                config_map.insert(field.to_string(), json!(value));
            }
        }
        
        // 判断是否已配置（有任何非空配置项）
        let has_config = !config_map.is_empty() || enabled;
        
        channels.push(ChannelConfig {
            id: channel_id.to_string(),
            channel_type: channel_type.to_string(),
            enabled: has_config,
            config: config_map,
        });
    }
    
    info!("[渠道配置] ✓ 返回 {} 个渠道配置", channels.len());
    for ch in &channels {
        debug!("[渠道配置] - {}: enabled={}", ch.id, ch.enabled);
    }
    Ok(channels)
}

/// 保存渠道配置 - 保存到 openclaw.json
#[command]
pub async fn save_channel_config(channel: ChannelConfig) -> Result<String, String> {
    info!(
        "[保存渠道配置] 保存渠道配置: {} ({})",
        channel.id, channel.channel_type
    );
    
    let mut config = load_openclaw_config()?;
    let env_path = platform::get_env_file_path();
    debug!("[保存渠道配置] 环境文件路径: {}", env_path);
    
    // 确保 channels 对象存在
    if config.get("channels").is_none() {
        config["channels"] = json!({});
    }
    
    // 确保 plugins 对象存在
    if config.get("plugins").is_none() {
        config["plugins"] = json!({
            "allow": [],
            "entries": {}
        });
    }
    if config["plugins"].get("allow").is_none() {
        config["plugins"]["allow"] = json!([]);
    }
    if config["plugins"].get("entries").is_none() {
        config["plugins"]["entries"] = json!({});
    }
    
    // 这些字段只用于测试，不保存到 openclaw.json，而是保存到 env 文件
    let test_only_fields = vec!["userId", "testChatId", "testChannelId"];
    
    // 构建渠道配置
    let mut channel_obj = json!({
        "enabled": true
    });
    
    // 添加渠道特定配置
    for (key, value) in &channel.config {
        if test_only_fields.contains(&key.as_str()) {
            // 保存到 env 文件
            let env_key = format!(
                "OPENCLAW_{}_{}",
                channel.id.to_uppercase(),
                key.to_uppercase()
            );
            if let Some(val_str) = value.as_str() {
                let _ = file::set_env_value(&env_path, &env_key, val_str);
            }
        } else {
            // 保存到 openclaw.json
            channel_obj[key] = value.clone();
        }
    }
    
    // 更新 channels 配置
    config["channels"][&channel.id] = channel_obj;
    
    // 更新 plugins.allow 数组 - 确保渠道在白名单中
    if let Some(allow_arr) = config["plugins"]["allow"].as_array_mut() {
        let channel_id_val = json!(&channel.id);
        if !allow_arr.contains(&channel_id_val) {
            allow_arr.push(channel_id_val);
        }
    }
    
    // 更新 plugins.entries - 确保插件已启用
    config["plugins"]["entries"][&channel.id] = json!({
        "enabled": true
    });
    
    // 保存配置
    info!("[保存渠道配置] 写入配置文件...");
    match save_openclaw_config(&config) {
        Ok(_) => {
            info!(
                "[保存渠道配置] ✓ {} 配置保存成功",
                channel.channel_type
            );
            Ok(format!("{} 配置已保存", channel.channel_type))
        }
        Err(e) => {
            error!("[保存渠道配置] ✗ 保存失败: {}", e);
            Err(e)
        }
    }
}

/// 清空渠道配置 - 从 openclaw.json 中删除指定渠道的配置
#[command]
pub async fn clear_channel_config(channel_id: String) -> Result<String, String> {
    info!("[清空渠道配置] 清空渠道配置: {}", channel_id);
    
    let mut config = load_openclaw_config()?;
    let env_path = platform::get_env_file_path();
    
    // 从 channels 对象中删除该渠道
    if let Some(channels) = config.get_mut("channels").and_then(|v| v.as_object_mut()) {
        channels.remove(&channel_id);
        info!("[清空渠道配置] 已从 channels 中删除: {}", channel_id);
    }
    
    // 从 plugins.allow 数组中删除
    if let Some(allow_arr) = config.pointer_mut("/plugins/allow").and_then(|v| v.as_array_mut()) {
        allow_arr.retain(|v| v.as_str() != Some(&channel_id));
        info!("[清空渠道配置] 已从 plugins.allow 中删除: {}", channel_id);
    }
    
    // 从 plugins.entries 中删除
    if let Some(entries) = config.pointer_mut("/plugins/entries").and_then(|v| v.as_object_mut()) {
        entries.remove(&channel_id);
        info!("[清空渠道配置] 已从 plugins.entries 中删除: {}", channel_id);
    }
    
    // 清除相关的环境变量
    let env_prefixes = vec![
        format!("OPENCLAW_{}_USERID", channel_id.to_uppercase()),
        format!("OPENCLAW_{}_TESTCHATID", channel_id.to_uppercase()),
        format!("OPENCLAW_{}_TESTCHANNELID", channel_id.to_uppercase()),
    ];
    for env_key in env_prefixes {
        let _ = file::remove_env_value(&env_path, &env_key);
    }
    
    // 保存配置
    match save_openclaw_config(&config) {
        Ok(_) => {
            info!("[清空渠道配置] ✓ {} 配置已清空", channel_id);
            Ok(format!("{} 配置已清空", channel_id))
        }
        Err(e) => {
            error!("[清空渠道配置] ✗ 清空失败: {}", e);
            Err(e)
        }
    }
}

// ============ 飞书插件管理 ============

/// 插件状态
#[derive(Debug, Serialize, Deserialize)]
pub struct FeishuPluginStatus {
    pub installed: bool,
    pub version: Option<String>,
    pub plugin_name: Option<String>,
}

/// 检查插件是否已安装
#[command]
pub async fn check_feishu_plugin(plugin_name: Option<String>) -> Result<FeishuPluginStatus, String> {
    let plugin_name = plugin_name.unwrap_or("@m1heng-clawd/feishu".to_string());
    let plugin_keyword = if plugin_name.contains("wecom") {
        "wecom"
    } else if plugin_name.contains("qqbot") {
        "qqbot"
    } else {
        "feishu"
    };
    
    info!("[插件检查] 检查 {} 插件安装状态...", plugin_keyword);
    
    // 执行 openclaw plugins list 命令
    match shell::run_openclaw(&["plugins", "list"]) {
        Ok(output) => {
            debug!("[插件检查] plugins list 输出: {}", output);
            
            // 查找包含关键字的行（不区分大小写）
            let lines: Vec<&str> = output.lines().collect();
            let plugin_line = lines.iter().find(|line| {
                line.to_lowercase().contains(plugin_keyword)
            });
            
            if let Some(line) = plugin_line {
                info!("[插件检查] ✓ {} 插件已安装: {}", plugin_keyword, line);
                
                // 尝试解析版本号（通常格式为 "name@version" 或 "name version"）
                let version = if line.contains('@') {
                    line.split('@').last().map(|s| s.trim().to_string())
                } else {
                    // 尝试匹配版本号模式 (如 0.1.2)
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    parts.iter()
                        .find(|p| p.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false))
                        .map(|s| s.to_string())
                };
                
                Ok(FeishuPluginStatus {
                    installed: true,
                    version,
                    plugin_name: Some(line.trim().to_string()),
                })
            } else {
                info!("[插件检查] ✗ {} 插件未安装", plugin_keyword);
                Ok(FeishuPluginStatus {
                    installed: false,
                    version: None,
                    plugin_name: None,
                })
            }
        }
        Err(e) => {
            warn!("[插件检查] 检查插件列表失败: {}", e);
            // 如果命令失败，假设插件未安装
            Ok(FeishuPluginStatus {
                installed: false,
                version: None,
                plugin_name: None,
            })
        }
    }
}

/// 安装插件
#[command]
pub async fn install_feishu_plugin(plugin_name: Option<String>) -> Result<String, String> {
    let plugin_name = plugin_name.unwrap_or("@m1heng-clawd/feishu".to_string());
    let plugin_keyword = if plugin_name.contains("wecom") {
        "企业微信"
    } else if plugin_name.contains("qqbot") {
        "QQ"
    } else {
        "飞书"
    };
    
    info!("[插件安装] 开始安装 {} 插件...", plugin_keyword);
    
    // 先检查是否已安装
    let status = check_feishu_plugin(Some(plugin_name.clone())).await?;
    if status.installed {
        info!("[插件安装] {} 插件已安装，跳过", plugin_keyword);
        return Ok(format!("{} 插件已安装: {}", plugin_keyword, status.plugin_name.unwrap_or_default()));
    }
    
    // 安装插件
    info!("[插件安装] 执行 openclaw plugins install {} ...", plugin_name);
    match shell::run_openclaw(&["plugins", "install", &plugin_name]) {
        Ok(output) => {
            info!("[插件安装] 安装输出: {}", output);
            
            // 验证安装结果
            let verify_status = check_feishu_plugin(Some(plugin_name)).await?;
            if verify_status.installed {
                info!("[插件安装] ✓ {} 插件安装成功", plugin_keyword);
                Ok(format!("{} 插件安装成功: {}", plugin_keyword, verify_status.plugin_name.unwrap_or_default()))
            } else {
                warn!("[插件安装] 安装命令执行成功但插件未找到");
                Err(format!("安装命令执行成功但插件未找到，请检查 openclaw 版本"))
            }
        }
        Err(e) => {
            error!("[插件安装] ✗ 安装失败: {}", e);
            Err(format!("安装 {} 插件失败: {}\n\n请手动执行: openclaw plugins install {}", plugin_keyword, e, plugin_name))
        }
    }
}

// ============ 聊天模块相关命令 ============

/// 智能体信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub avatar: Option<String>,
    #[serde(rename = "isDefault")]
    pub is_default: bool,
    pub model: Option<String>,
}

/// 模型信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub provider: Option<String>,
    #[serde(rename = "isDefault")]
    pub is_default: bool,
}

/// 智能体列表结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentsListResult {
    pub agents: Vec<AgentInfo>,
    #[serde(rename = "defaultId")]
    pub default_id: Option<String>,
}

/// 模型列表结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelsListResult {
    pub models: Vec<ModelInfo>,
    #[serde(rename = "defaultId")]
    pub default_id: Option<String>,
}

/// Gateway 配置（用于前端显示）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayConfigInfo {
    pub url: String,
    pub token_masked: Option<String>,
    pub token_full: Option<String>,
    pub password: Option<String>,
}

/// 对敏感字符串进行掩码处理
fn mask_sensitive_string(s: &str) -> String {
    if s.is_empty() {
        return String::new();
    }
    let len = s.len();
    if len <= 8 {
        "****".to_string()
    } else if len <= 16 {
        format!("{}****", &s[..4])
    } else {
        format!("{}****{}", &s[..4], &s[len - 4..])
    }
}

/// 获取智能体列表
#[command]
pub async fn get_agents() -> Result<AgentsListResult, String> {
    info!("[智能体] 获取智能体列表...");
    
    let config = load_openclaw_config()?;
    
    // 获取默认模型
    let default_model = config
        .pointer("/agents/defaults/model/primary")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    
    // OpenClaw 使用 "default" 作为默认智能体
    // 从配置中读取 agents.list 作为自定义智能体列表
    let mut agents: Vec<AgentInfo> = Vec::new();
    let mut default_id: Option<String> = None;
    
    // 添加默认智能体
    agents.push(AgentInfo {
        id: "default".to_string(),
        name: "默认助手".to_string(),
        description: Some("通用AI助手，使用配置的主模型进行对话".to_string()),
        avatar: Some("🤖".to_string()),
        is_default: true,
        model: default_model.clone(),
    });
    default_id = Some("default".to_string());
    
    // 尝试从配置中读取自定义智能体（使用 /agents/list 路径）
    if let Some(custom_agents) = config.pointer("/agents/list").and_then(|v| v.as_array()) {
        for agent_config in custom_agents {
            if let Some(agent_id) = agent_config.get("id").and_then(|v| v.as_str()) {
                if agent_id == "default" {
                    continue;
                }
                
                let name = agent_config
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or(agent_id)
                    .to_string();
                let description = agent_config
                    .get("description")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                let avatar = agent_config
                    .get("avatar")
                    .and_then(|v| v.as_str())
                    .or_else(|| agent_config.get("identity").and_then(|i| i.get("emoji").and_then(|v| v.as_str())))
                    .map(|s| s.to_string());
                let is_default = agent_config
                    .get("default")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                
                // 获取智能体的模型配置
                let agent_model = agent_config
                    .get("model")
                    .and_then(|m| m.get("primary").and_then(|v| v.as_str()))
                    .map(|s| s.to_string())
                    .or_else(|| default_model.clone());
                
                agents.push(AgentInfo {
                    id: agent_id.to_string(),
                    name,
                    description,
                    avatar,
                    is_default,
                    model: agent_model,
                });
                
                if is_default {
                    default_id = Some(agent_id.to_string());
                }
            }
        }
    }
    
    info!("[智能体] ✓ 返回 {} 个智能体", agents.len());
    Ok(AgentsListResult {
        agents,
        default_id,
    })
}

/// 获取模型列表
#[command]
pub async fn get_models() -> Result<ModelsListResult, String> {
    info!("[模型] 获取模型列表...");
    
    let config = load_openclaw_config()?;
    
    let mut models: Vec<ModelInfo> = Vec::new();
    let mut default_id: Option<String> = None;
    
    // 获取主模型作为默认
    if let Some(primary) = config
        .pointer("/agents/defaults/model/primary")
        .and_then(|v| v.as_str())
    {
        default_id = Some(primary.to_string());
    }
    
    // 从 providers 中读取已配置的模型
    if let Some(providers) = config.pointer("/models/providers").and_then(|v| v.as_object()) {
        for (provider_name, provider_config) in providers {
            if let Some(models_array) = provider_config.get("models").and_then(|v| v.as_array()) {
                for model_config in models_array {
                    if let Some(model_id) = model_config.get("id").and_then(|v| v.as_str()) {
                        let full_id = format!("{}/{}", provider_name, model_id);
                        let name = model_config
                            .get("name")
                            .and_then(|v| v.as_str())
                            .unwrap_or(model_id)
                            .to_string();
                        let is_default = default_id.as_ref() == Some(&full_id);
                        
                        models.push(ModelInfo {
                            id: full_id,
                            name,
                            provider: Some(provider_name.clone()),
                            is_default,
                        });
                    }
                }
            }
        }
    }
    
    // 如果没有配置任何模型，添加一些默认选项
    if models.is_empty() {
        models.push(ModelInfo {
            id: "anthropic/claude-sonnet-4-5-20250929".to_string(),
            name: "Claude Sonnet 4.5".to_string(),
            provider: Some("anthropic".to_string()),
            is_default: true,
        });
        default_id = Some("anthropic/claude-sonnet-4-5-20250929".to_string());
        
        models.push(ModelInfo {
            id: "openai/gpt-4o".to_string(),
            name: "GPT-4o".to_string(),
            provider: Some("openai".to_string()),
            is_default: false,
        });
    }
    
    info!("[模型] ✓ 返回 {} 个模型", models.len());
    Ok(ModelsListResult {
        models,
        default_id,
    })
}

/// 获取 Gateway 配置
#[command]
pub async fn get_gateway_config() -> Result<GatewayConfigInfo, String> {
    info!("[Gateway配置] 获取 Gateway 配置...");
    
    let config = load_openclaw_config()?;
    
    // Gateway URL 固定为本地地址
    let port = config
        .pointer("/gateway/port")
        .and_then(|v| v.as_u64())
        .unwrap_or(18789) as u16;
    let url = format!("ws://localhost:{}", port);
    
    // 获取 Token（完整版本，用于连接）
    let token_full = config
        .pointer("/gateway/auth/token")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    
    // Token 掩码版本（用于显示）
    let token_masked = token_full.as_ref().map(|t| mask_sensitive_string(t));
    
    // 获取密码
    let password = config
        .pointer("/gateway/auth/password")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    
    info!("[Gateway配置] ✓ URL: {}, Token: {}", url, token_masked.as_deref().unwrap_or("未设置"));
    
    Ok(GatewayConfigInfo {
        url,
        token_masked,
        token_full,
        password,
    })
}

/// 保存 Gateway 配置
#[command]
pub async fn save_gateway_config(
    url: Option<String>,
    token: Option<String>,
    password: Option<String>,
) -> Result<String, String> {
    info!("[Gateway配置] 保存 Gateway 配置...");
    
    let mut config = load_openclaw_config()?;
    
    // 确保路径存在
    if config.get("gateway").is_none() {
        config["gateway"] = json!({});
    }
    if config["gateway"].get("auth").is_none() {
        config["gateway"]["auth"] = json!({});
    }
    if config["gateway"].get("remote").is_none() {
        config["gateway"]["remote"] = json!({});
    }
    
    // 解析 URL 获取端口号（URL 仅用于前端显示，后端使用 port 字段）
    if let Some(u) = url {
        // 从 URL 中提取端口号
        if let Some(port_str) = u.split(':').last() {
            if let Ok(port) = port_str.parse::<u64>() {
                config["gateway"]["port"] = json!(port);
                info!("[Gateway配置] 设置端口: {}", port);
            }
        }
    }
    
    // 更新 Token - 同时设置 auth.token 和 remote.token
    if let Some(t) = token {
        if !t.is_empty() {
            config["gateway"]["auth"]["token"] = json!(t);
            // 设置 remote.token 以匹配 auth.token（解决 token mismatch 问题）
            config["gateway"]["remote"]["token"] = json!(t);
            info!("[Gateway配置] 设置 Token (auth + remote)");
        }
    }
    
    // 更新密码
    if let Some(p) = password {
        if !p.is_empty() {
            config["gateway"]["auth"]["password"] = json!(p);
        }
    }
    
    // 设置认证模式和绑定模式（使用正确的字段名）
    config["gateway"]["auth"]["mode"] = json!("token");
    config["gateway"]["mode"] = json!("local");
    config["gateway"]["bind"] = json!("loopback");
    
    save_openclaw_config(&config)?;
    
    info!("[Gateway配置] ✓ 配置已保存");
    Ok("Gateway 配置已保存".to_string())
}

/// 初始化 Gateway Token（如果不存在则创建）
#[command]
pub async fn init_gateway_token() -> Result<String, String> {
    info!("[Gateway Token] 初始化 Gateway Token...");
    
    // 使用现有的 get_or_create_gateway_token 逻辑
    let token = get_or_create_gateway_token().await?;
    
    // 返回掩码版本
    Ok(mask_sensitive_string(&token))
}

// ============ 企业微信多账号管理 ============

/// 企业微信账号信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WecomAccount {
    #[serde(rename = "accountId")]
    pub account_id: String,
    pub name: String,
    pub enabled: bool,
    pub configured: bool,
    #[serde(rename = "botId")]
    pub bot_id: String,
    #[serde(rename = "websocketUrl")]
    pub websocket_url: String,
    #[serde(rename = "agentConfigured")]
    pub agent_configured: bool,
    #[serde(rename = "callbackConfigured")]
    pub callback_configured: bool,
}

/// 企业微信账号配置数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WecomAccountConfig {
    #[serde(rename = "accountId")]
    pub account_id: String,
    pub name: String,
    #[serde(rename = "botId")]
    pub bot_id: String,
    pub secret: String,
    #[serde(rename = "websocketUrl")]
    pub websocket_url: String,
    pub enabled: bool,
    pub agent: WecomAgentConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WecomAgentConfig {
    #[serde(rename = "corpId")]
    pub corp_id: String,
    #[serde(rename = "corpSecret")]
    pub corp_secret: String,
    #[serde(rename = "agentId")]
    pub agent_id: String,
    pub callback: WecomCallbackConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WecomCallbackConfig {
    pub token: String,
    #[serde(rename = "encodingAESKey")]
    pub encoding_aes_key: String,
    pub path: String,
}

/// 获取企业微信账号列表
#[command]
pub async fn get_wecom_accounts() -> Result<Vec<WecomAccount>, String> {
    info!("[企业微信] 获取账号列表...");
    
    let config = load_openclaw_config()?;
    let mut accounts = Vec::new();
    
    // 获取 wecom 渠道配置
    if let Some(wecom_config) = config.pointer("/channels/wecom").and_then(|v| v.as_object()) {
        // 检查是否有多账号配置
        for (key, value) in wecom_config {
            // 跳过保留字段
            if ["enabled", "name", "botId", "secret", "websocketUrl", "agent", "callback"].contains(&key.as_str()) {
                continue;
            }
            
            // 检查是否是账号配置对象
            if let Some(account_obj) = value.as_object() {
                let bot_id = account_obj.get("botId")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let secret = account_obj.get("secret")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let websocket_url = account_obj.get("websocketUrl")
                    .and_then(|v| v.as_str())
                    .unwrap_or("wss://openws.work.weixin.qq.com")
                    .to_string();
                let enabled = account_obj.get("enabled")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true);
                let name = account_obj.get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or(key)
                    .to_string();
                
                // 检查 Agent 配置
                let agent_configured = if let Some(agent) = account_obj.get("agent").and_then(|v| v.as_object()) {
                    let corp_id = agent.get("corpId").and_then(|v| v.as_str()).unwrap_or("");
                    let corp_secret = agent.get("corpSecret").and_then(|v| v.as_str()).unwrap_or("");
                    let agent_id = agent.get("agentId").and_then(|v| v.as_str()).unwrap_or("");
                    !corp_id.is_empty() && !corp_secret.is_empty() && !agent_id.is_empty()
                } else {
                    false
                };
                
                // 检查回调配置
                let callback_configured = if let Some(agent) = account_obj.get("agent").and_then(|v| v.as_object()) {
                    if let Some(callback) = agent.get("callback").and_then(|v| v.as_object()) {
                        let token = callback.get("token").and_then(|v| v.as_str()).unwrap_or("");
                        let aes_key = callback.get("encodingAESKey").and_then(|v| v.as_str()).unwrap_or("");
                        !token.is_empty() && !aes_key.is_empty()
                    } else {
                        false
                    }
                } else {
                    false
                };
                
                accounts.push(WecomAccount {
                    account_id: key.clone(),
                    name,
                    enabled,
                    configured: !bot_id.is_empty() && !secret.is_empty(),
                    bot_id,
                    websocket_url,
                    agent_configured,
                    callback_configured,
                });
            }
        }
        
        // 如果没有多账号配置，检查默认配置
        if accounts.is_empty() {
            let bot_id = wecom_config.get("botId")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let secret = wecom_config.get("secret")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let websocket_url = wecom_config.get("websocketUrl")
                .and_then(|v| v.as_str())
                .unwrap_or("wss://openws.work.weixin.qq.com")
                .to_string();
            let enabled = wecom_config.get("enabled")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let name = wecom_config.get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("默认账号")
                .to_string();
            
            // 检查 Agent 配置
            let agent_configured = if let Some(agent) = wecom_config.get("agent").and_then(|v| v.as_object()) {
                let corp_id = agent.get("corpId").and_then(|v| v.as_str()).unwrap_or("");
                let corp_secret = agent.get("corpSecret").and_then(|v| v.as_str()).unwrap_or("");
                let agent_id = agent.get("agentId").and_then(|v| v.as_str()).unwrap_or("");
                !corp_id.is_empty() && !corp_secret.is_empty() && !agent_id.is_empty()
            } else {
                false
            };
            
            // 检查回调配置
            let callback_configured = if let Some(agent) = wecom_config.get("agent").and_then(|v| v.as_object()) {
                if let Some(callback) = agent.get("callback").and_then(|v| v.as_object()) {
                    let token = callback.get("token").and_then(|v| v.as_str()).unwrap_or("");
                    let aes_key = callback.get("encodingAESKey").and_then(|v| v.as_str()).unwrap_or("");
                    !token.is_empty() && !aes_key.is_empty()
                } else {
                    false
                }
            } else {
                false
            };
            
            accounts.push(WecomAccount {
                account_id: "default".to_string(),
                name,
                enabled,
                configured: !bot_id.is_empty() && !secret.is_empty(),
                bot_id,
                websocket_url,
                agent_configured,
                callback_configured,
            });
        }
    }
    
    info!("[企业微信] ✓ 返回 {} 个账号", accounts.len());
    Ok(accounts)
}

/// 获取企业微信账号详情
#[command]
pub async fn get_wecom_account(account_id: String) -> Result<WecomAccountConfig, String> {
    info!("[企业微信] 获取账号详情: {}", account_id);
    
    let config = load_openclaw_config()?;
    
    let wecom_config = config.pointer("/channels/wecom")
        .and_then(|v| v.as_object())
        .ok_or("企业微信渠道未配置")?;
    
    // 尝试从多账号配置中获取
    let account_obj = if account_id == "default" {
        wecom_config
    } else {
        wecom_config.get(&account_id)
            .and_then(|v| v.as_object())
            .unwrap_or(wecom_config)
    };
    
    let bot_id = account_obj.get("botId")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let secret = account_obj.get("secret")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let websocket_url = account_obj.get("websocketUrl")
        .and_then(|v| v.as_str())
        .unwrap_or("wss://openws.work.weixin.qq.com")
        .to_string();
    let enabled = account_obj.get("enabled")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    let name = account_obj.get("name")
        .and_then(|v| v.as_str())
        .unwrap_or(&account_id)
        .to_string();
    
    // 解析 Agent 配置
    let agent = if let Some(agent_obj) = account_obj.get("agent").and_then(|v| v.as_object()) {
        let callback = if let Some(callback_obj) = agent_obj.get("callback").and_then(|v| v.as_object()) {
            WecomCallbackConfig {
                token: callback_obj.get("token").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                encoding_aes_key: callback_obj.get("encodingAESKey").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                path: callback_obj.get("path").and_then(|v| v.as_str()).unwrap_or("/api/channels/wecom/callback").to_string(),
            }
        } else {
            WecomCallbackConfig {
                token: String::new(),
                encoding_aes_key: String::new(),
                path: "/api/channels/wecom/callback".to_string(),
            }
        };
        
        WecomAgentConfig {
            corp_id: agent_obj.get("corpId").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            corp_secret: agent_obj.get("corpSecret").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            agent_id: agent_obj.get("agentId").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            callback,
        }
    } else {
        WecomAgentConfig {
            corp_id: String::new(),
            corp_secret: String::new(),
            agent_id: String::new(),
            callback: WecomCallbackConfig {
                token: String::new(),
                encoding_aes_key: String::new(),
                path: "/api/channels/wecom/callback".to_string(),
            },
        }
    };
    
    Ok(WecomAccountConfig {
        account_id: account_id.clone(),
        name,
        bot_id,
        secret,
        websocket_url,
        enabled,
        agent,
    })
}

/// 创建企业微信账号
#[command]
pub async fn create_wecom_account(config: WecomAccountConfig) -> Result<String, String> {
    info!("[企业微信] 创建账号: {}", config.account_id);
    
    let mut openclaw_config = load_openclaw_config()?;
    
    // 确保 channels.wecom 存在
    if openclaw_config.get("channels").is_none() {
        openclaw_config["channels"] = json!({});
    }
    if openclaw_config["channels"].get("wecom").is_none() {
        openclaw_config["channels"]["wecom"] = json!({});
    }
    
    // 构建账号配置
    let account_obj = json!({
        "name": config.name,
        "botId": config.bot_id,
        "secret": config.secret,
        "websocketUrl": config.websocket_url,
        "enabled": config.enabled,
        "agent": {
            "corpId": config.agent.corp_id,
            "corpSecret": config.agent.corp_secret,
            "agentId": config.agent.agent_id,
            "callback": {
                "token": config.agent.callback.token,
                "encodingAESKey": config.agent.callback.encoding_aes_key,
                "path": config.agent.callback.path,
            }
        }
    });
    
    // 保存账号配置
    openclaw_config["channels"]["wecom"][&config.account_id] = account_obj;
    
    // 确保 plugins 配置正确
    if openclaw_config.get("plugins").is_none() {
        openclaw_config["plugins"] = json!({
            "allow": [],
            "entries": {}
        });
    }
    if openclaw_config["plugins"].get("allow").is_none() {
        openclaw_config["plugins"]["allow"] = json!([]);
    }
    if openclaw_config["plugins"].get("entries").is_none() {
        openclaw_config["plugins"]["entries"] = json!({});
    }
    
    // 确保 wecom 在白名单中
    if let Some(allow_arr) = openclaw_config["plugins"]["allow"].as_array_mut() {
        if !allow_arr.contains(&json!("wecom")) {
            allow_arr.push(json!("wecom"));
        }
    }
    
    // 确保 wecom 插件已启用
    openclaw_config["plugins"]["entries"]["wecom"] = json!({
        "enabled": true
    });
    
    save_openclaw_config(&openclaw_config)?;
    
    info!("[企业微信] ✓ 账号 {} 创建成功", config.account_id);
    Ok(format!("账号 {} 创建成功", config.account_id))
}

/// 更新企业微信账号
#[command]
pub async fn update_wecom_account(account_id: String, config: WecomAccountConfig) -> Result<String, String> {
    info!("[企业微信] 更新账号: {}", account_id);
    
    let mut openclaw_config = load_openclaw_config()?;
    
    // 检查账号是否存在
    if openclaw_config.pointer(&format!("/channels/wecom/{}", account_id)).is_none() {
        return Err(format!("账号 {} 不存在", account_id));
    }
    
    // 构建账号配置
    let account_obj = json!({
        "name": config.name,
        "botId": config.bot_id,
        "secret": config.secret,
        "websocketUrl": config.websocket_url,
        "enabled": config.enabled,
        "agent": {
            "corpId": config.agent.corp_id,
            "corpSecret": config.agent.corp_secret,
            "agentId": config.agent.agent_id,
            "callback": {
                "token": config.agent.callback.token,
                "encodingAESKey": config.agent.callback.encoding_aes_key,
                "path": config.agent.callback.path,
            }
        }
    });
    
    // 更新账号配置
    openclaw_config["channels"]["wecom"][&account_id] = account_obj;
    
    save_openclaw_config(&openclaw_config)?;
    
    info!("[企业微信] ✓ 账号 {} 更新成功", account_id);
    Ok(format!("账号 {} 更新成功", account_id))
}

/// 删除企业微信账号
#[command]
pub async fn delete_wecom_account(account_id: String) -> Result<String, String> {
    info!("[企业微信] 删除账号: {}", account_id);
    
    let mut openclaw_config = load_openclaw_config()?;
    
    // 从配置中删除账号
    if let Some(wecom) = openclaw_config.get_mut("channels").and_then(|c| c.get_mut("wecom")).and_then(|w| w.as_object_mut()) {
        wecom.remove(&account_id);
    }
    
    save_openclaw_config(&openclaw_config)?;
    
    info!("[企业微信] ✓ 账号 {} 已删除", account_id);
    Ok(format!("账号 {} 已删除", account_id))
}
