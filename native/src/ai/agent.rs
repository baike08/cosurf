//! Agent 模块 — N-API 导出接口
//!
//! 将 AI 核心功能通过 N-API 暴露给 Electron 主进程。
//! 包含: 流式对话、Agent 执行、标题生成、取消控制。

use napi::bindgen_prelude::*;
use napi::threadsafe_function::ThreadsafeFunction;
use tracing::{info, error};

use crate::ai::provider::{ChatMessage, ModelConfig};
use crate::ai::stream::{self, ChunkEvent, ToolCallEvent, ToolResultEvent, ElectronBridgeEvent, StreamCallbacks};
use crate::ai::{request_cancel, reset_cancel};
use crate::ai::skills::SkillsManager;
use crate::ai::mcp_manager::{McpToolManager, McpServerConfig};

use std::sync::Mutex;

lazy_static::lazy_static! {
    /// 全局 Skills 管理器
    static ref SKILLS_MANAGER: Mutex<Option<SkillsManager>> = Mutex::new(None);
    /// 全局 MCP 工具管理器
    static ref MCP_MANAGER: Mutex<Option<McpToolManager>> = Mutex::new(None);
}

/// 初始化 Skills 管理器
pub fn init_skills_manager(skills_dir: &str) {
    let mgr = SkillsManager::new(std::path::PathBuf::from(skills_dir));
    let mut guard = SKILLS_MANAGER.lock().unwrap();
    *guard = Some(mgr);
    info!("Skills manager initialized at: {}", skills_dir);
}

/// 加载 Skills
pub fn load_skills() -> Result<()> {
    let mut guard = SKILLS_MANAGER.lock()
        .map_err(|e| Error::from_reason(format!("Lock error: {}", e)))?;
    if let Some(ref mut mgr) = *guard {
        mgr.load_skills()
            .map_err(|e| Error::from_reason(e.to_string()))?;
    }
    Ok(())
}

/// 获取 Skills 工具 Schema
pub fn get_skills_schemas() -> Vec<serde_json::Value> {
    let guard = SKILLS_MANAGER.lock().unwrap();
    if let Some(ref mgr) = *guard {
        mgr.get_tool_schemas()
    } else {
        vec![]
    }
}

/// 获取 MCP 工具 Schema
pub fn get_mcp_schemas() -> Vec<serde_json::Value> {
    let guard = MCP_MANAGER.lock().unwrap();
    if let Some(ref mgr) = *guard {
        let schemas = mgr.get_tool_schemas();
        tracing::info!("📊 get_mcp_schemas: returning {} schemas", schemas.len());
        schemas
    } else {
        tracing::warn!("⚠️ get_mcp_schemas: MCP_MANAGER not initialized");
        vec![]
    }
}

/// 加载所有启用的 MCP Servers
pub async fn load_mcp_servers(servers: Vec<McpServerConfig>) {
    tracing::info!("🚀 Loading {} MCP servers", servers.len());
    
    let mut guard = MCP_MANAGER.lock().unwrap();
    if guard.is_none() {
        tracing::info!("🆕 Creating new MCP_MANAGER");
        *guard = Some(McpToolManager::new());
    }
    
    if let Some(ref mut mgr) = *guard {
        match mgr.load_all_servers(servers).await {
            Ok(()) => {
                let schemas_count = mgr.get_tool_schemas().len();
                tracing::info!("✅ MCP servers loaded successfully, total tools: {}", schemas_count);
            }
            Err(e) => error!("❌ Failed to load MCP servers: {}", e),
        }
    } else {
        tracing::error!("❌ Failed to lock MCP_MANAGER");
    }
}

/// 执行 MCP 工具
pub async fn execute_mcp_tool(function_name: &str, arguments: &serde_json::Value) -> crate::error::AppResult<serde_json::Value> {
    let guard = MCP_MANAGER.lock().unwrap();
    if let Some(ref mgr) = *guard {
        // MCP Manager 返回的是 String，需要解析为 Value
        let result_str = mgr.execute_tool(function_name, arguments).await?;
        // 尝试解析为 JSON Value
        serde_json::from_str(&result_str)
            .map_err(|e| crate::error::AppError::Internal(format!("Failed to parse MCP tool result: {}", e)))
    } else {
        Err(crate::error::AppError::Internal("MCP manager not initialized".to_string()))
    }
}

// ============================================================
// N-API 导出
// ============================================================

/// 流式对话 — N-API 异步函数
///
/// Electron 主进程调用:
/// ```js
/// await native.streamChat(configJson, messagesJson, convId, msgId, onChunk, onToolCall, onToolResult, onElectronBridge, onError)
/// ```
#[napi]
pub fn stream_chat(
    config_json: String,
    messages_json: String,
    conversation_id: String,
    message_id: String,
    on_chunk: JsFunction,
    on_tool_call: JsFunction,
    on_tool_result: JsFunction,
    on_electron_bridge: JsFunction,
    on_error: JsFunction,
) -> Result<()> {
    let config: ModelConfig = serde_json::from_str(&config_json)
        .map_err(|e| Error::from_reason(format!("Invalid config JSON: {}", e)))?;
    let messages: Vec<ChatMessage> = serde_json::from_str(&messages_json)
        .map_err(|e| Error::from_reason(format!("Invalid messages JSON: {}", e)))?;

    // 创建 ThreadsafeFunction 回调
    let chunk_tsfn: ThreadsafeFunction<ChunkEvent, napi::threadsafe_function::ErrorStrategy::Fatal> =
        on_chunk.create_threadsafe_function(0, |ctx| Ok(vec![ctx.value]))?;
    let tool_call_tsfn: ThreadsafeFunction<ToolCallEvent, napi::threadsafe_function::ErrorStrategy::Fatal> =
        on_tool_call.create_threadsafe_function(0, |ctx| Ok(vec![ctx.value]))?;
    let tool_result_tsfn: ThreadsafeFunction<ToolResultEvent, napi::threadsafe_function::ErrorStrategy::Fatal> =
        on_tool_result.create_threadsafe_function(0, |ctx| Ok(vec![ctx.value]))?;
    let electron_bridge_tsfn: ThreadsafeFunction<ElectronBridgeEvent, napi::threadsafe_function::ErrorStrategy::Fatal> =
        on_electron_bridge.create_threadsafe_function(0, |ctx| Ok(vec![ctx.value]))?;
    let error_tsfn: ThreadsafeFunction<String, napi::threadsafe_function::ErrorStrategy::Fatal> =
        on_error.create_threadsafe_function(0, |ctx| Ok(vec![ctx.value]))?;

    let callbacks = StreamCallbacks {
        on_chunk: Some(chunk_tsfn),
        on_tool_call: Some(tool_call_tsfn),
        on_tool_result: Some(tool_result_tsfn),
        on_electron_bridge: Some(electron_bridge_tsfn),
        on_error: Some(error_tsfn),
    };

    let skills_schemas = get_skills_schemas();

    // 在 tokio runtime 上运行异步任务
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            reset_cancel();
            match stream::stream_chat_completion(
                &config,
                messages,
                &conversation_id,
                &message_id,
                &callbacks,
                skills_schemas,
            ).await {
                Ok(()) => info!("Stream chat completed successfully"),
                Err(e) => {
                    error!("Stream chat error: {}", e);
                    callbacks.emit_error(&conversation_id, &e.to_string());
                }
            }
        });
    });

    Ok(())
}

/// 停止当前 AI 生成
#[napi]
pub fn ai_stop_generation() -> Result<()> {
    request_cancel();
    info!("Generation stop requested");
    Ok(())
}

/// 生成对话标题
#[napi]
pub fn ai_generate_title(content: String, config_json: String) -> Result<String> {
    let config: ModelConfig = serde_json::from_str(&config_json)
        .map_err(|e| Error::from_reason(format!("Invalid config JSON: {}", e)))?;

    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| Error::from_reason(format!("Failed to create runtime: {}", e)))?;

    let title = rt.block_on(async {
        stream::generate_title(&content, &config).await
            .unwrap_or_else(|_| "New Conversation".to_string())
    });

    Ok(title)
}

/// Agent 执行 — 完整 Agent Loop
#[napi]
pub fn agent_execute(
    params_json: String,
    on_chunk: JsFunction,
    on_tool_call: JsFunction,
    on_tool_result: JsFunction,
    on_electron_bridge: JsFunction,
    on_error: JsFunction,
) -> Result<()> {
    // 解析参数
    let params: serde_json::Value = serde_json::from_str(&params_json)
        .map_err(|e| Error::from_reason(format!("Invalid params JSON: {}", e)))?;

    let config: ModelConfig = serde_json::from_value(params.get("config").cloned().unwrap_or_default())
        .map_err(|e| Error::from_reason(format!("Invalid config: {}", e)))?;
    let messages: Vec<ChatMessage> = serde_json::from_value(params.get("messages").cloned().unwrap_or_default())
        .map_err(|e| Error::from_reason(format!("Invalid messages: {}", e)))?;
    let conversation_id = params.get("conversationId").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let message_id = params.get("messageId").and_then(|v| v.as_str()).unwrap_or("").to_string();

    let chunk_tsfn: ThreadsafeFunction<ChunkEvent, napi::threadsafe_function::ErrorStrategy::Fatal> =
        on_chunk.create_threadsafe_function(0, |ctx| Ok(vec![ctx.value]))?;
    let tool_call_tsfn: ThreadsafeFunction<ToolCallEvent, napi::threadsafe_function::ErrorStrategy::Fatal> =
        on_tool_call.create_threadsafe_function(0, |ctx| Ok(vec![ctx.value]))?;
    let tool_result_tsfn: ThreadsafeFunction<ToolResultEvent, napi::threadsafe_function::ErrorStrategy::Fatal> =
        on_tool_result.create_threadsafe_function(0, |ctx| Ok(vec![ctx.value]))?;
    let electron_bridge_tsfn: ThreadsafeFunction<ElectronBridgeEvent, napi::threadsafe_function::ErrorStrategy::Fatal> =
        on_electron_bridge.create_threadsafe_function(0, |ctx| Ok(vec![ctx.value]))?;
    let error_tsfn: ThreadsafeFunction<String, napi::threadsafe_function::ErrorStrategy::Fatal> =
        on_error.create_threadsafe_function(0, |ctx| Ok(vec![ctx.value]))?;

    let callbacks = StreamCallbacks {
        on_chunk: Some(chunk_tsfn),
        on_tool_call: Some(tool_call_tsfn),
        on_tool_result: Some(tool_result_tsfn),
        on_electron_bridge: Some(electron_bridge_tsfn),
        on_error: Some(error_tsfn),
    };

    let skills_schemas = get_skills_schemas();

    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            reset_cancel();
            match stream::stream_chat_completion(
                &config,
                messages,
                &conversation_id,
                &message_id,
                &callbacks,
                skills_schemas,
            ).await {
                Ok(()) => {}
                Err(e) => {
                    error!("Agent execute error: {}", e);
                    callbacks.emit_error(&conversation_id, &e.to_string());
                }
            }
        });
    });

    Ok(())
}

// ============================================================
// Skills N-API 导出
// ============================================================

/// 列出所有 Skills
#[napi]
pub fn skills_list() -> Result<String> {
    let guard = SKILLS_MANAGER.lock()
        .map_err(|e| Error::from_reason(format!("Lock error: {}", e)))?;
    if let Some(ref mgr) = *guard {
        let skills = mgr.list_skills();
        serde_json::to_string(&skills)
            .map_err(|e| Error::from_reason(format!("Serialization error: {}", e)))
    } else {
        Ok("[]".to_string())
    }
}

/// 获取 Skill 内容
#[napi]
pub fn skills_get_content(id: String) -> Result<String> {
    let mut guard = SKILLS_MANAGER.lock()
        .map_err(|e| Error::from_reason(format!("Lock error: {}", e)))?;
    if let Some(ref mut mgr) = *guard {
        mgr.get_skill_content(&id)
            .map_err(|e| Error::from_reason(e.to_string()))
    } else {
        Err(Error::from_reason("Skills manager not initialized"))
    }
}

/// 内部使用的 Skill 内容获取函数（供 tools.rs 调用）
pub fn skills_get_content_internal(skill_id: &str) -> crate::error::AppResult<String> {
    let mut guard = SKILLS_MANAGER.lock()
        .map_err(|e| crate::error::AppError::Internal(format!("Lock error: {}", e)))?;
    if let Some(ref mut mgr) = *guard {
        mgr.get_skill_content(skill_id)
    } else {
        Err(crate::error::AppError::Internal("Skills manager not initialized".to_string()))
    }
}

/// 删除 Skill
#[napi]
pub fn skills_delete_skill(id: String) -> Result<()> {
    let mut guard = SKILLS_MANAGER.lock()
        .map_err(|e| Error::from_reason(format!("Lock error: {}", e)))?;
    if let Some(ref mut mgr) = *guard {
        mgr.delete_skill(&id)
            .map_err(|e| Error::from_reason(e.to_string()))
    } else {
        Err(Error::from_reason("Skills manager not initialized"))
    }
}

/// 切换 Skill 启用状态
#[napi]
pub fn skills_toggle_skill(id: String) -> Result<bool> {
    let mut guard = SKILLS_MANAGER.lock()
        .map_err(|e| Error::from_reason(format!("Lock error: {}", e)))?;
    if let Some(ref mut mgr) = *guard {
        mgr.toggle_skill(&id)
            .map_err(|e| Error::from_reason(e.to_string()))
    } else {
        Err(Error::from_reason("Skills manager not initialized"))
    }
}

/// 从 Markdown 导入 Skill
#[napi]
pub fn skills_import_skill_from_markdown(content: String) -> Result<String> {
    let mut guard = SKILLS_MANAGER.lock()
        .map_err(|e| Error::from_reason(format!("Lock error: {}", e)))?;
    if let Some(ref mut mgr) = *guard {
        let info = mgr.import_markdown(&content)
            .map_err(|e| Error::from_reason(e.to_string()))?;
        serde_json::to_string(&info)
            .map_err(|e| Error::from_reason(format!("Serialization error: {}", e)))
    } else {
        Err(Error::from_reason("Skills manager not initialized"))
    }
}

/// 从目录导入 Skill
#[napi]
pub fn skills_import_skill_from_directory(dir_path: String) -> Result<String> {
    let mut guard = SKILLS_MANAGER.lock()
        .map_err(|e| Error::from_reason(format!("Lock error: {}", e)))?;
    if let Some(ref mut mgr) = *guard {
        let skill = mgr.import_directory(&dir_path)
            .map_err(|e| Error::from_reason(e.to_string()))?;
        serde_json::to_string(&skill)
            .map_err(|e| Error::from_reason(format!("Serialization error: {}", e)))
    } else {
        Err(Error::from_reason("Skills manager not initialized"))
    }
}

/// 列出 Skill 目录
#[napi]
pub fn skills_list_skill_files() -> Result<String> {
    let guard = SKILLS_MANAGER.lock()
        .map_err(|e| Error::from_reason(format!("Lock error: {}", e)))?;
    if let Some(ref mgr) = *guard {
        let files = mgr.list_skills();
        serde_json::to_string(&files)
            .map_err(|e| Error::from_reason(format!("Serialization error: {}", e)))
    } else {
        Err(Error::from_reason("Skills manager not initialized"))
    }
}

/// 获取 Skill 内容
#[napi]
pub fn skills_get_skill_content(id: String) -> Result<String> {
    let mut guard = SKILLS_MANAGER.lock()
        .map_err(|e| Error::from_reason(format!("Lock error: {}", e)))?;
    if let Some(ref mut mgr) = *guard {
        mgr.get_skill_content(&id)
            .map_err(|e| Error::from_reason(e.to_string()))
    } else {
        Err(Error::from_reason("Skills manager not initialized"))
    }
}

/// 更新 Skills 目录并重新加载
#[napi]
pub fn skills_set_directory(dir: String) -> Result<i64> {
    let mut guard = SKILLS_MANAGER.lock()
        .map_err(|e| Error::from_reason(format!("Lock error: {}", e)))?;
    if let Some(ref mut mgr) = *guard {
        let new_dir = std::path::PathBuf::from(&dir);
        let count = mgr.set_skills_dir(new_dir)
            .map_err(|e| Error::from_reason(e.to_string()))?;
        Ok(count as i64)
    } else {
        Err(Error::from_reason("Skills manager not initialized"))
    }
}

/// 加载所有启用的 MCP Servers
#[napi]
pub fn mcp_load_servers(servers_json: String) -> Result<()> {
    tracing::info!("🚀 mcp_load_servers called with {} bytes", servers_json.len());
    let servers: Vec<McpServerConfig> = serde_json::from_str(&servers_json)
        .map_err(|e| Error::from_reason(format!("Invalid servers JSON: {}", e)))?;
    
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            load_mcp_servers(servers).await;
        });
    });
    
    Ok(())
}
