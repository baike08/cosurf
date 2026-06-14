//! AI 流式对话核心
//!
//! 从 src-tauri/src/ai/stream.rs 迁移。
//! 核心改动: 将 Tauri emit 替换为 N-API ThreadsafeFunction 回调。

use futures::StreamExt;
use reqwest::Client;
use reqwest_eventsource::{Event, RequestBuilderExt};
use serde_json::json;
use tracing::{error, info, warn};

use crate::ai::provider::{
    build_headers, get_api_url, ChatMessage, ModelConfig, StreamDelta, StreamRequest,
    supports_tool_calling,
};
use crate::ai::tools::{normalize_tool_call_format, ToolCall, ToolResult, execute_builtin_tool};
use crate::ai::{is_cancelled, reset_cancel};
use crate::error::{AppError, AppResult};

/// 流式回调接口（通过 ThreadsafeFunction 从 Rust 线程安全地调用 Node.js）
pub struct StreamCallbacks {
    pub on_chunk: Option<napi::threadsafe_function::ThreadsafeFunction<ChunkEvent, napi::threadsafe_function::ErrorStrategy::Fatal>>,
    pub on_tool_call: Option<napi::threadsafe_function::ThreadsafeFunction<ToolCallEvent, napi::threadsafe_function::ErrorStrategy::Fatal>>,
    pub on_tool_result: Option<napi::threadsafe_function::ThreadsafeFunction<ToolResultEvent, napi::threadsafe_function::ErrorStrategy::Fatal>>,
    pub on_electron_bridge: Option<napi::threadsafe_function::ThreadsafeFunction<ElectronBridgeEvent, napi::threadsafe_function::ErrorStrategy::Fatal>>,
    pub on_error: Option<napi::threadsafe_function::ThreadsafeFunction<String, napi::threadsafe_function::ErrorStrategy::Fatal>>,
}

#[napi(object)]
#[derive(Debug, Clone)]
pub struct ChunkEvent {
    pub conversation_id: String,
    pub message_id: String,
    pub delta: String,
    pub is_thinking: bool,
    pub done: bool,
}

#[napi(object)]
#[derive(Debug, Clone)]
pub struct ToolCallEvent {
    pub conversation_id: String,
    pub message_id: String,
    pub tool_name: String,
    pub arguments: serde_json::Value,
}

#[napi(object)]
#[derive(Debug, Clone)]
pub struct ToolResultEvent {
    pub conversation_id: String,
    pub message_id: String,
    pub tool_name: String,
    pub output: String,
    pub success: bool,
}

#[napi(object)]
#[derive(Debug, Clone)]
pub struct ElectronBridgeEvent {
    pub conversation_id: String,
    pub message_id: String,
    pub tool_call_json: String,  // 改为字符串，避免序列化问题
}

impl StreamCallbacks {
    pub fn emit_chunk(&self, conv_id: &str, msg_id: &str, delta: &str, is_thinking: bool, done: bool) {
        if let Some(ref cb) = self.on_chunk {
            let event = ChunkEvent {
                conversation_id: conv_id.to_string(),
                message_id: msg_id.to_string(),
                delta: delta.to_string(),
                is_thinking,
                done,
            };
            let _ = cb.call(event, napi::threadsafe_function::ThreadsafeFunctionCallMode::NonBlocking);
        }
    }

    pub fn emit_tool_call_start(&self, conv_id: &str, msg_id: &str, tool_call: &ToolCall) {
        if let Some(ref cb) = self.on_tool_call {
            let event = ToolCallEvent {
                conversation_id: conv_id.to_string(),
                message_id: msg_id.to_string(),
                tool_name: tool_call.name.clone(),
                arguments: tool_call.arguments.clone(),
            };
            let _ = cb.call(event, napi::threadsafe_function::ThreadsafeFunctionCallMode::NonBlocking);
        }
    }

    pub fn emit_tool_result(&self, conv_id: &str, msg_id: &str, tool_call: &ToolCall, result: &ToolResult) {
        if let Some(ref cb) = self.on_tool_result {
            let event = ToolResultEvent {
                conversation_id: conv_id.to_string(),
                message_id: msg_id.to_string(),
                tool_name: tool_call.name.clone(),
                output: result.output.clone(),
                success: result.success,
            };
            let _ = cb.call(event, napi::threadsafe_function::ThreadsafeFunctionCallMode::NonBlocking);
        }
    }

    pub fn emit_error(&self, _conv_id: &str, error_msg: &str) {
        if let Some(ref cb) = self.on_error {
            let _ = cb.call(error_msg.to_string(), napi::threadsafe_function::ThreadsafeFunctionCallMode::NonBlocking);
        }
    }

    pub fn emit_electron_bridge(&self, conv_id: &str, msg_id: &str, tool_call: &serde_json::Value) {
        if let Some(ref cb) = self.on_electron_bridge {
            let tool_call_json = serde_json::to_string(tool_call).unwrap_or_else(|_| "{}".to_string());
            let event = ElectronBridgeEvent {
                conversation_id: conv_id.to_string(),
                message_id: msg_id.to_string(),
                tool_call_json,
            };
            let _ = cb.call(event, napi::threadsafe_function::ThreadsafeFunctionCallMode::NonBlocking);
        }
    }
}

/// 流式聊天完成（Agent Loop 模式）
pub async fn stream_chat_completion(
    config: &ModelConfig,
    messages: Vec<ChatMessage>,
    conversation_id: &str,
    message_id: &str,
    callbacks: &StreamCallbacks,
    skills_schemas: Vec<serde_json::Value>,
) -> AppResult<()> {
    info!("🤖 Starting Agent Loop for conversation {}", conversation_id);

    let mut current_messages = messages;
    let mut iteration = 0;
    let max_iterations = 30;

    // 重复调用检测
    let mut last_tool_signatures: Vec<String> = Vec::new();
    let mut repeat_count = 0u32;

    loop {
        iteration += 1;
        info!("🔄 Agent Loop iteration {}/{}", iteration, max_iterations);

        let tool_calls_result = stream_single_turn(
            config,
            current_messages.clone(),
            conversation_id,
            message_id,
            callbacks,
            &skills_schemas,
        ).await?;

        if tool_calls_result.is_empty() {
            info!("✅ No more tool calls, agent loop completed after {} iterations", iteration);
            break;
        }

        info!("🔧 Found {} tool calls in iteration {}", tool_calls_result.len(), iteration);

        // 重复调用检测
        let current_signatures = compute_tool_signatures(&tool_calls_result);
        if current_signatures == last_tool_signatures {
            repeat_count += 1;
            warn!("⚠️  Detected repeated tool calls (consecutive #{})", repeat_count);

            if repeat_count >= 2 {
                warn!("🛑 LLM looping, injecting force-stop nudge");

                for tc in &tool_calls_result {
                    let normalized_tc = normalize_tool_call_format(tc);
                    if let Ok(tool_call) = serde_json::from_value::<ToolCall>(normalized_tc) {
                        let nudge_msg = ChatMessage {
                            role: "tool".to_string(),
                            content: format!(
                                "[系统提示] 工具 '{}' 已在之前的迭代中成功执行过。请勿重复调用，直接根据已有结果回答。",
                                tool_call.name
                            ),
                            name: Some(tool_call.name.clone()),
                            tool_call_id: Some(tool_call.id.clone()),
                        };
                        current_messages.push(nudge_msg);
                    }
                }

                current_messages.push(ChatMessage {
                    role: "user".to_string(),
                    content: "[系统] 你已拥有足够的工具执行结果，请直接回答用户的问题。".to_string(),
                    name: None,
                    tool_call_id: None,
                });

                if repeat_count >= 3 {
                    warn!("🛑 Force stopping after {} consecutive repeats", repeat_count);
                    callbacks.emit_chunk(conversation_id, message_id, "", false, true);
                    break;
                }

                last_tool_signatures = vec!["__force_break__".to_string()];
                continue;
            }
        } else {
            repeat_count = 0;
            last_tool_signatures = current_signatures;
        }

        // 并行执行工具
        let mut futures = Vec::new();
        for tc in &tool_calls_result {
            let normalized_tc = normalize_tool_call_format(tc);
            if let Ok(tool_call) = serde_json::from_value::<ToolCall>(normalized_tc) {
                callbacks.emit_tool_call_start(conversation_id, message_id, &tool_call);
                let tc_clone = tool_call.clone();
                let cbs = callbacks.clone();
                let conv_id = conversation_id.to_string();
                let msg_id = message_id.to_string();
                futures.push(async move {
                    match execute_tool(&tc_clone).await {
                        Ok(result) => {
                            // 检查是否需要 Electron 桥接
                            if result.output.starts_with("__ELECTRON_BRIDGE_REQUIRED__:") {
                                let tool_call_json = result.output.strip_prefix("__ELECTRON_BRIDGE_REQUIRED__:").unwrap();
                                if let Ok(tool_call_value) = serde_json::from_str::<serde_json::Value>(tool_call_json) {
                                    // 通知 Electron 主进程执行
                                    cbs.emit_electron_bridge(&conv_id, &msg_id, &tool_call_value);
                                    // 等待 Electron 执行结果（这里简化处理，实际应该等待回调）
                                    // TODO: 实现异步等待 Electron 返回结果
                                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                                    let fake = ToolResult {
                                        tool_call_id: tc_clone.id.clone(),
                                        output: format!("Electron bridge tool '{}' executed (placeholder)", tc_clone.name),
                                        success: true,
                                    };
                                    Ok((tc_clone, fake))
                                } else {
                                    let fake = ToolResult {
                                        tool_call_id: tc_clone.id.clone(),
                                        output: "Failed to parse tool call JSON".to_string(),
                                        success: false,
                                    };
                                    Ok((tc_clone, fake))
                                }
                            } else {
                                Ok((tc_clone, result))
                            }
                        }
                        Err(e) => {
                            let fake = ToolResult {
                                tool_call_id: tc_clone.id.clone(),
                                output: format!("工具执行失败: {}", e),
                                success: false,
                            };
                            Ok((tc_clone, fake))
                        }
                    }
                });
            }
        }

        let tool_results: Vec<std::result::Result<(ToolCall, ToolResult), AppError>> =
            futures::future::join_all(futures).await;

        for result in &tool_results {
            if let Ok((tool_call, tool_result)) = result {
                callbacks.emit_tool_result(conversation_id, message_id, tool_call, tool_result);

                let tool_msg = ChatMessage {
                    role: "tool".to_string(),
                    content: tool_result.output.clone(),
                    name: Some(tool_call.name.clone()),
                    tool_call_id: Some(tool_call.id.clone()),
                };
                current_messages.push(tool_msg);
            }
        }

        if iteration >= max_iterations {
            warn!("⚠️  Reached max iterations, stopping agent loop");
            callbacks.emit_chunk(conversation_id, message_id, "", false, true);
            break;
        }
    }

    info!("🏁 Agent Loop completed after {} iterations", iteration);
    Ok(())
}

/// 计算工具调用签名
fn compute_tool_signatures(tool_calls: &[serde_json::Value]) -> Vec<String> {
    tool_calls.iter().map(|tc| {
        let normalized = normalize_tool_call_format(tc);
        let name = normalized.get("name").and_then(|v| v.as_str()).unwrap_or("unknown");
        let args = normalized.get("arguments").map(|v| v.to_string()).unwrap_or_else(|| "{}".to_string());
        format!("{}({})", name, args)
    }).collect()
}

/// 执行单轮流式对话
async fn stream_single_turn(
    config: &ModelConfig,
    messages: Vec<ChatMessage>,
    conversation_id: &str,
    message_id: &str,
    callbacks: &StreamCallbacks,
    skills_schemas: &[serde_json::Value],
) -> AppResult<Vec<serde_json::Value>> {
    let url = get_api_url(config)?;
    let headers = build_headers(config)?;

    let body = build_stream_request(config, &messages, skills_schemas);

    let client = Client::new();
    let mut request = client.post(&url);
    for (key, value) in &headers {
        request = request.header(key.as_str(), value.as_str());
    }

    let body_json = serde_json::to_string(&body)?;

    info!("=== LLM Request: model={}, messages={} ===", config.model_id, body.messages.len());
    if let Some(ref tools) = body.tools {
        info!("Tools count: {}", tools.len());
    }

    let mut event_source = request
        .body(body_json)
        .eventsource()
        .map_err(|e| AppError::AiProvider(format!("Failed to create event source: {}", e)))?;

    let mut full_response = String::new();
    let mut full_thinking = String::new();
    let mut thinking_started = false;
    let mut sse_chunk_count = 0;
    let mut accumulating_tool_calls: std::collections::HashMap<usize, serde_json::Map<String, serde_json::Value>> = std::collections::HashMap::new();

    while let Some(event) = event_source.next().await {
        // 检查取消
        if is_cancelled() {
            info!("Stream cancelled by user");
            callbacks.emit_chunk(conversation_id, message_id, "", false, true);
            reset_cancel();
            break;
        }

        match event {
            Ok(Event::Open) => {
                info!("SSE connection opened");
            }
            Ok(Event::Message(msg)) => {
                sse_chunk_count += 1;
                if msg.data == "[DONE]" {
                    let final_tool_calls: Vec<serde_json::Value> = accumulating_tool_calls
                        .values()
                        .map(|m| serde_json::Value::Object(m.clone()))
                        .collect();

                    if !final_tool_calls.is_empty() {
                        info!("🔧 Found {} tool calls, returning to Agent Loop", final_tool_calls.len());
                        // 发送暂停 chunk（done=false）
                        callbacks.emit_chunk(conversation_id, message_id, "", false, false);
                        return Ok(final_tool_calls);
                    } else {
                        info!("ℹ️  No tool calls, single turn completed");
                        callbacks.emit_chunk(conversation_id, message_id, "", false, true);
                        return Ok(vec![]);
                    }
                }

                match serde_json::from_str::<StreamDelta>(&msg.data) {
                    Ok(delta) => {
                        if let Some(choice) = delta.choices.first() {
                            // 处理工具调用 delta
                            if let Some(ref tc) = choice.delta.tool_calls {
                                for tool_call_delta in tc {
                                    let index = tool_call_delta.index;
                                    let acc = accumulating_tool_calls.entry(index).or_insert_with(serde_json::Map::new);

                                    if let Some(ref id) = tool_call_delta.id {
                                        if !id.is_empty() {
                                            acc.insert("id".to_string(), serde_json::Value::String(id.clone()));
                                        }
                                    }
                                    if let Some(ref call_type) = tool_call_delta.call_type {
                                        acc.insert("type".to_string(), serde_json::Value::String(call_type.clone()));
                                    }
                                    if let Some(ref func_delta) = tool_call_delta.function {
                                        let func_obj = acc.entry("function".to_string())
                                            .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()))
                                            .as_object_mut().unwrap();

                                        if let Some(ref name) = func_delta.name {
                                            if !name.is_empty() {
                                                func_obj.insert("name".to_string(), serde_json::Value::String(name.clone()));
                                            }
                                        }
                                        if let Some(ref args) = func_delta.arguments {
                                            let existing = match func_obj.get("arguments") {
                                                Some(serde_json::Value::String(s)) => s.clone(),
                                                _ => String::new(),
                                            };
                                            func_obj.insert("arguments".to_string(), serde_json::Value::String(format!("{}{}", existing, args)));
                                        }
                                    }
                                }
                            }

                            // 处理 reasoning/thinking 内容
                            if let Some(reasoning) = &choice.delta.reasoning_content {
                                if !reasoning.is_empty() {
                                    full_thinking.push_str(reasoning);
                                    if !thinking_started {
                                        thinking_started = true;
                                        callbacks.emit_chunk(conversation_id, message_id, "", true, false);
                                    }
                                    callbacks.emit_chunk(conversation_id, message_id, reasoning, true, false);
                                    // 保存到数据库
                                    save_chunk_to_db(message_id, reasoning, true);
                                }
                            }

                            // 处理文本内容
                            if let Some(content) = &choice.delta.content {
                                if !content.is_empty() {
                                    full_response.push_str(content);
                                    callbacks.emit_chunk(conversation_id, message_id, content, false, false);
                                    save_chunk_to_db(message_id, content, false);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        if !msg.data.trim().is_empty() {
                            warn!("Failed to parse SSE delta: {} - data: {}", e, &msg.data);
                        }
                    }
                }
            }
            Err(e) => {
                error!("SSE stream error: {}", e);
                let error_msg = format!(
                    "AI 服务错误: {}\n\n请检查:\n1. 网络连接\n2. API Key\n3. Base URL: {}\n4. 模型: {}",
                    e,
                    get_api_url(config).unwrap_or_default(),
                    config.model_id
                );
                callbacks.emit_error(conversation_id, &error_msg);
                callbacks.emit_chunk(conversation_id, message_id, "", false, true);
                return Err(AppError::AiProvider(format!("Stream error: {}", e)));
            }
        }
    }

    event_source.close();

    // 检查未完成的 tool calls
    let orphan_tool_calls: Vec<serde_json::Value> = accumulating_tool_calls
        .values()
        .map(|m| serde_json::Value::Object(m.clone()))
        .collect();

    if !orphan_tool_calls.is_empty() {
        warn!("⚠️  SSE ended without [DONE], but {} tool calls accumulated", orphan_tool_calls.len());
        return Ok(orphan_tool_calls);
    }

    info!("📡 SSE stream closed: {} chunks, {} thinking chars, {} response chars",
        sse_chunk_count, full_thinking.len(), full_response.len());

    callbacks.emit_chunk(conversation_id, message_id, "", false, true);
    Ok(vec![])
}

/// 构建流式请求
fn build_stream_request(config: &ModelConfig, messages: &[ChatMessage], skills_schemas: &[serde_json::Value]) -> StreamRequest {
    let mut req = StreamRequest {
        model: config.model_id.clone(),
        messages: messages.to_vec(),
        temperature: config.temperature,
        top_p: config.top_p,
        max_tokens: config.max_tokens,
        stream: true,
        tools: None,
        tool_choice: None,
    };

    if supports_tool_calling(&config.provider) {
        let mut tools = crate::ai::tools::get_builtin_tools_schemas();
        tracing::info!("🔧 Built-in tools count: {}", tools.len());
        
        tools.extend(skills_schemas.to_vec());
        tracing::info!("📚 Skills tools count: {} (total: {})", skills_schemas.len(), tools.len());
        
        // 添加 MCP 工具 schemas
        let mcp_schemas = crate::ai::agent::get_mcp_schemas();
        tracing::info!("🌐 MCP tools count: {} (total: {})", mcp_schemas.len(), tools.len() + mcp_schemas.len());
        tools.extend(mcp_schemas.clone());
        
        if !tools.is_empty() {
            info!("✅ Injecting {} total tools for model {} (builtin={}, skills={}, mcp={})", 
                tools.len(), config.model_id, 
                crate::ai::tools::get_builtin_tools_schemas().len(),
                skills_schemas.len(),
                mcp_schemas.len());
            req.tools = Some(tools);
            req.tool_choice = Some(json!("auto"));
        } else {
            warn!("⚠️ No tools available for model {}", config.model_id);
        }
    } else {
        warn!("⚠️ Model {} does not support tool calling", config.model_id);
    }

    req
}

/// 保存流式内容到数据库
fn save_chunk_to_db(message_id: &str, delta: &str, is_thinking: bool) {
    // 直接调用 native DB 函数
    let _ = crate::db::db_append_message_content(
        message_id.to_string(),
        delta.to_string(),
        is_thinking,
    );
}

/// 执行工具调用
async fn execute_tool(tool_call: &ToolCall) -> AppResult<ToolResult> {
    execute_builtin_tool(tool_call).await
}

/// 生成对话标题（使用 LLM 非流式调用）
pub async fn generate_title(content: &str, config: &ModelConfig) -> AppResult<String> {
    let url = get_api_url(config)?;
    let headers = build_headers(config)?;

    let messages = vec![
        ChatMessage {
            role: "system".to_string(),
            content: "你是一个标题生成器。根据用户的消息，生成一个简短的对话标题（不超过20个字）。只返回标题文本，不要加引号或其他格式。".to_string(),
            name: None,
            tool_call_id: None,
        },
        ChatMessage {
            role: "user".to_string(),
            content: content.to_string(),
            name: None,
            tool_call_id: None,
        },
    ];

    let body = json!({
        "model": config.model_id,
        "messages": messages,
        "temperature": 0.3,
        "max_tokens": 50,
        "stream": false,
    });

    let client = Client::new();
    let mut request = client.post(&url);
    for (key, value) in &headers {
        request = request.header(key.as_str(), value.as_str());
    }

    let response = request.json(&body).send().await
        .map_err(|e| AppError::AiProvider(format!("Title generation request failed: {}", e)))?;

    let response_json: serde_json::Value = response.json().await
        .map_err(|e| AppError::AiProvider(format!("Failed to parse title response: {}", e)))?;

    let title = response_json
        .get("choices")
        .and_then(|c| c.as_array())
        .and_then(|c| c.first())
        .and_then(|c| c.get("message"))
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_str())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "New Conversation".to_string());

    Ok(title)
}
