pub mod ai;
pub mod commands;
pub mod db;
pub mod error;
pub mod state;

use tauri::Manager;
use tracing::info;
use tracing_subscriber::EnvFilter;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

use crate::db::Database;
use crate::state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to get app data directory");

            info!("=== CoSurf Application Starting ===");
            info!("Application data directory: {:?}", app_data_dir);
            
            let db_path = app_data_dir.join("cosurf.db");
            info!("Database file path: {:?}", db_path);
            info!("Initializing database...");
            
            let database = Database::new(&app_data_dir)
                .expect("Failed to initialize database");
            
            info!("Database initialized successfully");
            
            let skills_dir = app_data_dir.join("skills");
            info!("Skills directory: {:?}", skills_dir);

            info!("Initializing application state...");
            app.manage(AppState::new(database, app_data_dir));
            info!("Application state initialized successfully");

            // 注册全局截图快捷键 Ctrl+Shift+X
            info!("Registering global screenshot shortcut: Control+Shift+X");
            let shortcut_handle = app.handle().clone();
            app.global_shortcut().on_shortcut("Control+Shift+X", move |_app, _shortcut, event| {
                if event.state == ShortcutState::Pressed {
                    // 发送事件到前端，触发全屏截图
                    let h = shortcut_handle.clone();
                    tauri::async_runtime::spawn(async move {
                        if let Err(e) = commands::screenshot::capture_full_screen(h).await {
                            tracing::error!("Screenshot failed: {:?}", e);
                        }
                    });
                }
            }).map_err(|e| format!("Failed to register screenshot shortcut: {}", e))?;
            info!("Global shortcut Control+Shift+X registered successfully for screenshot");

            // 自动更新检查
            #[cfg(feature = "updater")]
            {
                info!("Starting automatic update check...");
                let handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    check_for_updates(handle).await;
                });
            }

            info!("=== CoSurf backend initialized successfully ===");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // 对话管理
            commands::conversation::list_conversations,
            commands::conversation::get_conversation,
            commands::conversation::create_conversation,
            commands::conversation::update_conversation,
            commands::conversation::delete_conversation,
            commands::conversation::get_conversation_with_messages,
            // 消息管理
            commands::message::list_messages,
            commands::message::get_message,
            commands::message::create_message,
            commands::message::update_message,
            commands::message::delete_message,
            commands::message::append_message_content,
            commands::message::complete_message,
            commands::message::set_message_feedback,
            // 书签管理
            commands::bookmark::list_bookmarks,
            commands::bookmark::create_bookmark,
            commands::bookmark::delete_bookmark,
            commands::bookmark::list_bookmark_folders,
            commands::bookmark::create_bookmark_folder,
            commands::bookmark::delete_bookmark_folder,
            // 设置
            commands::settings::get_settings,
            commands::settings::get_setting,
            commands::settings::set_setting,
            commands::settings::list_model_configs,
            commands::settings::get_model_config,
            commands::settings::get_active_model,
            commands::settings::create_model_config,
            commands::settings::update_model_config,
            commands::settings::set_active_model,
            commands::settings::delete_model_config,
            // Skills 配置
            commands::settings::get_skills_directory,
            commands::settings::set_skills_directory,
            // IQS API Key 配置
            commands::settings::get_iqs_api_key,
            commands::settings::set_iqs_api_key,
            // MCP Server 配置
            commands::settings::list_mcp_servers,
            commands::settings::get_mcp_server,
            commands::settings::create_mcp_server,
            commands::settings::update_mcp_server,
            commands::settings::delete_mcp_server,
            commands::settings::test_mcp_server,
            commands::settings::import_mcp_servers_from_json,
            // AI
            commands::ai::send_chat_message,
            commands::ai::stop_generation,
            commands::ai::append_stream_chunk,
            commands::ai::complete_stream,
            commands::ai::generate_conversation_title,
            // AI Agent
            commands::ai_agent::agent_execute,
            commands::ai_agent::configure_qwen_model,
            commands::ai_agent::generate_page_summary,
            commands::ai_agent::extract_memory,
            // 浏览历史
            commands::browser::list_history,
            commands::browser::search_history,
            commands::browser::add_history,
            commands::browser::clear_history,
            commands::browser::delete_history_entry,
            // WebView 导航
            commands::browser_nav::browser_navigate,
            commands::browser_nav::browser_reload,
            commands::browser_nav::browser_go_back,
            commands::browser_nav::browser_go_forward,
            commands::browser_nav::browser_get_state,
            commands::browser_nav::browser_execute_script,
            commands::browser_nav::browser_get_page_content,
            commands::browser_nav::browser_screenshot,
            commands::browser_nav::browser_close_tab,
            // 浏览器操作
            commands::browser_nav::browser_toggle_select_mode,
            commands::browser_nav::browser_click_element,
            commands::browser_nav::browser_input_text,
            commands::browser_nav::browser_scroll,
            commands::browser_nav::set_active_tab,
            commands::browser_nav::get_webview_title,
            // 页面上下文（AI 用）
            commands::page_context::get_page_context,
            commands::page_context::inject_page_context,
            commands::page_context::summarize_page,
            commands::page_context::execute_web_action,
            commands::page_context::receive_page_content,
            // 页面缓存
            commands::page_cache::save_page_cache_command,
            commands::page_cache::load_page_cache_command,
            commands::page_cache::cleanup_expired_cache_command,
            // 截图
            commands::screenshot::capture_full_screen,
            commands::screenshot::capture_region_from_base64,
            commands::screenshot::save_screenshot,
            commands::screenshot::copy_screenshot_to_clipboard,
            // Skills 管理
            commands::skills::list_skills,
            commands::skills::delete_skill,
            commands::skills::toggle_skill,
            commands::skills::import_skill_from_markdown,
            commands::skills::import_skill_from_directory,
            commands::skills::list_skill_files,
            commands::skills::get_skill_content,
        ])
        .run(tauri::generate_context!())
        .expect("Failed to run CoSurf");
}

/// 检查并安装更新
#[cfg(feature = "updater")]
async fn check_for_updates(app: tauri::AppHandle) {
    use tauri_plugin_updater::UpdaterExt;

    info!("Checking for application updates...");
    match app.updater() {
        Ok(Some(updater)) => {
            match updater.check().await {
                Ok(Some(update)) => {
                    info!("New version available: {}", update.version);

                    // 通知前端有新更新
                    let _ = app.emit("updater:update-available", &update.version);

                    // 下载并安装（静默）
                    info!("Downloading and installing update: {}", update.version);
                    if let Err(e) = update.download_and_install(|_, _| {}, || {}).await {
                        tracing::error!("Failed to install update: {}", e);
                    } else {
                        info!("Update installed successfully");
                    }
                }
                Ok(None) => {
                    info!("No updates available, running latest version");
                }
                Err(e) => {
                    tracing::error!("Failed to check for updates: {}", e);
                }
            }
        }
        Ok(None) => {
            tracing::warn!("Updater not configured properly");
        }
        Err(e) => {
            tracing::error!("Failed to create updater: {}", e);
        }
    }
}
