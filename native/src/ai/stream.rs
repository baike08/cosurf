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
use crate::ai::checkpoint::{CheckpointManager, FileChange, backup_file_if_needed};
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

    // 初始化 CheckpointManager（使用 SQLite 存储）
    // 使用应用数据目录存储检查点数据库
    let app_data_dir = std::env::var("COSURF_APP_DATA_DIR")
        .unwrap_or_else(|_| ".".to_string());
    let checkpoint_db_path = std::path::Path::new(&app_data_dir)
        .join(format!("checkpoint_{}.db", conversation_id))
        .to_string_lossy().to_string();
    
    let mut checkpoint_mgr = match CheckpointManager::new(&checkpoint_db_path) {
        Ok(mgr) => {
            info!("✅ CheckpointManager initialized: {}", checkpoint_db_path);
            Some(mgr)
        }
        Err(e) => {
            warn!("⚠️  Failed to initialize CheckpointManager: {}, continuing without checkpoints", e);
            None
        }
    };

    let mut current_messages = messages;
    let mut iteration = 0;
    let max_iterations = 30;

    // 重复调用检测
    let mut last_tool_signatures: Vec<String> = Vec::new();
    let mut repeat_count = 0u32;

    loop {
        iteration += 1;
        info!("🔄 Agent Loop iteration {}/{}", iteration, max_iterations);

        // 在迭代前创建检查点（仅当 CheckpointManager 可用时）
        if let Some(ref mut mgr) = checkpoint_mgr {
            // 只在第 3 次迭代后开始创建检查点，避免过多开销
            if iteration >= 3 {
                match mgr.create_checkpoint(
                    conversation_id,
                    iteration - 1, // 保存上一次迭代的状态
                    vec![], // new_messages 为空，增量记录
                    vec![], // file_changes 将在工具执行后填充
                    vec![], // tool_results 将在工具执行后填充
                ) {
                    Ok(cp_id) => info!("📸 Created checkpoint: {} (iteration={})", cp_id, iteration - 1),
                    Err(e) => warn!("⚠️  Failed to create checkpoint: {}", e),
                }
            }
        }

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

        // 收集工具执行结果和文件变更记录
        let mut file_changes: Vec<FileChange> = vec![];
        let mut tool_result_records: Vec<crate::ai::checkpoint::ToolResultRecord> = vec![];
        let mut failed_tools: Vec<(ToolCall, String)> = vec![]; // 记录失败的工具

        for result in &tool_results {
            match result {
                Ok((tool_call, tool_result)) => {
                    callbacks.emit_tool_result(conversation_id, message_id, tool_call, tool_result);

                    // 记录工具执行结果
                    tool_result_records.push(crate::ai::checkpoint::ToolResultRecord::new(tool_call, tool_result));

                    // 检测文件变更（针对写入类工具）
                    if tool_call.name == "export_markdown" || tool_call.name == "run_command" {
                        // 尝试从工具参数中提取文件路径
                        if let Some(path) = extract_file_path_from_tool(tool_call) {
                            // 备份文件（如果存在）
                            match backup_file_if_needed(&path) {
                                Ok(Some(change)) => {
                                    info!("📦 Backed up file before modification: {}", path);
                                    file_changes.push(change);
                                }
                                Ok(None) => {
                                    // 文件不存在，无需备份（可能是新创建的文件）
                                }
                                Err(e) => {
                                    warn!("⚠️  Failed to backup file {}: {}", path, e);
                                }
                            }
                        }
                    }

                    let tool_msg = ChatMessage {
                        role: "tool".to_string(),
                        content: tool_result.output.clone(),
                        name: Some(tool_call.name.clone()),
                        tool_call_id: Some(tool_call.id.clone()),
                    };
                    current_messages.push(tool_msg);
                }
                Err(e) => {
                    // 记录失败的工具
                    warn!("❌ Tool execution failed: {}", e);
                    // 这里可以尝试从错误中提取 tool_call 信息
                }
            }
        }

        // 检测连续失败并触发回滚
        if !failed_tools.is_empty() && failed_tools.len() >= 3 {
            warn!("🔄 Detected {} consecutive failures, attempting rollback...", failed_tools.len());
            
            if let Some(ref mgr) = checkpoint_mgr {
                // 获取上一个检查点
                if let Ok(Some(prev_checkpoint)) = mgr.get_latest_checkpoint(conversation_id) {
                    warn!("🔄 Rolling back to checkpoint: {} (iteration={})", 
                        prev_checkpoint.id, prev_checkpoint.iteration);
                    
                    // 回滚文件变更
                    if !prev_checkpoint.file_changes.is_empty() {
                        match crate::ai::checkpoint::rollback_file_changes(&prev_checkpoint.file_changes) {
                            Ok(()) => info!("✅ Successfully rolled back file changes"),
                            Err(e) => error!("❌ Failed to rollback file changes: {}", e),
                        }
                    }
                    
                    // 恢复消息上下文
                    current_messages = prev_checkpoint.new_messages;
                    
                    // 通知用户
                    callbacks.emit_chunk(
                        conversation_id,
                        message_id,
                        "\n[系统] 检测到连续失败，已回滚到上一个稳定状态。请重试。\n",
                        false,
                        false,
                    );
                    
                    // 跳过本次迭代，继续下一轮
                    continue;
                }
            }
        }

        // 更新检查点（添加文件变更和工具结果）
        if let Some(ref mut mgr) = checkpoint_mgr {
            if !file_changes.is_empty() || !tool_result_records.is_empty() {
                match mgr.create_checkpoint(
                    conversation_id,
                    iteration,
                    vec![], // new_messages
                    file_changes,
                    tool_result_records,
                ) {
                    Ok(cp_id) => info!("📸 Updated checkpoint with file changes: {} (iteration={})", cp_id, iteration),
                    Err(e) => warn!("⚠️  Failed to update checkpoint: {}", e),
                }
            }
        }

        if iteration >= max_iterations {
            warn!("⚠️  Reached max iterations, stopping agent loop");
            callbacks.emit_chunk(conversation_id, message_id, "", false, true);
            break;
        }
    }

    info!("🏁 Agent Loop completed after {} iterations", iteration);
    
    // 会话结束时清理检查点（保留最近 24 小时）
    if let Some(mut mgr) = checkpoint_mgr {
        match mgr.cleanup_old_checkpoints(24) {
            Ok(deleted) => {
                if deleted > 0 {
                    info!("🧹 Cleaned up {} old checkpoints (retention: 24h)", deleted);
                }
            }
            Err(e) => warn!("⚠️  Failed to cleanup checkpoints: {}", e),
        }
        
        // 清理过期备份文件（保留最近 24 小时）
        match crate::ai::checkpoint::cleanup_old_backups(24) {
            Ok(deleted) => {
                if deleted > 0 {
                    info!("🧹 Cleaned up {} old backup files (retention: 24h)", deleted);
                }
            }
            Err(e) => warn!("⚠️  Failed to cleanup backup files: {}", e),
        }
    }
    
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

    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(120)) // 2分钟超时
        .build()
        .map_err(|e| AppError::Internal(format!("Failed to create HTTP client: {}", e)))?;
    
    let mut request = client.post(&url);
    for (key, value) in &headers {
        request = request.header(key.as_str(), value.as_str());
    }

    let body_json = serde_json::to_string(&body)?;

    info!("📤 Sending request to {} (model={}, messages={}, tools={})", 
        url, config.model_id, body.messages.len(), 
        body.tools.as_ref().map(|t| t.len()).unwrap_or(0));

    let mut event_source = request
        .body(body_json)
        .eventsource()
        .map_err(|e| AppError::AiProvider(format!("Failed to create event source: {}", e)))?;

    info!("✅ SSE connection established, waiting for response...");

    let mut full_response = String::new();
    let mut full_thinking = String::new();
    let mut thinking_started = false;
    let mut sse_chunk_count = 0;
    let mut last_event_time = std::time::Instant::now();
    let mut accumulating_tool_calls: std::collections::HashMap<usize, serde_json::Map<String, serde_json::Value>> = std::collections::HashMap::new();

    while let Some(event) = event_source.next().await {
        // 检查取消
        if is_cancelled() {
            info!("Stream cancelled by user");
            callbacks.emit_chunk(conversation_id, message_id, "", false, true);
            reset_cancel();
            break;
        }
        
        // 检查超时（30秒无响应）
        let elapsed = last_event_time.elapsed();
        if elapsed > std::time::Duration::from_secs(30) {
            error!("⏰ SSE timeout: no events for {:?}", elapsed);
            callbacks.emit_chunk(conversation_id, message_id, "", false, true);
            return Err(AppError::AiProvider("Response timeout (30s)".to_string()));
        }

        match event {
            Ok(Event::Open) => {
                info!("SSE connection opened");
                last_event_time = std::time::Instant::now();
            }
            Ok(Event::Message(msg)) => {
                sse_chunk_count += 1;
                last_event_time = std::time::Instant::now(); // 更新最后事件时间
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

/// 优化的 Agent Loop（集成智能调度和上下文管理）
pub async fn stream_chat_completion_optimized(
    config: &ModelConfig,
    messages: Vec<ChatMessage>,
    conversation_id: &str,
    message_id: &str,
    callbacks: &StreamCallbacks,
    skills_schemas: Vec<serde_json::Value>,
) -> AppResult<()> {
    use crate::ai::{scheduler, context_manager};
    
    info!("🤖 Starting Optimized Agent Loop for conversation {}", conversation_id);
    
    // 初始化上下文管理器
    let mut ctx_mgr = context_manager::ContextManager::new(messages);
    let mut iteration = 0;
    let max_iterations = 30;
    let token_budget = 100000; // 默认 100K tokens
    
    // 重复调用检测
    let mut last_tool_signatures: Vec<String> = Vec::new();
    let mut repeat_count = 0u32;
    
    loop {
        iteration += 1;
        info!("🔄 Optimized Agent Loop iteration {}/{}", iteration, max_iterations);
        
        // 1. 检查 Token 预算
        let estimated_tokens = ctx_mgr.estimate_tokens();
        if estimated_tokens > token_budget {
            warn!("🛑 Token budget exceeded ({} / {})", estimated_tokens, token_budget);
            callbacks.emit_chunk(conversation_id, message_id, 
                "[系统] 已达到 Token 预算限制，停止执行", false, true);
            break;
        }
        
        // 2. 压缩上下文（如果需要）
        ctx_mgr.compress_if_needed(token_budget, 0.8); // 压缩到 80%
        
        // 3. 执行单轮流式对话
        let current_messages = ctx_mgr.messages();
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
        
        // 4. 解析工具调用
        let mut tool_calls: Vec<ToolCall> = vec![];
        for tc in &tool_calls_result {
            let normalized_tc = normalize_tool_call_format(tc);
            if let Ok(tool_call) = serde_json::from_value::<ToolCall>(normalized_tc) {
                tool_calls.push(tool_call);
            }
        }
        
        // 5. 重复调用检测
        let current_signatures = compute_tool_signatures(&tool_calls_result);
        if current_signatures == last_tool_signatures {
            repeat_count += 1;
            warn!("⚠️  Detected repeated tool calls (consecutive #{})", repeat_count);
            
            if repeat_count >= 2 {
                warn!("🛑 LLM looping, injecting force-stop nudge");
                
                for tc in &tool_calls {
                    let nudge_msg = ChatMessage {
                        role: "tool".to_string(),
                        content: format!(
                            "[系统提示] 工具 '{}' 已在之前的迭代中成功执行过。请勿重复调用，直接根据已有结果回答。",
                            tc.name
                        ),
                        name: Some(tc.name.clone()),
                        tool_call_id: Some(tc.id.clone()),
                    };
                    ctx_mgr.add_user_message(nudge_msg.content);
                }
                
                ctx_mgr.add_user_message(
                    "[系统] 你已拥有足够的工具执行结果，请直接回答用户的问题。".to_string()
                );
                
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
        
        // 6. 智能并行调度
        let scheduled = scheduler::schedule_tools(tool_calls);
        
        // 7. 执行工具（按类别分组执行）
        let mut all_results: Vec<(ToolCall, ToolResult)> = vec![];
        
        // 7.1 并行执行读取类工具
        if !scheduled.read_tools.is_empty() {
            info!("📖 Executing {} read tools in parallel", scheduled.read_tools.len());
            let mut futures = vec![];
            for tc in scheduled.read_tools {
                let cbs = callbacks.clone();
                let conv_id = conversation_id.to_string();
                let msg_id = message_id.to_string();
                
                cbs.emit_tool_call_start(&conv_id, &msg_id, &tc);
                
                let tc_clone = tc.clone();
                futures.push(async move {
                    match execute_tool(&tc_clone).await {
                        Ok(result) => Ok((tc_clone, result)),
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
            let results: Vec<std::result::Result<(ToolCall, ToolResult), AppError>> = 
                futures::future::join_all(futures).await;
            for r in results {
                if let Ok((tc, result)) = r {
                    all_results.push((tc, result));
                }
            }
        }
        
        // 7.2 并行执行网络类工具
        if !scheduled.network_tools.is_empty() {
            info!("🌐 Executing {} network tools in parallel", scheduled.network_tools.len());
            let mut futures = vec![];
            for tc in scheduled.network_tools {
                let cbs = callbacks.clone();
                let conv_id = conversation_id.to_string();
                let msg_id = message_id.to_string();
                
                cbs.emit_tool_call_start(&conv_id, &msg_id, &tc);
                
                let tc_clone = tc.clone();
                futures.push(async move {
                    match execute_tool(&tc_clone).await {
                        Ok(result) => Ok((tc_clone, result)),
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
            let results: Vec<std::result::Result<(ToolCall, ToolResult), AppError>> = 
                futures::future::join_all(futures).await;
            for r in results {
                if let Ok((tc, result)) = r {
                    all_results.push((tc, result));
                }
            }
        }
        
        // 7.3 串行执行写入类工具
        for tc in scheduled.write_tools {
            info!("✍️  Executing write tool: {}", tc.name);
            callbacks.emit_tool_call_start(conversation_id, message_id, &tc);
            
            match execute_tool(&tc).await {
                Ok(result) => all_results.push((tc, result)),
                Err(e) => {
                    let fake = ToolResult {
                        tool_call_id: tc.id.clone(),
                        output: format!("工具执行失败: {}", e),
                        success: false,
                    };
                    all_results.push((tc, fake));
                }
            }
        }
        
        // 7.4 串行执行浏览器类工具
        for tc in scheduled.browser_tools {
            info!("🌍 Executing browser tool: {}", tc.name);
            callbacks.emit_tool_call_start(conversation_id, message_id, &tc);
            
            match execute_tool(&tc).await {
                Ok(result) => all_results.push((tc, result)),
                Err(e) => {
                    let fake = ToolResult {
                        tool_call_id: tc.id.clone(),
                        output: format!("工具执行失败: {}", e),
                        success: false,
                    };
                    all_results.push((tc, fake));
                }
            }
        }
        
        // 8. 添加工具结果到上下文
        for (tool_call, tool_result) in &all_results {
            callbacks.emit_tool_result(conversation_id, message_id, tool_call, tool_result);
            ctx_mgr.add_tool_result(tool_call, tool_result);
        }
        
        // 9. 冻结重要消息
        ctx_mgr.freeze_important_messages();
        
        if iteration >= max_iterations {
            warn!("⚠️  Reached max iterations, stopping agent loop");
            callbacks.emit_chunk(conversation_id, message_id, "", false, true);
            break;
        }
    }
    
    info!("🏁 Optimized Agent Loop completed after {} iterations", iteration);
    Ok(())
}

// ============================================================
// 辅助函数：从工具参数中提取文件路径
// ============================================================

/// 从工具调用中提取文件路径（如果存在）
fn extract_file_path_from_tool(tool_call: &ToolCall) -> Option<String> {
    // 尝试从 arguments 中提取 path 或 file_path 字段
    let args = &tool_call.arguments;
    
    if let Some(path) = args.get("path").and_then(|v| v.as_str()) {
        return Some(path.to_string());
    }
    
    if let Some(file_path) = args.get("file_path").and_then(|v| v.as_str()) {
        return Some(file_path.to_string());
    }
    
    if let Some(filename) = args.get("filename").and_then(|v| v.as_str()) {
        return Some(filename.to_string());
    }
    
    None
}
