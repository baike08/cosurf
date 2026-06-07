use serde::{Deserialize, Serialize};

use crate::db::settings::ModelConfig;
use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub temperature: f64,
    pub top_p: f64,
    pub max_tokens: i64,
    pub stream: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatChoice {
    pub message: ChatMessage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub choices: Vec<ChatChoice>,
}

// ========== 流式响应类型（支持工具调用） ==========

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamDelta {
    pub choices: Vec<StreamChoice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChoice {
    pub delta: DeltaContent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeltaContent {
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub reasoning_content: Option<String>,
    #[serde(default)]
    pub tool_calls: Option<Vec<ToolCallDelta>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallDelta {
    pub index: usize,
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub call_type: Option<String>,
    pub function: Option<FunctionDelta>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDelta {
    pub name: Option<String>,
    pub arguments: Option<String>,
}

// ========== 工具调用结果类型 ==========

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallResult {
    pub id: String,
    #[serde(rename = "type")]
    pub call_type: String,
    pub function: FunctionCall,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
}

// ========== 辅助函数 ==========

pub fn build_chat_request(config: &ModelConfig, messages: Vec<ChatMessage>) -> ChatRequest {
    ChatRequest {
        model: config.model_id.clone(),
        messages,
        temperature: config.temperature,
        top_p: config.top_p,
        max_tokens: config.max_tokens,
        stream: true,
    }
}

pub fn get_api_url(config: &ModelConfig) -> AppResult<String> {
    let base_url = config
        .base_url
        .as_deref()
        .ok_or_else(|| AppError::Config("Model base_url is not configured".into()))?;

    match config.provider.as_str() {
        "anthropic" => Ok(format!("{}/messages", base_url.trim_end_matches('/'))),
        _ => Ok(format!(
            "{}/chat/completions",
            base_url.trim_end_matches('/')
        )),
    }
}

pub fn build_headers(config: &ModelConfig) -> AppResult<Vec<(String, String)>> {
    let mut headers = vec![("Content-Type".into(), "application/json".into())];

    if let Some(api_key) = &config.api_key {
        match config.provider.as_str() {
            "anthropic" => {
                headers.push(("x-api-key".into(), api_key.clone()));
                headers.push(("anthropic-version".into(), "2023-06-01".into()));
                headers.push(("anthropic-beta".into(), "tools-2024-05-16".into()));
            }
            _ => {
                headers.push(("Authorization".into(), format!("Bearer {}", api_key)));
            }
        }
    }

    Ok(headers)
}
