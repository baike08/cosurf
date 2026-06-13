use tauri::{AppHandle, Emitter, Manager, State};
use tauri::utils::config::WebviewUrl;
use tracing::info;

use crate::db::history::{AddHistoryRequest, HistoryEntry};
use crate::error::ErrorResponse;
use crate::state::AppState;

#[tauri::command]
pub fn list_history(
    state: State<'_, AppState>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<HistoryEntry>, ErrorResponse> {
    let db = state.db.lock().map_err(|e| ErrorResponse {
        code: "LOCK_ERROR".into(),
        message: e.to_string(),
    })?;
    db.list_history(limit.unwrap_or(50), offset.unwrap_or(0))
        .map_err(|e| ErrorResponse::from(e))
}

#[tauri::command]
pub fn search_history(
    state: State<'_, AppState>,
    query: String,
    limit: Option<i64>,
) -> Result<Vec<HistoryEntry>, ErrorResponse> {
    let db = state.db.lock().map_err(|e| ErrorResponse {
        code: "LOCK_ERROR".into(),
        message: e.to_string(),
    })?;
    db.search_history(&query, limit.unwrap_or(50))
        .map_err(|e| ErrorResponse::from(e))
}

#[tauri::command]
pub fn add_history(
    state: State<'_, AppState>,
    request: AddHistoryRequest,
) -> Result<HistoryEntry, ErrorResponse> {
    let db = state.db.lock().map_err(|e| ErrorResponse {
        code: "LOCK_ERROR".into(),
        message: e.to_string(),
    })?;
    db.add_history(&request).map_err(|e| ErrorResponse::from(e))
}

#[tauri::command]
pub fn clear_history(state: State<'_, AppState>) -> Result<(), ErrorResponse> {
    let db = state.db.lock().map_err(|e| ErrorResponse {
        code: "LOCK_ERROR".into(),
        message: e.to_string(),
    })?;
    db.clear_history().map_err(|e| ErrorResponse::from(e))
}

#[tauri::command]
pub fn delete_history_entry(state: State<'_, AppState>, id: String) -> Result<(), ErrorResponse> {
    let db = state.db.lock().map_err(|e| ErrorResponse {
        code: "LOCK_ERROR".into(),
        message: e.to_string(),
    })?;
    db.delete_history_entry(&id).map_err(|e| ErrorResponse::from(e))
}

/// 在 CoSurf 新标签页中打开 URL（替代 shell.open）
/// 前端通过 postMessage 或 invoke 调用，在应用内创建新标签页
#[tauri::command]
pub async fn open_in_cosurf_tab(app: AppHandle, url: String) -> Result<(), ErrorResponse> {
    info!("📎 open_in_cosurf_tab: {}", url);

    // 验证 URL 格式
    if url.is_empty() {
        return Err(ErrorResponse {
            code: "INVALID_URL".into(),
            message: "URL cannot be empty".into(),
        });
    }

    // 忽略诊断 URL（链接拦截器注入确认）
    if url.starts_with("cosurf:") {
        info!("📎 Diagnostic message: {}", url);
        return Ok(());
    }

    // 从 URL 提取标题
    let title = url::Url::parse(&url)
        .ok()
        .map(|u| u.host_str().unwrap_or("新标签页").to_string())
        .unwrap_or_else(|| "新标签页".into());

    // 发送事件到前端创建新标签页
    app.emit("webview:create-tab", serde_json::json!({
        "requestId": format!("open_tab_{}", chrono::Utc::now().timestamp_millis()),
        "url": url,
        "title": title,
    }))
    .map_err(|e| ErrorResponse {
        code: "EMIT_ERROR".into(),
        message: format!("Failed to emit create-tab event: {}", e),
    })?;

    Ok(())
}

/// 在系统默认浏览器中打开 URL
#[tauri::command]
pub async fn open_in_system_browser(url: String) -> Result<(), ErrorResponse> {
    info!("🌐 open_in_system_browser: {}", url);
    open::that(&url).map_err(|e| ErrorResponse {
        code: "OPEN_ERROR".into(),
        message: format!("Failed to open URL in system browser: {}", e),
    })?;
    Ok(())
}

/// 检查 webview 中 Tauri API 是否可用
#[tauri::command]
pub async fn check_webview_tauri_api(
    app: AppHandle,
    webview_label: String,
) -> Result<String, ErrorResponse> {
    info!("🔍 check_webview_tauri_api: label={}", webview_label);

    let webview = app
        .get_webview_window(&webview_label)
        .ok_or_else(|| ErrorResponse {
            code: "WEBVIEW_NOT_FOUND".into(),
            message: format!("Webview '{}' not found", webview_label),
        })?;

    // 检查 __TAURI_INTERNALS__ 和 __TAURI__ 是否存在
    let script = r#"
        (function() {
            return JSON.stringify({
                hasTauriInternals: typeof window.__TAURI_INTERNALS__ !== 'undefined',
                hasTauriGlobal: typeof window.__TAURI__ !== 'undefined',
                hasTauriCore: typeof window.__TAURI__ !== 'undefined' && typeof window.__TAURI__.core !== 'undefined',
                hasInvoke: typeof window.__TAURI_INTERNALS__ !== 'undefined' && typeof window.__TAURI_INTERNALS__.invoke === 'function',
                location: window.location.href,
                title: document.title
            });
        })()
    "#;

    // 尝试使用 eval 并捕获结果
    let check_script = format!(
        "var r={}; window.__cosurf_check_result = r; document.title = '[CoSurf-DBG]' + r;",
        script
    );

    webview.eval(&check_script).map_err(|e| ErrorResponse {
        code: "EVAL_ERROR".into(),
        message: format!("Failed to evaluate script: {}", e),
    })?;

    Ok("check_executed".to_string())
}

/// 从 Rust 端创建标签页 WebviewWindow（解决动态 webview 无 __TAURI_INTERNALS__ 的问题）
#[tauri::command]
pub async fn create_tab_webview(
    app: AppHandle,
    tab_id: String,
    url: String,
    _x: f64,
    _y: f64,
    _width: f64,
    _height: f64,
) -> Result<String, ErrorResponse> {
    let label = format!("tab-{}", tab_id.replace(|c: char| !c.is_alphanumeric() && c != '-' && c != '_', "_"));
    info!("📺 create_tab_webview: label={}, url={}", label, url);

    let parsed_url = url::Url::parse(&url).map_err(|e| ErrorResponse {
        code: "INVALID_URL".into(),
        message: format!("Invalid URL: {}", e),
    })?;

    // 为每个闭包单独 clone AppHandle
    let new_window_app = app.clone();
    let nav_label = label.clone();
    let page_load_label = label.clone();
    let return_label = label.clone();

    // JS 链接拦截脚本 — 通过 window.open 触发 on_new_window 回调
    let init_script = r#"
(function() {
    document.addEventListener('click', function(e) {
        var el = e.target;
        while (el && el.tagName !== 'A') el = el.parentElement;
        if (el && el.href && el.href !== '' && !el.href.startsWith('javascript:') && el.target !== '_self') {
            e.preventDefault();
            e.stopPropagation();
            window.open(el.href, '_blank');
        }
    }, true);
})();
"#;

    // 获取主窗口作为父窗口
    let parent_window = app.get_webview_window("main");
    
    let mut builder = tauri::WebviewWindowBuilder::new(&app, &label, tauri::utils::config::WebviewUrl::External(parsed_url))
        .decorations(false)
        .skip_taskbar(true)
        .focused(false)
        .visible(true)
        .initialization_script(init_script)
        // 拦截 window.open() 和 target="_blank"
        .on_new_window(move |_url, _features| {
            let url_str = _url.to_string();
            info!("📺 on_new_window intercepted: {}", url_str);
            let app_clone = new_window_app.clone();
            tauri::async_runtime::spawn(async move {
                let title = url::Url::parse(&url_str)
                    .ok()
                    .map(|u| u.host_str().unwrap_or("新标签页").to_string())
                    .unwrap_or_else(|| "新标签页".into());
                let _ = app_clone.emit("webview:create-tab", serde_json::json!({
                    "url": url_str,
                    "title": title,
                }));
            });
            tauri::webview::NewWindowResponse::Deny
        })
        // 拦截所有导航（仅日志，不拦截；新标签页创建由 on_new_window 处理）
        .on_navigation(move |nav_url| {
            info!("📺 [{}] Navigation: {}", nav_label, nav_url);
            true // 始终允许导航（重定向、表单提交等都在当前 webview 中正常进行）
        })
        // 页面加载完成日志
        .on_page_load(move |_webview, payload| {
            info!("📺 [{}] Page loaded: {}", page_load_label, payload.url());
        });

    // 如果有主窗口，设置为子窗口
    if let Some(parent) = parent_window {
        builder = builder.parent(&parent).map_err(|e| ErrorResponse::from(crate::error::AppError::Tauri(e)))?;
    }

    let _webview_window = builder.build().map_err(|e| ErrorResponse {
        code: "BUILD_ERROR".into(),
        message: format!("Failed to build webview window: {}", e),
    })?;

    info!("📺 WebviewWindow created successfully: {}", return_label);
    Ok(return_label)
}

/// 关闭标签页 WebviewWindow
#[tauri::command]
pub async fn close_tab_webview(
    app: AppHandle,
    tab_id: String,
) -> Result<(), ErrorResponse> {
    let label = format!("tab-{}", tab_id.replace(|c: char| !c.is_alphanumeric() && c != '-' && c != '_', "_"));
    info!("📺 close_tab_webview: label={}", label);

    if let Some(window) = app.get_webview_window(&label) {
        window.destroy().map_err(|e| ErrorResponse {
            code: "CLOSE_ERROR".into(),
            message: format!("Failed to destroy webview: {}", e),
        })?;
        info!("📺 WebviewWindow destroyed: {}", label);
    } else {
        tracing::warn!("📺 WebviewWindow not found for close: {}", label);
    }

    Ok(())
}

/// 在指定 webview 中执行 JavaScript 脚本（用于链接拦截、页面内容获取等）
#[tauri::command]
pub async fn eval_in_webview(
    app: AppHandle,
    webview_label: String,
    script: String,
) -> Result<String, ErrorResponse> {
    info!("📝 eval_in_webview: label={}, script_len={}", webview_label, script.len());

    let webview = app
        .get_webview_window(&webview_label)
        .ok_or_else(|| ErrorResponse {
            code: "WEBVIEW_NOT_FOUND".into(),
            message: format!("Webview '{}' not found", webview_label),
        })?;

    webview
        .eval(&script)
        .map_err(|e| ErrorResponse {
            code: "EVAL_ERROR".into(),
            message: format!("Failed to evaluate script: {}", e),
        })?;

    Ok("ok".to_string())
}
