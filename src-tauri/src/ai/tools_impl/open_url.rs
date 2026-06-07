/// open_url 工具实现
/// 
/// 用于在浏览器中打开指定的 URL

use tauri::{AppHandle, Emitter, Manager, Listener};
use serde_json::json;
use tracing::{info, error, warn};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::error::{AppError, AppResult};
use crate::ai::tools::{ToolCall, ToolResult};
use crate::state::AppState;

/// 执行 open_url 工具
pub async fn execute(app: &AppHandle, tool_call: &ToolCall) -> AppResult<ToolResult> {
    info!("🌐 Executing open_url tool");
    
    let args = &tool_call.arguments;
    
    // 提取 URL 参数
    let url = args.get("url")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            error!("❌ Missing url parameter");
            AppError::Internal("Missing url parameter".into())
        })?
        .to_string();
    
    info!("  URL: {}", url);
    
    // 验证 URL 格式
    if !url.starts_with("http://") && !url.starts_with("https://") {
        error!("❌ Invalid URL format: {}", url);
        return Err(AppError::Internal("URL must start with http:// or https://".into()));
    }
    info!("✅ URL format validated");
    
    // 检查是否是重复请求（相同的 URL 在 5 秒内）
    if let Some(state) = app.try_state::<AppState>() {
        if let Ok(mut recent_urls_guard) = state.recent_opened_urls.lock() {
            let now = Instant::now();
            let recent_urls = &mut *recent_urls_guard;
            
            // 清理过期记录（超过 5 秒的）
            recent_urls.retain(|_, timestamp| now.duration_since(*timestamp) < Duration::from_secs(5));
            
            // 检查是否已经打开过这个 URL
            if let Some(last_opened) = recent_urls.get(&url) {
                let elapsed = now.duration_since(*last_opened);
                warn!("⚠️  Duplicate open_url request for '{}' (opened {:?} ago)", url, elapsed);
                return Ok(ToolResult {
                    tool_call_id: tool_call.id.clone(),
                    output: format!("该网页已在最近打开（{:.1} 秒前），无需重复打开。\nURL: {}", elapsed.as_secs_f64(), url),
                    success: true,
                });
            }
            
            // 记录这次打开
            recent_urls.insert(url.clone(), now);
            info!("✅ Recorded URL opening: {}", url);
        }
    }
    
    // 通知前端创建新标签页并导航
    info!("  Requesting new tab from frontend...");
    if let Some(window) = app.get_webview_window("main") {
        // 生成唯一的请求 ID
        let request_id = format!("open_url_{}", chrono::Utc::now().timestamp_millis());
        
        // 发送创建标签页请求
        window.emit("webview:create-tab", json!({
            "requestId": request_id,
            "url": url.clone(),
            "title": "加载中..."
        })).map_err(|e| AppError::Internal(format!("Failed to emit event: {}", e)))?;
        
        info!("✅ Event sent, waiting for new tab ID...");
        // 等待前端返回新标签页 ID
        let new_tab_id = wait_for_new_tab_id(app, &request_id).await?;
        info!("✅ New tab created with ID: {}", new_tab_id);
        
        // 设置新标签页为活跃标签页
        if let Some(state) = app.try_state::<AppState>() {
            crate::commands::browser_nav::set_active_tab(new_tab_id.clone(), state).await?;
        }
        
        // 额外延迟以确保前端完成焦点切换
        tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
        
        Ok(ToolResult {
            tool_call_id: tool_call.id.clone(),
            output: format!("成功打开网页: {} (新标签页)", url),
            success: true,
        })
    } else {
        Err(AppError::Internal("Main window not found".into()))
    }
}

/// 等待前端返回新标签页 ID
async fn wait_for_new_tab_id(app: &AppHandle, request_id: &str) -> AppResult<String> {
    use tokio::sync::oneshot;
    
    // 创建一个 oneshot channel 来接收标签页 ID
    let (tx, rx) = oneshot::channel::<String>();
    
    // 【关键修复】监听正确的事件名称：cosurf:new-tab-response
    let app_clone = app.clone();
    let tx_clone = std::sync::Arc::new(std::sync::Mutex::new(Some(tx)));
    let request_id_clone = request_id.to_string();
    
    let unlisten = app_clone.listen("cosurf:new-tab-response", move |event| {
        info!("📥 Received cosurf:new-tab-response event");
        if let Ok(payload) = serde_json::from_str::<serde_json::Value>(&event.payload()) {
            // 检查是否是我们要的请求
            if let Some(req_id) = payload.get("requestId").and_then(|v| v.as_str()) {
                if req_id == request_id_clone {
                    if let Some(tab_id) = payload.get("tabId").and_then(|v| v.as_str()) {
                        info!("✅ New tab ID received: {}", tab_id);
                        if let Some(sender) = tx_clone.lock().unwrap().take() {
                            let _ = sender.send(tab_id.to_string());
                        }
                    }
                }
            }
        }
    });
    
    // 等待响应，设置超时（增加到 15 秒，给前端更多时间）
    let result = tokio::time::timeout(
        tokio::time::Duration::from_secs(15),
        rx
    ).await;
    
    // 取消监听
    let _ = unlisten;
    
    match result {
        Ok(Ok(tab_id)) => Ok(tab_id),
        Ok(Err(_)) => Err(AppError::Internal("Channel closed".into())),
        Err(_) => Err(AppError::Internal("Timeout waiting for new tab ID (15s)".into())),
    }
}
