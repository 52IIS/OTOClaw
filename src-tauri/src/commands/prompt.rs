use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct OptimizePromptRequest {
    pub prompt: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OptimizePromptResponse {
    pub optimized: String,
}

#[tauri::command]
pub async fn optimize_prompt(prompt: String) -> Result<OptimizePromptResponse, String> {
    if prompt.trim().is_empty() {
        return Err("提示词不能为空".to_string());
    }

    let optimization_prompt = format!(
        r#"请优化以下提示词，使其更加清晰、具体、结构化，并能获得更好的AI响应结果。

原始提示词:
{}

请直接输出优化后的提示词，不要包含任何解释或前缀。优化后的提示词应该:
1. 清晰明确地表达用户意图
2. 包含必要的上下文信息
3. 使用适当的格式和结构
4. 避免歧义和模糊表达
"#,
        prompt
    );

    Ok(OptimizePromptResponse {
        optimized: optimization_prompt,
    })
}
