use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::Emitter;
use tracing::{info, warn};

use crate::error::{AppError, AppResult};

// 全局存储每个标签页的导航状态
lazy_static::lazy_static! {
    static ref TAB_STATES: Mutex<HashMap<String, TabState>> = Mutex::new(HashMap::new());
}

#[derive(Debug, Clone)]
struct TabState {
    current_url: String,
    navigation_history: Vec<String>,
    history_index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationState {
    pub tab_id: String,
    pub url: String,
    pub title: String,
    pub can_go_back: bool,
    pub can_go_forward: bool,
    pub is_loading: bool,
}

/// 导航到指定 URL (通知前端更新 iframe)
#[tauri::command]
pub async fn browser_navigate(
    app: tauri::AppHandle,
    tab_id: String,
    url: String,
) -> AppResult<NavigationState> {
    info!(tab_id = %tab_id, url = %url, "Navigating to URL");

    // 更新标签页状态
    let mut states = TAB_STATES.lock().map_err(|e| {
        AppError::Internal(format!("Failed to lock tab states: {}", e))
    })?;

    let tab_state = states.entry(tab_id.clone()).or_insert(TabState {
        current_url: "about:blank".to_string(),
        navigation_history: vec!["about:blank".to_string()],
        history_index: 0,
    });

    // 添加新的导航历史
    if tab_state.history_index < tab_state.navigation_history.len() - 1 {
        // 如果不在历史记录的末尾,删除后面的记录
        tab_state.navigation_history.drain(tab_state.history_index + 1..);
    }
    tab_state.navigation_history.push(url.clone());
    tab_state.history_index = tab_state.navigation_history.len() - 1;
    tab_state.current_url = url.clone();

    let can_go_back = tab_state.history_index > 0;
    let can_go_forward = tab_state.history_index < tab_state.navigation_history.len() - 1;

    drop(states);

    // 通知前端更新 iframe src
    app.emit("webview:navigating", serde_json::json!({
        "tabId": tab_id,
        "url": url
    })).map_err(|e| {
        AppError::Internal(format!("Failed to emit event: {}", e))
    })?;

    Ok(NavigationState {
        tab_id,
        url,
        title: "Loading...".to_string(),
        can_go_back,
        can_go_forward,
        is_loading: true,
    })
}

/// 刷新当前页面
#[tauri::command]
pub async fn browser_reload(app: tauri::AppHandle, tab_id: String) -> AppResult<()> {
    info!(tab_id = %tab_id, "Reloading page");

    app.emit("webview:reload", serde_json::json!({ "tabId": tab_id })).map_err(|e| {
        AppError::Internal(format!("Failed to emit event: {}", e))
    })?;

    Ok(())
}

/// 后退
#[tauri::command]
pub async fn browser_go_back(app: tauri::AppHandle, tab_id: String) -> AppResult<NavigationState> {
    info!(tab_id = %tab_id, "Going back");

    let mut states = TAB_STATES.lock().map_err(|e| {
        AppError::Internal(format!("Failed to lock tab states: {}", e))
    })?;

    if let Some(tab_state) = states.get_mut(&tab_id) {
        if tab_state.history_index > 0 {
            tab_state.history_index -= 1;
            let url = tab_state.navigation_history[tab_state.history_index].clone();
            
            let can_go_back = tab_state.history_index > 0;
            let can_go_forward = tab_state.history_index < tab_state.navigation_history.len() - 1;

            drop(states);

            // 通知前端更新 iframe
            app.emit("webview:navigating", serde_json::json!({
                "tabId": tab_id,
                "url": url
            })).map_err(|e| {
                AppError::Internal(format!("Failed to emit event: {}", e))
            })?;

            return Ok(NavigationState {
                tab_id,
                url,
                title: "Loading...".to_string(),
                can_go_back,
                can_go_forward,
                is_loading: true,
            });
        }
    }

    Err(AppError::Internal("Cannot go back".into()))
}

/// 前进
#[tauri::command]
pub async fn browser_go_forward(app: tauri::AppHandle, tab_id: String) -> AppResult<NavigationState> {
    info!(tab_id = %tab_id, "Going forward");

    let mut states = TAB_STATES.lock().map_err(|e| {
        AppError::Internal(format!("Failed to lock tab states: {}", e))
    })?;

    if let Some(tab_state) = states.get_mut(&tab_id) {
        if tab_state.history_index < tab_state.navigation_history.len() - 1 {
            tab_state.history_index += 1;
            let url = tab_state.navigation_history[tab_state.history_index].clone();
            
            let can_go_back = tab_state.history_index > 0;
            let can_go_forward = tab_state.history_index < tab_state.navigation_history.len() - 1;

            drop(states);

            // 通知前端更新 iframe
            app.emit("webview:navigating", serde_json::json!({
                "tabId": tab_id,
                "url": url
            })).map_err(|e| {
                AppError::Internal(format!("Failed to emit event: {}", e))
            })?;

            return Ok(NavigationState {
                tab_id,
                url,
                title: "Loading...".to_string(),
                can_go_back,
                can_go_forward,
                is_loading: true,
            });
        }
    }

    Err(AppError::Internal("Cannot go forward".into()))
}

/// 获取当前标签页的状态
#[tauri::command]
pub async fn browser_get_state(_app: tauri::AppHandle, tab_id: String) -> AppResult<NavigationState> {
    let states = TAB_STATES.lock().map_err(|e| {
        AppError::Internal(format!("Failed to lock tab states: {}", e))
    })?;

    if let Some(tab_state) = states.get(&tab_id) {
        let can_go_back = tab_state.history_index > 0;
        let can_go_forward = tab_state.history_index < tab_state.navigation_history.len() - 1;

        Ok(NavigationState {
            tab_id,
            url: tab_state.current_url.clone(),
            title: "Page".to_string(),
            can_go_back,
            can_go_forward,
            is_loading: false,
        })
    } else {
        Ok(NavigationState {
            tab_id,
            url: "about:blank".to_string(),
            title: "New Tab".to_string(),
            can_go_back: false,
            can_go_forward: false,
            is_loading: false,
        })
    }
}

/// 关闭标签页
#[tauri::command]
pub async fn browser_close_tab(_app: tauri::AppHandle, tab_id: String) -> AppResult<()> {
    info!(tab_id = %tab_id, "Closing tab");

    let mut states = TAB_STATES.lock().map_err(|e| {
        AppError::Internal(format!("Failed to lock tab states: {}", e))
    })?;
    states.remove(&tab_id);

    Ok(())
}

/// 执行页面 JavaScript
#[tauri::command]
pub async fn browser_execute_script(
    app: tauri::AppHandle,
    tab_id: String,
    script: String,
) -> AppResult<String> {
    info!(tab_id = %tab_id, "Executing script");

    // 通知前端执行脚本
    app.emit("webview:execute-script", serde_json::json!({
        "tabId": tab_id,
        "script": script
    })).map_err(|e| {
        AppError::Internal(format!("Failed to emit event: {}", e))
    })?;

    Ok(String::new())
}

/// 获取页面内容(用于 AI 上下文)
#[tauri::command]
pub async fn browser_get_page_content(
    app: tauri::AppHandle,
    tab_id: String,
) -> AppResult<String> {
    info!(tab_id = %tab_id, "Getting page content for AI context");

    let script = r#"
        (function() {
            const clone = document.body.cloneNode(true);
            clone.querySelectorAll('script, style, noscript').forEach(el => el.remove());
            return clone.innerText.trim().substring(0, 10000);
        })()
    "#;

    // 通知前端获取页面内容
    app.emit("webview:get-content", serde_json::json!({
        "tabId": tab_id,
        "script": script
    })).map_err(|e| {
        AppError::Internal(format!("Failed to emit event: {}", e))
    })?;

    Ok(String::new())
}

/// 截图(用于 AI 视觉理解)
#[tauri::command]
pub async fn browser_screenshot(
    _app: tauri::AppHandle,
    _tab_id: String,
    _full_page: bool,
) -> AppResult<String> {
    // TODO: 实现截图功能
    Ok(String::new())
}

/// 切换元素选择模式
#[tauri::command]
pub async fn browser_toggle_select_mode(
    app: tauri::AppHandle,
    tab_id: String,
    enabled: bool,
) -> AppResult<()> {
    info!(tab_id = %tab_id, enabled, "Toggling element select mode");

    let script = if enabled {
        r#"
        (function() {
            if (window.__cosurfSelectMode) return;
            window.__cosurfSelectMode = true;
            
            const style = document.createElement('style');
            style.id = 'cosurf-select-mode-style';
            style.textContent = `
                *[data-cosurf-highlight] {
                    outline: 2px solid #3b82f6 !important;
                    outline-offset: 2px;
                    cursor: pointer !important;
                }
            `;
            document.head.appendChild(style);
            
            document.addEventListener('mouseover', window.__cosurfMouseOverHandler = function(e) {
                if (!window.__cosurfSelectMode) return;
                e.target.setAttribute('data-cosurf-highlight', 'true');
            }, true);
            
            document.addEventListener('mouseout', window.__cosurfMouseOutHandler = function(e) {
                if (!window.__cosurfSelectMode) return;
                e.target.removeAttribute('data-cosurf-highlight');
            }, true);
            
            document.addEventListener('click', window.__cosurfClickHandler = function(e) {
                if (!window.__cosurfSelectMode) return;
                e.preventDefault();
                e.stopPropagation();
                
                const selector = generateSelector(e.target);
                window.__cosurfSelectedElement = selector;
                
                // 通知 Tauri
                if (window.__TAURI__) {
                    window.__TAURI__.event.emit('element-selected', { selector });
                }
            }, true);
            
            function generateSelector(el) {
                if (el.id) return '#' + el.id;
                if (el.className && typeof el.className === 'string') {
                    return el.tagName.toLowerCase() + '.' + el.className.trim().split(/\s+/).join('.');
                }
                return el.tagName.toLowerCase();
            }
        })()
        "#
    } else {
        r#"
        (function() {
            window.__cosurfSelectMode = false;
            const style = document.getElementById('cosurf-select-mode-style');
            if (style) style.remove();
            
            document.removeEventListener('mouseover', window.__cosurfMouseOverHandler, true);
            document.removeEventListener('mouseout', window.__cosurfMouseOutHandler, true);
            document.removeEventListener('click', window.__cosurfClickHandler, true);
            
            document.querySelectorAll('[data-cosurf-highlight]').forEach(el => {
                el.removeAttribute('data-cosurf-highlight');
            });
        })()
        "#
    };

    app.emit("webview:execute-script", serde_json::json!({
        "tabId": tab_id,
        "script": script
    })).map_err(|e| {
        AppError::Internal(format!("Failed to emit event: {}", e))
    })?;

    Ok(())
}

/// 点击元素
#[tauri::command]
pub async fn browser_click_element(
    app: tauri::AppHandle,
    tab_id: String,
    selector: String,
) -> AppResult<()> {
    info!(tab_id = %tab_id, selector = %selector, "Clicking element");

    let script = format!(
        r#"
        (function() {{
            const el = document.querySelector('{}');
            if (el) {{
                el.click();
                return true;
            }}
            return false;
        }})()
        "#,
        selector.replace('\'', "\\'")
    );

    app.emit("webview:execute-script", serde_json::json!({
        "tabId": tab_id,
        "script": script
    })).map_err(|e| {
        AppError::Internal(format!("Failed to emit event: {}", e))
    })?;

    Ok(())
}

/// 在元素中输入文本
#[tauri::command]
pub async fn browser_input_text(
    app: tauri::AppHandle,
    tab_id: String,
    selector: String,
    text: String,
) -> AppResult<()> {
    info!(tab_id = %tab_id, selector = %selector, "Inputting text");

    let script = format!(
        r#"
        (function() {{
            const el = document.querySelector('{}');
            if (el) {{
                el.value = '{}';
                el.dispatchEvent(new Event('input', {{ bubbles: true }}));
                el.dispatchEvent(new Event('change', {{ bubbles: true }}));
                return true;
            }}
            return false;
        }})()
        "#,
        selector.replace('\'', "\\'"),
        text.replace('\'', "\\'").replace('"', "\\\"")
    );

    app.emit("webview:execute-script", serde_json::json!({
        "tabId": tab_id,
        "script": script
    })).map_err(|e| {
        AppError::Internal(format!("Failed to emit event: {}", e))
    })?;

    Ok(())
}

/// 滚动页面
#[tauri::command]
pub async fn browser_scroll(
    app: tauri::AppHandle,
    tab_id: String,
    direction: String,
) -> AppResult<()> {
    info!(tab_id = %tab_id, direction = %direction, "Scrolling page");

    let script = match direction.as_str() {
        "up" => "window.scrollBy({ top: -300, behavior: 'smooth' });",
        "down" => "window.scrollBy({ top: 300, behavior: 'smooth' });",
        "left" => "window.scrollBy({ left: -300, behavior: 'smooth' });",
        "right" => "window.scrollBy({ left: 300, behavior: 'smooth' });",
        _ => "window.scrollBy({ top: 300, behavior: 'smooth' });",
    };

    app.emit("webview:execute-script", serde_json::json!({
        "tabId": tab_id,
        "script": script
    })).map_err(|e| {
        AppError::Internal(format!("Failed to emit event: {}", e))
    })?;

    Ok(())
}

/// 设置当前活跃标签页 ID（供前端调用）
#[tauri::command]
pub async fn set_active_tab(tab_id: String, state: tauri::State<'_, crate::state::AppState>) -> AppResult<()> {
    info!(tab_id = %tab_id, "Setting active tab");
    
    if let Ok(mut active_tab) = state.active_tab_id.lock() {
        *active_tab = Some(tab_id);
    }
    
    Ok(())
}

/// 获取 WebView2 窗口的标题（通过 HTTP 请求获取）
#[tauri::command]
pub async fn get_webview_title(_app: tauri::AppHandle, tab_id: String, url: String) -> AppResult<String> {
    info!(tab_id = %tab_id, url = %url, "Getting webview title via HTTP");
    
    // 通过 HTTP 请求获取网页标题
    match fetch_page_title(&url).await {
        Ok(title) => {
            if !title.is_empty() {
                info!(title = %title, "Got page title");
                Ok(title)
            } else {
                // 如果没有标题，使用主机名
                Ok(extract_hostname(&url))
            }
        }
        Err(e) => {
            warn!(error = %e, "Failed to fetch title, using hostname");
            Ok(extract_hostname(&url))
        }
    }
}

/// 通过 HTTP 请求获取网页标题
async fn fetch_page_title(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()?;
    
    let response = client.get(url).send().await?;
    let html = response.text().await?;
    
    // 简单解析 <title> 标签
    if let Some(start) = html.find("<title>") {
        if let Some(end) = html[start..].find("</title>") {
            let title = html[start + 7..start + end].trim().to_string();
            return Ok(title);
        }
    }
    
    Ok(String::new())
}

/// 从 URL 提取主机名
fn extract_hostname(url: &str) -> String {
    if let Ok(parsed) = url::Url::parse(url) {
        let hostname = parsed.host_str().unwrap_or("").to_string();
        let hostname = hostname.replace("www.", "");
        let mut chars = hostname.chars();
        match chars.next() {
            None => String::new(),
            Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
        }
    } else {
        "未知页面".to_string()
    }
}
