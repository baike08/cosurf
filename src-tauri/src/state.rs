use std::path::PathBuf;
use std::sync::{Arc, Mutex, atomic::AtomicBool};
use std::collections::HashMap;
use std::time::Instant;

use crate::ai::skills::SkillsManager;
use crate::db::Database;

pub struct AppState {
    pub db: Mutex<Database>,
    pub app_data_dir: PathBuf,
    pub cancel_flag: Arc<AtomicBool>,
    pub active_tab_id: Arc<Mutex<Option<String>>>,
    /// 存储页面内容提取的响应 (request_id -> content)
    pub page_content_responses: Arc<Mutex<HashMap<String, String>>>,
    /// Skills 管理器
    pub skills_manager: Arc<Mutex<SkillsManager>>,
    /// 最近打开的 URL 记录 (URL -> 打开时间)，用于去重
    pub recent_opened_urls: Arc<Mutex<HashMap<String, Instant>>>,
    /// MCP 工具注册表: mcp_tool_function_name -> (server_name, original_mcp_tool_name)
    /// 用于 Agent Loop dispatcher 将 mcp_{server}_{tool} 调用路由到正确的 MCP server
    pub mcp_tool_registry: Arc<Mutex<HashMap<String, (String, String)>>>,
}

impl AppState {
    pub fn new(db: Database, app_data_dir: PathBuf) -> Self {
        // 从数据库获取 Skills 目录配置（如果不存在则使用默认值）
        let skills_dir_str = db.get_skills_directory()
            .unwrap_or_else(|_| {
                // 如果出错，使用默认路径
                let default_path = app_data_dir.join("skills");
                tracing::info!(path = ?default_path, "Using default skills directory");
                default_path.to_string_lossy().to_string()
            });
        
        let skills_dir = PathBuf::from(&skills_dir_str);
        tracing::info!(path = ?skills_dir, "Initializing SkillsManager with directory");
        
        // 确保目录存在
        if !skills_dir.exists() {
            tracing::info!(path = ?skills_dir, "Creating skills directory");
            if let Err(e) = std::fs::create_dir_all(&skills_dir) {
                tracing::error!(error = %e, path = ?skills_dir, "Failed to create skills directory");
            }
        }
        
        let mut skills_manager = SkillsManager::new(skills_dir.clone());
        
        // 同步示例 Skills 到 Skills 目录（确保示例是最新版本）
        match skills_manager.seed_example_skills() {
            Ok(count) => {
                tracing::info!(count, "Seeded example skills");
            }
            Err(e) => {
                tracing::warn!(error = %e, "Failed to seed example skills (non-fatal)");
            }
        }
        
        // 加载已有的 Skills
        match skills_manager.load_skills_from_directory() {
            Ok(count) => {
                tracing::info!(count, path = ?skills_dir, "Loaded skills from directory");
            }
            Err(e) => {
                tracing::error!(error = %e, "Failed to load skills");
            }
        }
        
        Self { 
            db: Mutex::new(db), 
            app_data_dir,
            cancel_flag: Arc::new(AtomicBool::new(false)),
            active_tab_id: Arc::new(Mutex::new(None)),
            page_content_responses: Arc::new(Mutex::new(HashMap::new())),
            skills_manager: Arc::new(Mutex::new(skills_manager)),
            recent_opened_urls: Arc::new(Mutex::new(HashMap::new())),
            mcp_tool_registry: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}
