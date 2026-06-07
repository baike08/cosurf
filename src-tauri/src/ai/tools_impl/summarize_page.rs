/// summarize_page 工具实现
/// 
/// 提取当前页面内容并使用 AI 进行总结
/// 支持多种提取方式：iframe -> Playwright -> HTTP fallback

use tauri::{AppHandle, Emitter, Manager, Listener};
use serde_json::json;
use tracing::{info, warn, error};

use crate::error::{AppError, AppResult};
use crate::ai::tools::{ToolCall, ToolResult};
use crate::ai::provider::{build_chat_request, build_headers, get_api_url, ChatMessage, ChatResponse};
use crate::ai::playwright_client::PlaywrightClient;
use crate::state::AppState;

/// 执行 summarize_page 工具
pub async fn execute(app: &AppHandle, tool_call: &ToolCall) -> AppResult<ToolResult> {
    info!("📄 Executing summarize_page tool");
    
    let args = &tool_call.arguments;
    
    // 提取参数
    let max_length = args.get("max_length")
        .and_then(|v| v.as_u64())
        .unwrap_or(500) as usize;
    info!("  max_length: {}", max_length);
    
    // 获取当前活跃标签页 ID 和 URL
    info!("  Getting active tab info...");
    let (tab_id, tab_url) = get_active_tab_info(app).await?;
    info!("  Active tab: {} - {}", tab_id, tab_url);
    
    // 【混合策略】尝试多种方式提取页面内容
    let content = extract_page_content_hybrid(app, &tab_id, &tab_url).await?;
    
    if content.is_empty() {
        // 内容为空，提供友好的错误提示
        warn!("⚠️  Page content is empty after all extraction attempts");
        Ok(ToolResult {
            tool_call_id: tool_call.id.clone(),
            output: "❌ 无法自动提取页面内容\n\n原因：该网站有严格的安全防护（CSP/X-Frame-Options/反爬虫），禁止第三方工具读取内容。\n\n解决方案：\n1. 点击'在系统浏览器中打开'按钮\n2. 在系统浏览器中选中并复制您想总结的内容\n3. 回到这里，粘贴内容并说'请总结以下内容'\n\n或者，您可以尝试使用 web_agent 工具进行自动化操作。".to_string(),
            success: false,
        })
    } else {
        // 使用 AI 总结内容
        info!("✅ Extracted page content, length: {}", content.len());
        let summary = summarize_with_ai(app, &content, max_length).await?;
        
        Ok(ToolResult {
            tool_call_id: tool_call.id.clone(),
            output: summary,
            success: true,
        })
    }
}

/// 获取当前活跃标签页 ID 和 URL
async fn get_active_tab_info(app: &AppHandle) -> AppResult<(String, String)> {
    info!("  🔍 Getting active tab info from AppState...");
    
    if let Some(state) = app.try_state::<AppState>() {
        info!("  ✅ AppState found");
        
        // 获取 active_tab_id
        let tab_id = {
            if let Ok(tab_id_guard) = state.active_tab_id.lock() {
                if let Some(id) = tab_id_guard.as_ref() {
                    id.clone()
                } else {
                    return Err(AppError::Internal("No active tab found".into()));
                }
            } else {
                return Err(AppError::Internal("Failed to lock active_tab_id".into()));
            }
        };
        
        // 【关键修复】从前端获取真实的 URL
        info!("  📡 Requesting tab URL from frontend...");
        let tab_url = get_tab_url_from_frontend(app, &tab_id).await.unwrap_or_else(|e| {
            warn!("⚠️  Failed to get URL from frontend: {}, using about:blank", e);
            "about:blank".to_string()
        });
        
        info!("  ✅ Active tab: {} - {}", tab_id, tab_url);
        Ok((tab_id, tab_url))
    } else {
        error!("  ❌ AppState not found");
        Err(AppError::Internal("App state not found".into()))
    }
}

/// 从前端获取标签页 URL
async fn get_tab_url_from_frontend(app: &AppHandle, tab_id: &str) -> AppResult<String> {
    use tokio::sync::oneshot;
    use tauri::{Emitter, Listener};
    
    let (tx, rx) = oneshot::channel::<String>();
    let tx_clone = std::sync::Arc::new(std::sync::Mutex::new(Some(tx)));
    
    // 监听前端响应
    let unlisten = app.listen("cosurf:tab-url-response", move |event| {
        info!("📥 Received tab-url-response event");
        if let Ok(payload) = serde_json::from_str::<serde_json::Value>(&event.payload()) {
            if let Some(url) = payload.get("url").and_then(|v| v.as_str()) {
                info!("✅ Received tab URL: {}", url);
                if let Some(sender) = tx_clone.lock().unwrap().take() {
                    let _ = sender.send(url.to_string());
                }
            }
        }
    });
    
    // 发送请求给前端
    if let Some(window) = app.get_webview_window("main") {
        window.emit("webview:get-tab-url", serde_json::json!({
            "tabId": tab_id
        })).map_err(|e| AppError::Internal(format!("Failed to emit event: {}", e)))?;
        
        info!("📤 Sent webview:get-tab-url request");
    } else {
        return Err(AppError::Internal("Main window not found".into()));
    }
    
    // 等待响应（3秒超时）
    let result = tokio::time::timeout(
        tokio::time::Duration::from_secs(3),
        rx
    ).await;
    
    // 取消监听
    let _ = unlisten;
    
    match result {
        Ok(Ok(url)) => Ok(url),
        Ok(Err(_)) => Err(AppError::Internal("Channel closed".into())),
        Err(_) => Err(AppError::Internal("Timeout waiting for tab URL".into())),
    }
}

/// 【混合策略】尝试多种方式提取页面内容
async fn extract_page_content_hybrid(
    app: &AppHandle,
    tab_id: &str,
    tab_url: &str,
) -> AppResult<String> {
    info!("🔍 Starting hybrid content extraction for: {}", tab_url);
    
    // 跳过 about:blank 和内部 URL
    if tab_url.starts_with("about:") || tab_url.starts_with("chrome:") {
        warn!("⚠️  Skipping internal URL: {}", tab_url);
        return Ok(String::new());
    }
    
    // === 第一步：尝试从前端 iframe 读取 ===
    info!("📡 Attempt 1: Extracting from iframe...");
    match try_extract_from_iframe(app, tab_id).await {
        Ok(content) if !content.is_empty() => {
            info!("✅ Iframe extraction successful, length: {}", content.len());
            return Ok(content);
        }
        Ok(_) => {
            warn!("⚠️  Iframe extraction returned empty content (likely cross-origin)");
        }
        Err(e) => {
            warn!("⚠️  Iframe extraction failed: {}", e);
        }
    }
    
    // === 第二步：使用 Playwright 无头浏览器 ===
    info!("🎭 Attempt 2: Using Playwright headless browser...");
    match try_extract_with_playwright(tab_url).await {
        Ok(content) if !content.is_empty() => {
            info!("✅ Playwright extraction successful, length: {}", content.len());
            return Ok(content);
        }
        Ok(_) => {
            warn!("⚠️  Playwright extraction returned empty content");
        }
        Err(e) => {
            warn!("⚠️  Playwright extraction failed: {}", e);
        }
    }
    
    // === 第三步：降级到 HTTP 请求 ===
    info!("🌐 Attempt 3: Using HTTP request fallback...");
    match try_extract_via_http(tab_url).await {
        Ok(content) if !content.is_empty() => {
            info!("✅ HTTP extraction successful, length: {}", content.len());
            return Ok(content);
        }
        Ok(_) => {
            warn!("⚠️  HTTP extraction returned empty content");
        }
        Err(e) => {
            warn!("⚠️  HTTP extraction failed: {}", e);
        }
    }
    
    // 所有方法都失败
    error!("❌ All extraction methods failed");
    Ok(String::new())
}

/// 尝试从 iframe 提取内容
async fn try_extract_from_iframe(app: &AppHandle, tab_id: &str) -> AppResult<String> {
    info!("  📤 Sending webview:get-content event to frontend");
    
    if let Some(window) = app.get_webview_window("main") {
        window.emit("webview:get-content", json!({
            "tabId": tab_id,
            "script": r#"
                (function() {
                    const clone = document.body.cloneNode(true);
                    clone.querySelectorAll('script, style, noscript, nav, footer').forEach(el => el.remove());
                    return clone.innerText.trim().substring(0, 10000);
                })()
            "#
        })).map_err(|e| AppError::Internal(format!("Failed to emit event: {}", e)))?;
        
        // 等待前端返回内容
        wait_for_page_content(app, tab_id).await
    } else {
        Err(AppError::Internal("Main window not found".into()))
    }
}

/// 等待前端返回页面内容
async fn wait_for_page_content(app: &AppHandle, _tab_id: &str) -> AppResult<String> {
    use tokio::sync::oneshot;
    
    // 创建一个 oneshot channel 来接收内容
    let (tx, rx) = oneshot::channel::<String>();
    
    // 存储 sender 到全局状态（简化实现）
    // TODO: 使用更好的方式管理多个并发请求
    
    // 监听 cosurf:page-content-response 事件
    let app_clone = app.clone();
    let tx_clone = std::sync::Arc::new(std::sync::Mutex::new(Some(tx)));
    
    let unlisten = app_clone.listen("cosurf:page-content-response", move |event| {
        info!("📥 Received page-content-response event");
        info!("  Event payload: {}", event.payload());
        if let Ok(payload) = serde_json::from_str::<serde_json::Value>(&event.payload()) {
            info!("  Parsed payload: {:?}", payload);
            
            // 检查是否有错误
            if let Some(error) = payload.get("error") {
                if !error.is_null() {
                    warn!("⚠️  Page content extraction error: {}", error);
                    // 即使有错误，也尝试使用 null 数据
                }
            }
            
            // 获取 data 字段
            if let Some(data) = payload.get("data") {
                if let Some(content) = data.as_str() {
                    info!("✅ Received page content, length: {}", content.len());
                    if let Some(sender) = tx_clone.lock().unwrap().take() {
                        let _ = sender.send(content.to_string());
                    }
                } else if data.is_null() {
                    // data 为 null，发送空字符串
                    info!("⚠️  Page content is null (cross-origin restriction)");
                    if let Some(sender) = tx_clone.lock().unwrap().take() {
                        let _ = sender.send(String::new());
                    }
                } else {
                    warn!("⚠️  Unexpected data type: {:?}", data);
                }
            } else {
                warn!("⚠️  No 'data' field in payload");
            }
        } else {
            error!("❌ Failed to parse payload as JSON");
        }
    });
    
    // 等待响应，设置超时（增加到10秒，给前端更多时间）
    let result = tokio::time::timeout(
        tokio::time::Duration::from_secs(10),
        rx
    ).await;
    
    // 取消监听
    let _ = unlisten;
    
    match result {
        Ok(Ok(content)) => Ok(content),
        Ok(Err(_)) => Err(AppError::Internal("Channel closed".into())),
        Err(_) => Err(AppError::Internal("Timeout waiting for page content".into())),
    }
}

/// 尝试使用 Playwright 提取内容
async fn try_extract_with_playwright(url: &str) -> AppResult<String> {
    info!("  🎭 Launching Playwright browser...");
    
    let client = PlaywrightClient::new();
    
    // 使用 markdown 格式获取内容（更适合 AI 总结）
    match client.fetch_page_content(url, "markdown", None).await {
        Ok(response) => {
            info!("  ✅ Playwright extracted content from: {}", response.url);
            Ok(response.content)
        }
        Err(e) => {
            error!("  ❌ Playwright failed: {}", e);
            Err(AppError::Internal(format!("Playwright extraction failed: {}", e)))
        }
    }
}

/// 尝试使用 HTTP 请求提取内容
async fn try_extract_via_http(url: &str) -> AppResult<String> {
    info!("  🌐 Sending HTTP request to: {}", url);
    
    use reqwest::Client;
    
    let client = Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| AppError::Internal(format!("Failed to create HTTP client: {}", e)))?;
    
    let response = client.get(url)
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("HTTP request failed: {}", e)))?;
    
    let html = response.text()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to read response: {}", e)))?;
    
    // 简单提取文本内容（移除 HTML 标签）
    let text = extract_text_from_html(&html);
    
    info!("  ✅ HTTP extracted {} characters", text.len());
    Ok(text)
}

/// 从 HTML 中提取纯文本（简单实现）
fn extract_text_from_html(html: &str) -> String {
    // 使用正则表达式移除 HTML 标签
    let re = regex::Regex::new(r"<[^>]*>").unwrap();
    let text = re.replace_all(html, " ");
    
    // 清理多余空白和特殊字符
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .filter(|c| !c.is_control())  // 移除控制字符
        .take(10000)  // 限制长度
        .collect()
}

/// 使用 AI 总结页面内容
async fn summarize_with_ai(app: &AppHandle, content: &str, max_length: usize) -> AppResult<String> {
    // 获取模型配置
    let model_config = {
        if let Some(state) = app.try_state::<AppState>() {
            if let Ok(db) = state.db.lock() {
                db.get_active_model_config()
                    .map_err(|e| AppError::Internal(e.to_string()))?
                    .ok_or_else(|| AppError::Internal("No active model configured".into()))?
            } else {
                return Err(AppError::Internal("Database locked".into()));
            }
        } else {
            return Err(AppError::Internal("App state not found".into()));
        }
    };
    
    // 构建总结请求
    let system_prompt = format!(
        "你是一个专业的页面内容总结器。请根据提供的网页内容，生成一个简洁、准确的总结（不超过 {} 字）。\n\n要求：\n- 只返回总结文本，不要包含任何解释或额外内容\n- 总结应该概括页面的核心主题和关键信息\n- 使用中文\n- 保持简洁明了",
        max_length
    );
    
    let messages = vec![
        ChatMessage {
            role: "system".into(),
            content: system_prompt,
            name: None,
            tool_call_id: None,
        },
        ChatMessage {
            role: "user".into(),
            content: format!("请总结以下网页内容：\n\n{}", content),
            name: None,
            tool_call_id: None,
        },
    ];
    
    // 调用非流式 API 获取总结
    use reqwest::Client;
    
    let url = get_api_url(&model_config)?;
    let headers = build_headers(&model_config)?;
    
    let request_body = build_chat_request(&model_config, messages);
    let mut non_stream_request = request_body.clone();
    non_stream_request.stream = false;
    
    let client = Client::new();
    let mut request = client.post(&url);
    
    for (key, value) in &headers {
        request = request.header(key.as_str(), value.as_str());
    }
    
    let body_json = serde_json::to_string(&non_stream_request)?;
    
    let response = request.body(body_json).send().await
        .map_err(|e| AppError::Internal(format!("Network error: {}", e)))?;
    
    let chat_response: ChatResponse = response.json().await
        .map_err(|e| AppError::Internal(format!("Parse error: {}", e)))?;
    
    // 提取总结
    let summary = chat_response.choices.first()
        .map(|c| c.message.content.trim().to_string())
        .unwrap_or_else(|| "无法生成总结".to_string());
    
    Ok(summary)
}
