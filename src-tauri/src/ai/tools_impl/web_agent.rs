/// web_agent 工具实现
/// 
/// 执行网页自动化操作（点击、输入等）

use tauri::{AppHandle, Manager};
use tracing::info;

use crate::error::{AppError, AppResult};
use crate::ai::tools::{ToolCall, ToolResult};
use crate::state::AppState;

/// 执行 web_agent 工具
pub async fn execute(app: &AppHandle, tool_call: &ToolCall) -> AppResult<ToolResult> {
    let args = &tool_call.arguments;
    
    // 提取参数
    let action = args.get("action")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Internal("Missing action parameter".into()))?
        .to_string();
    
    let selector = args.get("selector")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Internal("Missing selector parameter".into()))?
        .to_string();
    
    let value = args.get("value")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    
    // 获取当前活跃标签页 ID
    let tab_id = get_active_tab_id(app).await?;
    
    // 执行网页操作
    // 调用现有的 execute_web_action 命令
    let result = crate::commands::page_context::execute_web_action(
        app.clone(),
        tab_id,
        action,
        selector,
        value,
    ).await?;
    
    Ok(ToolResult {
        tool_call_id: tool_call.id.clone(),
        output: result,
        success: true,
    })
}

/// 获取当前活跃标签页 ID
async fn get_active_tab_id(app: &AppHandle) -> AppResult<String> {
    info!("  🔍 Getting active tab ID from AppState...");
    
    if let Some(state) = app.try_state::<AppState>() {
        info!("  ✅ AppState found");
        if let Ok(tab_id_guard) = state.active_tab_id.lock() {
            info!("  ✅ Lock acquired");
            if let Some(tab_id) = tab_id_guard.as_ref() {
                info!("  ✅ Active tab ID found: {}", tab_id);
                return Ok(tab_id.clone());
            } else {
                info!("  ❌ Active tab ID is None");
            }
        } else {
            use tracing::error;
            error!("  ❌ Failed to acquire lock on active_tab_id");
        }
    } else {
        use tracing::error;
        error!("  ❌ AppState not found");
    }
    
    // 如果没有活跃标签页，返回错误
    use tracing::error;
    error!("  ❌ No active tab found");
    Err(AppError::Internal("No active tab found".into()))
}
