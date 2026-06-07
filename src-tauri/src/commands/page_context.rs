use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State};
use tracing::info;
use uuid::Uuid;

use crate::ai::provider::ChatMessage;
use crate::error::{AppError, AppResult, ErrorResponse};
use crate::state::AppState;

/// 页面上下文信息（用于 AI 对话）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageContext {
    pub url: String,
    pub title: String,
    pub domain: String,
    pub content_summary: String,
    pub is_secure: bool,
}

/// 获取当前活动标签页的页面上下文
#[tauri::command]
pub async fn get_page_context(
    app: AppHandle,
    tab_id: String,
) -> AppResult<PageContext> {
    info!(tab_id = %tab_id, "Getting page context");
    
    // 从前端获取标签页信息
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| AppError::Internal("Main window not found".into()))?;

    // 生成唯一的请求 ID
    let request_id = Uuid::new_v4().to_string();
    
    // 通知前端获取标签页信息
    window.emit("webview:get-tab-info", serde_json::json!({
        "requestId": request_id,
        "tabId": tab_id
    })).map_err(|e| {
        AppError::Internal(format!("Failed to emit event: {}", e))
    })?;
    
    // 等待前端返回信息（最多等待 3 秒）
    use tokio::time::{timeout, Duration};
    
    // 从 AppState 中获取响应
    let state = app.state::<AppState>();
    
    let tab_info: Option<serde_json::Value> = timeout(Duration::from_secs(3), async {
        loop {
            tokio::time::sleep(Duration::from_millis(50)).await;
            
            // 检查是否有响应
            if let Ok(responses) = state.page_content_responses.lock() {
                if let Some(response_str) = responses.get(&request_id) {
                    if let Ok(value) = serde_json::from_str(response_str) {
                        return Some(value);
                    }
                }
            }
        }
    }).await.ok().flatten();
    
    // 清理响应缓存
    if let Ok(mut responses) = state.page_content_responses.lock() {
        responses.remove(&request_id);
    }
    
    // 解析返回的数据
    let (url, title, is_loading) = if let Some(info) = tab_info {
        let url = info.get("url").and_then(|v| v.as_str()).unwrap_or("about:blank").to_string();
        let title = info.get("title").and_then(|v| v.as_str()).unwrap_or("未知页面").to_string();
        let is_loading = info.get("isLoading").and_then(|v| v.as_bool()).unwrap_or(false);
        (url, title, is_loading)
    } else {
        // 如果超时或失败，使用默认值
        ("about:blank".to_string(), "未知页面".to_string(), false)
    };
    
    // 提取域名
    let domain = url.parse::<url::Url>()
        .ok()
        .map(|u| u.host_str().unwrap_or("").to_string())
        .unwrap_or_default();
    
    let is_secure = url.starts_with("https://");
    
    // 构建内容摘要（如果是加载中的页面，提示用户）
    let content_summary = if is_loading {
        "页面正在加载中...".to_string()
    } else if url == "about:blank" {
        "新标签页，尚未导航到任何页面".to_string()
    } else {
        format!("当前页面: {} ({})", title, domain)
    };
    
    info!(url = %url, title = %title, "Got page context from frontend");

    Ok(PageContext {
        url,
        title,
        domain,
        content_summary,
        is_secure,
    })
}

/// 向 AI 对话注入页面上下文
#[tauri::command]
pub async fn inject_page_context(
    app: AppHandle,
    conversation_id: String,
    tab_id: String,
) -> AppResult<String> {
    let page_ctx = get_page_context(app.clone(), tab_id).await?;

    // 构建系统提示词
    let context_prompt = format!(
        r#"以下是用户当前正在浏览的页面信息，请在后续回答中参考：

页面标题: {title}
URL: {url}
域名: {domain}
安全连接: {is_secure}

页面内容摘要:
{content_summary}

用户可能会询问关于此页面的问题，请基于以上信息回答。"#,
        title = page_ctx.title,
        url = page_ctx.url,
        domain = page_ctx.domain,
        is_secure = if page_ctx.is_secure { "是" } else { "否" },
        content_summary = page_ctx.content_summary,
    );

    Ok(context_prompt)
}

/// 总结当前页面（使用AI）
#[tauri::command]
pub async fn summarize_page(
    app: AppHandle,
    state: State<'_, AppState>,
    tab_id: String,
    max_length: Option<usize>,
) -> AppResult<String> {
    info!(tab_id = %tab_id, "Summarizing page content");

    let window = app
        .get_webview_window("main")
        .ok_or_else(|| AppError::Internal("Main window not found".into()))?;

    let max_len = max_length.unwrap_or(500);

    // 生成唯一的请求 ID
    let request_id = Uuid::new_v4().to_string();
    info!(request_id = %request_id, "Generated request ID for page content extraction");

    // 通知前端提取页面内容
    window.emit("webview:get-content", serde_json::json!({
        "tabId": tab_id,
        "requestId": request_id,
        "script": r#"
            (function() {
                const clone = document.body.cloneNode(true);
                clone.querySelectorAll('script, style, noscript, nav, footer, header').forEach(el => el.remove());
                return clone.innerText.trim().substring(0, 15000);
            })()
        "#
    })).map_err(|e| {
        AppError::Internal(format!("Failed to emit event: {}", e))
    })?;

    // 等待前端返回内容（最多等待 5 秒）
    use tokio::time::{timeout, Duration};
    
    let content: Option<String> = timeout(Duration::from_secs(5), async {
        loop {
            tokio::time::sleep(Duration::from_millis(100)).await;
            
            // 检查是否有响应
            if let Ok(responses) = state.page_content_responses.lock() {
                if let Some(content) = responses.get(&request_id) {
                    return Some(content.clone());
                }
            }
        }
    }).await.ok().flatten();

    // 清理响应缓存
    if let Ok(mut responses) = state.page_content_responses.lock() {
        responses.remove(&request_id);
    }

    match content {
        Some(page_content) if !page_content.is_empty() => {
            info!(content_len = page_content.len(), "Successfully extracted page content");
            
            // 截断到指定长度
            let truncated = if page_content.chars().count() > max_len {
                page_content.chars().take(max_len).collect::<String>() + "..."
            } else {
                page_content
            };
            
            Ok(truncated)
        }
        _ => {
            info!("Failed to extract page content or content is empty");
            Err(AppError::Internal(
                "无法提取页面内容。可能的原因：\n1. 页面跨域限制\n2. 页面加载未完成\n3. 请求超时\n\n请尝试手动复制页面内容后发送给 AI 进行总结。".to_string()
            ))
        }
    }
}

/// 接收前端返回的页面内容
#[tauri::command]
pub fn receive_page_content(
    state: State<'_, AppState>,
    request_id: String,
    content: String,
) -> AppResult<()> {
    info!(request_id = %request_id, content_len = content.len(), "Received page content from frontend");
    
    if let Ok(mut responses) = state.page_content_responses.lock() {
        responses.insert(request_id, content);
    }
    
    Ok(())
}

/// 执行网页操作（点击、填写表单等）
#[tauri::command]
pub async fn execute_web_action(
    app: AppHandle,
    tab_id: String,
    action: String,
    selector: String,
    value: Option<String>,
) -> AppResult<String> {
    info!(tab_id = %tab_id, action = %action, selector = %selector, "Executing web action");

    let window = app
        .get_webview_window("main")
        .ok_or_else(|| AppError::Internal("Main window not found".into()))?;

    let script = match action.as_str() {
        "click" => {
            format!(
                r#"
                (function() {{
                    const el = document.querySelector('{}');
                    if (el) {{
                        el.click();
                        return '成功点击元素';
                    }}
                    return '未找到元素: {}';
                }})()
                "#,
                selector.replace('\'', "\\'"),
                selector
            )
        }
        "fill" => {
            let val = value.unwrap_or_default();
            format!(
                r#"
                (function() {{
                    const el = document.querySelector('{}');
                    if (el) {{
                        el.value = '{}';
                        el.dispatchEvent(new Event('input', {{ bubbles: true }}));
                        el.dispatchEvent(new Event('change', {{ bubbles: true }}));
                        return '成功填写表单';
                    }}
                    return '未找到输入框: {}';
                }})()
                "#,
                selector.replace('\'', "\\'"),
                val.replace('\'', "\\'"),
                selector
            )
        }
        "close_popup" | "dismiss" => {
            format!(
                r#"
                (function() {{
                    // 尝试关闭弹窗
                    const modal = document.querySelector('.modal, .popup, [role="dialog"]');
                    if (modal) {{
                        modal.remove();
                        return '已关闭弹窗';
                    }}
                    
                    // 尝试点击关闭按钮
                    const closeBtn = document.querySelector('.close, [aria-label="Close"], button.close');
                    if (closeBtn) {{
                        closeBtn.click();
                        return '已点击关闭按钮';
                    }}
                    
                    // 按 ESC 键
                    document.dispatchEvent(new KeyboardEvent('keydown', {{ key: 'Escape' }}));
                    return '已发送ESC键';
                }})()
                "#
            )
        }
        _ => {
            return Err(AppError::Internal(format!("不支持的操作类型: {}", action)));
        }
    };

    // 执行脚本
    window.emit("webview:execute-script", serde_json::json!({
        "tabId": tab_id,
        "script": script
    })).map_err(|e| {
        AppError::Internal(format!("Failed to emit event: {}", e))
    })?;

    Ok("操作已执行".to_string())
}
