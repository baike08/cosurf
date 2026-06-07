use futures::StreamExt;
use reqwest::Client;
use reqwest_eventsource::{Event, RequestBuilderExt};
use serde_json::json;
use tauri::{AppHandle, Emitter, Listener, Manager};
use tracing::{error, info, warn};

use crate::ai::provider::{build_chat_request, build_headers, get_api_url, ChatMessage, StreamDelta};
use crate::ai::tools::{get_available_tools_schemas_async, ToolCall, ToolResult};
use crate::db::messages::StreamChunk;
use crate::db::settings::ModelConfig;
use crate::error::{AppError, AppResult};
use crate::state::AppState;

/// 转换工具调用格式：将嵌套的 function.name/function.arguments 提升到顶层
/// Qwen API 返回的格式：{"id": "...", "type": "function", "function": {"name": "...", "arguments": "..."}}
/// ToolCall 期望的格式：{"id": "...", "name": "...", "arguments": ...}
fn normalize_tool_call_format(tc: &serde_json::Value) -> serde_json::Value {
    if let Some(obj) = tc.as_object() {
        let mut normalized = serde_json::Map::new();
        
        // 复制 id
        if let Some(id) = obj.get("id") {
            normalized.insert("id".to_string(), id.clone());
        }
        
        // 从 function 中提取 name 和 arguments
        if let Some(function) = obj.get("function") {
            if let Some(func_obj) = function.as_object() {
                // 提取 name
                if let Some(name) = func_obj.get("name") {
                    normalized.insert("name".to_string(), name.clone());
                }
                
                // 提取 arguments 并尝试解析为 JSON
                if let Some(args_value) = func_obj.get("arguments") {
                    if let Some(args_str) = args_value.as_str() {
                        // 尝试将字符串解析为 JSON 对象
                        match serde_json::from_str::<serde_json::Value>(args_str) {
                            Ok(parsed_args) => {
                                normalized.insert("arguments".to_string(), parsed_args);
                            }
                            Err(_) => {
                                // 如果解析失败，保留原始字符串
                                normalized.insert("arguments".to_string(), args_value.clone());
                            }
                        }
                    } else {
                        // 如果 arguments 不是字符串，直接复制
                        normalized.insert("arguments".to_string(), args_value.clone());
                    }
                }
            }
        }
        
        serde_json::Value::Object(normalized)
    } else {
        tc.clone()
    }
}

/// 流式聊天完成请求
#[derive(Debug, Clone, serde::Serialize)]
struct StreamRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f64,
    top_p: f64,
    max_tokens: i64,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_choice: Option<serde_json::Value>,
}

pub async fn stream_chat_completion(
    app: AppHandle,
    config: &ModelConfig,
    messages: Vec<ChatMessage>,
    conversation_id: &str,
    message_id: &str,
) -> AppResult<()> {
    info!("🤖 Starting Agent Loop for conversation {}", conversation_id);
    
    let mut current_messages = messages.clone();
    let mut iteration = 0;
    let max_iterations = 30;
    
    // 重复调用检测：记录上一轮的工具调用签名，避免 LLM 反复调用相同工具
    let mut last_tool_signatures: Vec<String> = Vec::new();
    let mut repeat_count = 0u32; // 连续重复次数
    
    loop {
        iteration += 1;
        info!("🔄 Agent Loop iteration {}/{}", iteration, max_iterations);
        info!("📝 Current messages count: {}", current_messages.len());
        
        // 执行单轮流式对话
        let tool_calls_result = stream_single_turn(
            app.clone(),
            config,
            current_messages.clone(),
            conversation_id,
            message_id.to_string(),
        ).await?;
        
        // 如果没有工具调用，结束循环
        if tool_calls_result.is_empty() {
            info!("✅ No more tool calls, agent loop completed after {} iterations", iteration);
            break;
        }
        
        // 如果有工具调用，并行执行它们
        info!("🔧 Found {} tool calls in iteration {}, executing in parallel...", tool_calls_result.len(), iteration);
        
        // 记录本轮所有工具调用
        for (i, tc) in tool_calls_result.iter().enumerate() {
            info!("  🔧 Tool call #{}: {}", i, serde_json::to_string(tc).unwrap_or_default());
        }
        
        // ===== 重复调用检测 =====
        let current_signatures = compute_tool_signatures(&tool_calls_result);
        if current_signatures == last_tool_signatures {
            repeat_count += 1;
            warn!("⚠️  Detected repeated tool calls (consecutive #{}): {:?}", repeat_count, current_signatures);
            
            if repeat_count >= 2 {
                // 连续重复 2 次以上：注入强制终止提示
                warn!("🛑 LLM is looping on same tools ({} times), injecting force-stop nudge", repeat_count);
                
                // 为每个工具调用添加一个“已执行过”的消息，引导 LLM 停止
                for tc in &tool_calls_result {
                    let normalized_tc = normalize_tool_call_format(tc);
                    if let Ok(tool_call) = serde_json::from_value::<ToolCall>(normalized_tc) {
                        let nudge_msg = ChatMessage {
                            role: "tool".to_string(),
                            content: format!(
                                "[系统提示] 工具 '{}' 已在之前的迭代中成功执行过，结果已包含在对话历史中。\
                                 请勿重复调用相同工具，直接根据已有结果回答用户的问题。",
                                tool_call.name
                            ),
                            name: Some(tool_call.name.clone()),
                            tool_call_id: Some(tool_call.id.clone()),
                        };
                        current_messages.push(nudge_msg);
                    }
                }
                
                // 添加强制 assistant 引导消息
                current_messages.push(ChatMessage {
                    role: "user".to_string(),
                    content: "[系统] 你已经拥有足够的工具执行结果，请直接回答用户的问题，不要再次调用工具。".to_string(),
                    name: None,
                    tool_call_id: None,
                });
                
                // 如果重复 3 次以上，强制结束
                if repeat_count >= 3 {
                    warn!("🛑 Force stopping after {} consecutive repeats, sending done=true", repeat_count);
                    emit_chunk(&app, conversation_id, &message_id, "", false, true)?;
                    if let Some(state) = app.try_state::<AppState>() {
                        if let Ok(db) = state.db.lock() {
                            let _ = db.complete_message(&message_id);
                        }
                    }
                    break;
                }
                
                // 重置签名以避免立即再次匹配（让 LLM 有机会做最后一次不同调用）
                last_tool_signatures = vec!["__force_break__".to_string()];
                info!("🔄 Continuing after nudge, waiting for LLM final response");
                continue;
            }
        } else {
            repeat_count = 0;
            last_tool_signatures = current_signatures;
        }
        
        // ===== 执行工具 =====
        let mut futures = Vec::new();
        for tc in &tool_calls_result {
            let normalized_tc = normalize_tool_call_format(tc);
            if let Ok(tool_call) = serde_json::from_value::<ToolCall>(normalized_tc) {
                info!("⚙️  Preparing tool: {}", tool_call.name);
                
                let _ = emit_tool_call_start(&app, conversation_id, &message_id, &tool_call);
                
                let app_clone = app.clone();
                let tool_call_clone = tool_call.clone();
                let future = async move {
                    match execute_tool(&app_clone, &tool_call_clone).await {
                        Ok(result) => {
                            info!("✅ Tool {} succeeded", tool_call_clone.name);
                            Ok((tool_call_clone, result))
                        }
                        Err(e) => {
                            error!("❌ Tool {} failed: {}", tool_call_clone.name, e);
                            let fake_result = ToolResult {
                                tool_call_id: tool_call_clone.id.clone(),
                                output: format!("工具执行失败: {}", e),
                                success: false,
                            };
                            Ok((tool_call_clone, fake_result))
                        }
                    }
                };
                futures.push(future);
            } else {
                error!("❌ Failed to parse tool call: {:?}", tc);
                let error_msg = ChatMessage {
                    role: "tool".to_string(),
                    content: "工具调用格式错误，无法解析。请检查工具参数并重试。".to_string(),
                    name: Some("unknown_tool".to_string()),
                    tool_call_id: None,
                };
                current_messages.push(error_msg);
            }
        }
        
        let tool_results: Vec<Result<(ToolCall, ToolResult), AppError>> = futures::future::join_all(futures).await;
        
        info!("✅ All tools completed in iteration {}, collecting results", iteration);
        
        for result in &tool_results {
            match result {
                Ok((tc, tr)) => {
                    info!("  📋 Tool result: {} (success={}, output_len={})", tc.name, tr.success, tr.output.len());
                    info!("     output_preview: {}", tr.output.chars().take(200).collect::<String>());
                }
                Err(e) => {
                    error!("  ❌ Tool result error: {}", e);
                }
            }
        }
        
        for result in &tool_results {
            match result {
                Ok((tool_call, tool_result)) => {
                    let _ = emit_tool_call_result(&app, conversation_id, &message_id, tool_call, tool_result);
                    
                    let tool_msg = ChatMessage {
                        role: "tool".to_string(),
                        content: tool_result.output.clone(),
                        name: Some(tool_call.name.clone()),
                        tool_call_id: Some(tool_call.id.clone()),
                    };
                    current_messages.push(tool_msg);
                    
                    info!("📝 Added tool result: {} = {}", tool_call.name, tool_result.output.chars().take(50).collect::<String>());
                }
                Err(e) => {
                    error!("❌ Unexpected error in tool execution: {:?}", e);
                    let error_msg = ChatMessage {
                        role: "tool".to_string(),
                        content: format!("工具执行异常: {}", e),
                        name: Some("system_error".to_string()),
                        tool_call_id: None,
                    };
                    current_messages.push(error_msg);
                }
            }
        }
        
        // 检查是否达到最大迭代次数
        if iteration >= max_iterations {
            warn!("⚠️  Reached max iterations ({}), stopping agent loop", max_iterations);
            info!("📤 Sending done=true chunk after max iterations");
            emit_chunk(&app, conversation_id, &message_id, "", false, true)?;
            if let Some(state) = app.try_state::<AppState>() {
                if let Ok(db) = state.db.lock() {
                    let _ = db.complete_message(&message_id);
                }
            }
            break;
        }
        
        info!("🔄 Continuing to next iteration with {} messages", current_messages.len());
    }
    
    info!("🏁 Agent Loop completed after {} iterations", iteration);
    Ok(())
}

/// 计算工具调用签名列表（用于检测重复调用）
/// 签名格式: "tool_name(arg1=val1,arg2=val2)"
fn compute_tool_signatures(tool_calls: &[serde_json::Value]) -> Vec<String> {
    tool_calls.iter().map(|tc| {
        let normalized = normalize_tool_call_format(tc);
        let name = normalized.get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let args = normalized.get("arguments")
            .map(|v| v.to_string())
            .unwrap_or_else(|| "{}".to_string());
        format!("{}({})", name, args)
    }).collect()
}

/// 执行单轮流式对话，返回工具调用列表（如果有）
async fn stream_single_turn(
    app: AppHandle,
    config: &ModelConfig,
    messages: Vec<ChatMessage>,
    conversation_id: &str,
    message_id: String,
) -> AppResult<Vec<serde_json::Value>> {
    let url = get_api_url(config)?;
    let headers = build_headers(config)?;

    // 构建请求体，支持工具调用（异步加载 Skills 和 MCP 工具）
    let body = build_stream_request(&app, config, &messages).await;

    let client = Client::new();
    let mut request = client.post(&url);

    for (key, value) in &headers {
        request = request.header(key.as_str(), value.as_str());
    }

    let body_json = serde_json::to_string(&body)?;

    // 打印完整的 LLM 请求（用于调试）
    info!("=== LLM Request ===");
    info!("URL: {}", url);
    info!("Model: {}", config.model_id);
    info!("Messages count: {}", body.messages.len());
    for (i, msg) in body.messages.iter().enumerate() {
        // 安全截断：按字符边界截断，避免 UTF-8 panic
        let content_preview: String = if msg.content.chars().count() > 300 {
            let truncated: String = msg.content.chars().take(300).collect();
            format!("{}... (truncated, total {} chars)", truncated, msg.content.chars().count())
        } else {
            msg.content.clone()
        };
        info!("  [{}] role={}, content_len={}", i, msg.role, msg.content.len());
        info!("       content={}", content_preview);
        if let Some(ref name) = msg.name {
            info!("       name={}", name);
        }
        if let Some(ref tool_call_id) = msg.tool_call_id {
            info!("       tool_call_id={}", tool_call_id);
        }
    }
    if let Some(ref tools) = body.tools {
        info!("Tools count: {}", tools.len());
        for tool in tools {
            if let Some(func) = tool.get("function") {
                if let Some(name) = func.get("name") {
                    info!("  - tool: {}", name);
                }
            }
        }
    }
    info!("=== End LLM Request ===");

    info!(
        "Starting SSE stream for conversation {} (model: {})",
        conversation_id, config.model_id
    );

    let mut event_source = request
        .body(body_json)
        .eventsource()
        .map_err(|e| AppError::AiProvider(format!("Failed to create event source: {}", e)))?;

    let mut full_response = String::new();
    let mut full_thinking = String::new();
    let mut thinking_started = false;
    let mut has_tool_calls = false;
    let mut sse_chunk_count = 0;
    
    info!("📡 Starting SSE stream, iteration context: conversation={}", conversation_id);
    
    // 用于累积流式 tool calls
    // key: tool_call index, value: 累积的 ToolCall
    let mut accumulating_tool_calls: std::collections::HashMap<usize, serde_json::Map<String, serde_json::Value>> = std::collections::HashMap::new();

    while let Some(event) = event_source.next().await {
        // 检查是否被取消
        if let Some(state) = app.try_state::<AppState>() {
            if state.cancel_flag.load(std::sync::atomic::Ordering::SeqCst) {
                info!("Stream cancelled by user");
                // 发送完成信号
                emit_chunk(&app, conversation_id, &message_id, "", false, true)?;
                // 标记消息为完成
                if let Ok(db) = state.db.lock() {
                    let _ = db.complete_message(&message_id);
                }
                // 重置取消标志
                state.cancel_flag.store(false, std::sync::atomic::Ordering::SeqCst);
                break;
            }
        }

        match event {
            Ok(Event::Open) => {
                info!("SSE connection opened");
            }
            Ok(Event::Message(msg)) => {
                sse_chunk_count += 1;
                if msg.data == "[DONE]" {
                    info!("📡 SSE [DONE] received after {} chunks, full_response_len={}, tool_calls_count={}", 
                          sse_chunk_count, full_response.len(), accumulating_tool_calls.len());
                    
                    // 如果有工具调用，不要发送 done=true，因为还会有下一轮
                    let final_tool_calls: Vec<serde_json::Value> = accumulating_tool_calls
                        .values()
                        .map(|m| serde_json::Value::Object(m.clone()))
                        .collect();
                    
                    if !final_tool_calls.is_empty() {
                        info!("🔧 Found {} tool calls, returning to Agent Loop", final_tool_calls.len());
                        
                        // 发送工具调用通知到前端（仅通知，不执行）
                        for tc in &final_tool_calls {
                            if let Ok(tool_call) = serde_json::from_value::<ToolCall>(normalize_tool_call_format(tc)) {
                                let _ = emit_tool_call_start(&app, conversation_id, &message_id, &tool_call);
                            }
                        }
                        
                        // 不发送 done=true，也不标记为 complete，因为还会有下一轮
                        // 只发送一个空 chunk 表示这一轮暂停
                        info!("📤 Sending pause chunk (done=false) before tool execution");
                        emit_chunk(&app, conversation_id, &message_id, "", false, false)?;
                        info!("✅ Pause chunk sent");
                        
                        return Ok(final_tool_calls);
                    } else {
                        // 没有工具调用，才是真正的结束
                        info!("ℹ️  No tool calls, single turn completed");
                        
                        // 发送完成信号
                        emit_chunk(&app, conversation_id, &message_id, "", false, true)?;

                        // 标记消息为完成
                        if let Some(state) = app.try_state::<AppState>() {
                            if let Ok(db) = state.db.lock() {
                                let _ = db.complete_message(&message_id);
                            }
                        }
                        
                        return Ok(vec![]);
                    }
                }

                match serde_json::from_str::<StreamDelta>(&msg.data) {
                    Ok(delta) => {
                        if let Some(choice) = delta.choices.first() {
                            // 检查是否有工具调用
                            if let Some(ref tc) = choice.delta.tool_calls {
                                info!("🔧 Received tool_calls chunk from AI: {} items", tc.len());
                                
                                for tool_call_delta in tc {
                                    let index = tool_call_delta.index;
                                    info!("  Processing tool call delta at index: {}", index);
                                    info!("    ID: {:?}", tool_call_delta.id);
                                    info!("    Type: {:?}", tool_call_delta.call_type);
                                    if let Some(ref func) = tool_call_delta.function {
                                        info!("    Function name: {:?}", func.name);
                                        info!("    Function args: {:?}", func.arguments);
                                    }
                                    
                                    // 获取或创建该 index 的累积对象
                                    let acc = accumulating_tool_calls.entry(index).or_insert_with(|| {
                                        serde_json::Map::new()
                                    });
                                    
                                    // 合并 ID
                                    if let Some(ref id) = tool_call_delta.id {
                                        if !id.is_empty() {
                                            acc.insert("id".to_string(), serde_json::Value::String(id.clone()));
                                        }
                                    }
                                    
                                    // 合并 type
                                    if let Some(ref call_type) = tool_call_delta.call_type {
                                        acc.insert("type".to_string(), serde_json::Value::String(call_type.clone()));
                                    }
                                    
                                    // 合并 function
                                    if let Some(ref func_delta) = tool_call_delta.function {
                                        // 获取或创建 function 对象
                                        let func_obj = acc.entry("function".to_string())
                                            .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()))
                                            .as_object_mut().unwrap();
                                        
                                        // 合并 name
                                        if let Some(ref name) = func_delta.name {
                                            if !name.is_empty() {
                                                func_obj.insert("name".to_string(), serde_json::Value::String(name.clone()));
                                            }
                                        }
                                        
                                        // 合并 arguments (追加)
                                        if let Some(ref args) = func_delta.arguments {
                                            // 获取现有的 arguments
                                            let existing_args = match func_obj.get("arguments") {
                                                Some(serde_json::Value::String(s)) => s.clone(),
                                                _ => String::new(),
                                            };
                                            let new_args = format!("{}{}", existing_args, args);
                                            func_obj.insert("arguments".to_string(), serde_json::Value::String(new_args));
                                        }
                                    }
                                    
                                    info!("  Accumulated tool call at index {}: {:?}", index, acc);
                                }
                            }

                            // 处理 reasoning/thinking 内容
                            if let Some(reasoning) = &choice.delta.reasoning_content {
                                if !reasoning.is_empty() {
                                    full_thinking.push_str(reasoning);
                                    info!("🧠 Received reasoning_content: {} chars", reasoning.len());
                                    // 只在第一次发送 thinking 标记
                                    if !thinking_started {
                                        thinking_started = true;
                                        emit_thinking_chunk(&app, conversation_id, &message_id)?;
                                    }
                                    // 发送 thinking 内容并保存到数据库
                                    emit_chunk(&app, conversation_id, &message_id, reasoning, true, false)?;
                                    save_chunk_to_db(&app, &message_id, reasoning, true);
                                }
                            }

                            // 处理文本内容
                            if let Some(content) = &choice.delta.content {
                                if !content.is_empty() {
                                    full_response.push_str(content);
                                    info!("💬 Received content: {} chars", content.len());
                                    // 发送正式回复内容并保存到数据库
                                    emit_chunk(&app, conversation_id, &message_id, content, false, false)?;
                                    save_chunk_to_db(&app, &message_id, content, false);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        // 忽略解析错误，可能是空行或其他非数据行
                        if !msg.data.trim().is_empty() {
                            warn!("Failed to parse SSE delta: {} - data: {}", e, &msg.data);
                        }
                    }
                }
            }
            Err(e) => {
                error!("SSE stream error: {}", e);
                // 发送错误事件到前端
                let error_msg = format!(
                    "AI 服务错误: {}\n\n请检查:\n1. 网络连接是否正常\n2. API Key 是否正确\n3. Base URL 配置: {}\n4. 模型: {}",
                    e,
                    get_api_url(config).unwrap_or_default(),
                    config.model_id
                );
                emit_error(&app, conversation_id, &error_msg)?;
                emit_chunk(&app, conversation_id, &message_id, "", false, true)?;
                // 标记消息为错误
                if let Some(state) = app.try_state::<AppState>() {
                    if let Ok(db) = state.db.lock() {
                        let _ = db.update_message(&message_id, &crate::db::messages::UpdateMessageRequest {
                            content: None,
                            status: Some("error".into()),
                        });
                    }
                }
                return Err(AppError::AiProvider(format!("Stream error: {}", e)));
            }
        }
    }

    event_source.close();
    
    // 检查是否有未处理的 tool calls（SSE 流结束但没收到 [DONE]）
    let orphan_tool_calls: Vec<serde_json::Value> = accumulating_tool_calls
        .values()
        .map(|m| serde_json::Value::Object(m.clone()))
        .collect();
    
    if !orphan_tool_calls.is_empty() {
        warn!("⚠️  SSE stream ended without [DONE], but {} tool calls were accumulated, returning them", orphan_tool_calls.len());
        return Ok(orphan_tool_calls);
    }
    
    info!(
        "📡 SSE stream closed: conversation={} ({} chunks, {} thinking chars, {} response chars, no tool calls)",
        conversation_id,
        sse_chunk_count,
        full_thinking.len(),
        full_response.len(),
    );

    // 如果流结束了但没有 [DONE]，也没有 tool calls，发送完成信号
    emit_chunk(&app, conversation_id, &message_id, "", false, true)?;
    if let Some(state) = app.try_state::<AppState>() {
        if let Ok(db) = state.db.lock() {
            let _ = db.complete_message(&message_id);
        }
    }

    Ok(vec![])
}

/// 构建流式聊天请求，支持工具调用
async fn build_stream_request(app: &AppHandle, config: &ModelConfig, messages: &[ChatMessage]) -> StreamRequest {
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

    // 如果模型支持工具调用，添加工具 schema（异步加载 Skills 和 MCP）
    if supports_tool_calling(&config.provider) {
        let tools = get_available_tools_schemas_async(app).await;
        info!("Model {} (provider: {}) supports tool calling, injecting {} tools", 
              config.model_id, config.provider, tools.len());
        req.tools = Some(tools);
        // auto: 让模型自行决定是否使用工具
        req.tool_choice = Some(json!("auto"));
    } else {
        info!("Model {} (provider: {}) does NOT support tool calling", 
              config.model_id, config.provider);
    }

    req
}

/// 检查提供商是否支持工具调用
fn supports_tool_calling(provider: &str) -> bool {
    matches!(
        provider,
        "openai" | "anthropic" | "google" | "deepseek" | "moonshot" | "zhipu" | "qwen" | "aliyun" | "dashscope"
    )
}

/// 保存流式内容到数据库
fn save_chunk_to_db(app: &AppHandle, message_id: &str, delta: &str, is_thinking: bool) {
    if let Some(state) = app.try_state::<AppState>() {
        if let Ok(db) = state.db.lock() {
            let _ = db.append_message_content(message_id, delta, is_thinking);
        }
    }
}

/// 发送流式片段到前端
fn emit_chunk(
    app: &AppHandle,
    conversation_id: &str,
    message_id: &str,
    delta: &str,
    is_thinking: bool,
    done: bool,
) -> AppResult<()> {
    let chunk = StreamChunk {
        conversation_id: conversation_id.to_string(),
        message_id: message_id.to_string(),
        delta: delta.to_string(),
        is_thinking,
        done,
    };
    
    if !delta.is_empty() {
        info!("📤 Emitting chunk: is_thinking={}, delta_len={}, done={}", is_thinking, delta.len(), done);
    }
    
    app.emit("ai:stream-chunk", &chunk)
        .map_err(|e| AppError::Internal(format!("Failed to emit chunk: {}", e)))?;
    Ok(())
}

/// 发送 thinking 标记到前端
fn emit_thinking_chunk(
    app: &AppHandle,
    conversation_id: &str,
    message_id: &str,
) -> AppResult<()> {
    let chunk = StreamChunk {
        conversation_id: conversation_id.to_string(),
        message_id: message_id.to_string(),
        delta: "".to_string(),
        is_thinking: true,
        done: false,
    };
    app.emit("ai:stream-chunk", &chunk)
        .map_err(|e| AppError::Internal(format!("Failed to emit thinking chunk: {}", e)))?;
    Ok(())
}

/// 发送错误事件到前端
fn emit_error(
    app: &AppHandle,
    conversation_id: &str,
    error_msg: &str,
) -> AppResult<()> {
    #[derive(Debug, Clone, serde::Serialize)]
    struct StreamError {
        conversation_id: String,
        error: String,
    }
    
    let error_event = StreamError {
        conversation_id: conversation_id.to_string(),
        error: error_msg.to_string(),
    };
    
    app.emit("ai:stream-error", &error_event)
        .map_err(|e| AppError::Internal(format!("Failed to emit error: {}", e)))?;
    Ok(())
}

/// 发送工具调用开始事件
fn emit_tool_call_start(
    app: &AppHandle,
    conversation_id: &str,
    message_id: &str,
    tool_call: &ToolCall,
) -> AppResult<()> {
    #[derive(Debug, Clone, serde::Serialize)]
    struct ToolCallStart {
        conversation_id: String,
        message_id: String,
        tool_name: String,
        arguments: serde_json::Value,
    }
    
    let event = ToolCallStart {
        conversation_id: conversation_id.to_string(),
        message_id: message_id.to_string(),
        tool_name: tool_call.name.clone(),
        arguments: tool_call.arguments.clone(),
    };
    
    app.emit("ai:tool-call-start", &event)
        .map_err(|e| AppError::Internal(format!("Failed to emit tool call start: {}", e)))?;
    Ok(())
}

/// 发送工具调用结果事件
fn emit_tool_call_result(
    app: &AppHandle,
    conversation_id: &str,
    message_id: &str,
    tool_call: &ToolCall,
    result: &ToolResult,
) -> AppResult<()> {
    #[derive(Debug, Clone, serde::Serialize)]
    struct ToolCallResult {
        conversation_id: String,
        message_id: String,
        tool_name: String,
        output: String,
        success: bool,
    }
    
    let event = ToolCallResult {
        conversation_id: conversation_id.to_string(),
        message_id: message_id.to_string(),
        tool_name: tool_call.name.clone(),
        output: result.output.clone(),
        success: result.success,
    };
    
    app.emit("ai:tool-call-result", &event)
        .map_err(|e| AppError::Internal(format!("Failed to emit tool call result: {}", e)))?;
    Ok(())
}

/// 执行工具调用
async fn execute_tool(app: &AppHandle, tool_call: &ToolCall) -> AppResult<ToolResult> {
    // 委托给工具调度器
    crate::ai::tools_impl::execute(app, tool_call).await
}
