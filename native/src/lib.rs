//! CoSurf Native Module
//!
//! Rust N-API 原生模块，为 Electron 提供高性能计算能力：
//! - SQLite 数据库操作
//! - AI Agent 调度与流式对话
//! - Skills 管理
//! - 截图功能
//! - 页面缓存
//!
//! 通过 napi-rs 编译为 .node 文件，供 Node.js 主进程加载。

#[macro_use]
extern crate napi_derive;

pub mod db;
pub mod error;
pub mod ai;
pub mod screenshot;
pub mod cache;

use napi::bindgen_prelude::*;
use std::sync::Once;

static INIT: Once = Once::new();

/// 初始化 Native 模块（应用启动时由 Electron 主进程调用）
#[napi]
pub fn native_init(app_data_dir: String, skills_dir: Option<String>) -> Result<()> {
    INIT.call_once(|| {
        // 初始化日志
        tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
            )
            .init();

        tracing::info!("=== CoSurf Native Module Initializing ===");
    });

    // 初始化数据库
    db::init_database(&app_data_dir)?;

    // 初始化 Skills 管理器
    let skills_path = if let Some(dir) = skills_dir {
        dir
    } else {
        // 尝试从数据库读取配置
        match db::db_get_setting("skills.directory".to_string()) {
            Ok(Some(dir)) if !dir.trim().is_empty() => {
                tracing::info!(path = %dir, "Loaded skills directory from database");
                dir
            }
            _ => {
                // 使用默认路径
                let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
                let default_path = home.join(".cosurf").join("skills").to_string_lossy().to_string();
                tracing::info!(path = %default_path, "Using default skills directory");
                default_path
            }
        }
    };
    ai::agent::init_skills_manager(&skills_path);
    let _ = ai::agent::load_skills();

    // 自动加载所有启用的 MCP Servers
    std::thread::spawn(|| {
        match db::db_list_mcp_servers() {
            Ok(servers_json) => {
                tracing::info!("📦 Loaded MCP servers from database: {} bytes", servers_json.len());
                match serde_json::from_str::<Vec<crate::ai::mcp_manager::McpServerConfig>>(&servers_json) {
                    Ok(servers) => {
                        let enabled_servers: Vec<crate::ai::mcp_manager::McpServerConfig> = 
                            servers.into_iter().filter(|s| s.enabled).collect();
                        if !enabled_servers.is_empty() {
                            tracing::info!("🔄 Auto-loading {} enabled MCP servers at startup", enabled_servers.len());
                            let rt = tokio::runtime::Runtime::new().unwrap();
                            rt.block_on(async {
                                crate::ai::agent::load_mcp_servers(enabled_servers).await;
                            });
                        } else {
                            tracing::info!("ℹ️ No enabled MCP servers found in database");
                        }
                    }
                    Err(e) => tracing::error!("❌ Failed to parse MCP servers from database: {}", e),
                }
            }
            Err(e) => tracing::error!("❌ Failed to load MCP servers from database: {}", e),
        }
    });

    // 初始化缓存
    cache::init_cache(&app_data_dir)?;

    tracing::info!("CoSurf Native Module initialized successfully");
    Ok(())
}

/// 获取 Native 模块版本
#[napi]
pub fn native_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// 加载所有启用的 MCP Servers
#[napi]
pub fn mcp_load_servers(servers_json: String) -> Result<()> {
    tracing::info!("🚀🚀🚀 mcp_load_servers called with {} bytes", servers_json.len());
    
    // 解析 JSON
    let servers: Vec<crate::ai::mcp_manager::McpServerConfig> = 
        serde_json::from_str(&servers_json)
            .map_err(|e| Error::from_reason(format!("Failed to parse servers JSON: {}", e)))?;
    
    tracing::info!("📦 Parsed {} MCP server configs", servers.len());
    
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            crate::ai::agent::load_mcp_servers(servers).await;
        });
    });
    
    Ok(())
}
