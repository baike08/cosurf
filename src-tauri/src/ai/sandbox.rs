/// Sandbox 沙箱环境
/// 提供安全的执行环境,用于运行 CLI 命令和存储数据

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{info, warn};

use crate::error::{AppError, AppResult};

/// Sandbox 配置
#[derive(Debug, Clone)]
pub struct SandboxConfig {
    /// 沙箱根目录
    pub root_dir: PathBuf,
    /// 是否允许执行 CLI 命令
    pub allow_cli: bool,
    /// 允许的命令白名单
    pub allowed_commands: Vec<String>,
}

impl SandboxConfig {
    pub fn new(app_data_dir: &Path) -> Self {
        let root_dir = app_data_dir.join("sandbox");
        
        // 创建必要的子目录
        let _ = fs::create_dir_all(&root_dir);
        let _ = fs::create_dir_all(root_dir.join("web_pages"));
        let _ = fs::create_dir_all(root_dir.join("summaries"));
        let _ = fs::create_dir_all(root_dir.join("memories"));
        let _ = fs::create_dir_all(root_dir.join("history"));
        
        Self {
            root_dir,
            allow_cli: true,
            allowed_commands: vec![
                "ls".into(),
                "cat".into(),
                "echo".into(),
                "pwd".into(),
                "find".into(),
            ],
        }
    }
}

/// Sandbox 管理器
pub struct Sandbox {
    config: SandboxConfig,
}

impl Sandbox {
    pub fn new(config: SandboxConfig) -> Self {
        Self { config }
    }

    /// 保存网页内容
    pub fn save_web_page(&self, url: &str, content: &str, timestamp: i64) -> AppResult<()> {
        let filename = self.url_to_filename(url);
        let filepath = self.config.root_dir.join("web_pages").join(format!("{}.json", filename));
        
        let data = serde_json::json!({
            "url": url,
            "content": content,
            "timestamp": timestamp,
            "saved_at": chrono::Utc::now().to_rfc3339()
        });
        
        fs::write(&filepath, serde_json::to_string_pretty(&data)?)
            .map_err(|e| AppError::Internal(format!("Failed to save web page: {}", e)))?;
        
        info!(url = %url, "Web page saved to sandbox");
        Ok(())
    }

    /// 加载网页内容
    pub fn load_web_page(&self, url: &str) -> AppResult<Option<String>> {
        let filename = self.url_to_filename(url);
        let filepath = self.config.root_dir.join("web_pages").join(format!("{}.json", filename));
        
        if !filepath.exists() {
            return Ok(None);
        }
        
        let content = fs::read_to_string(&filepath)
            .map_err(|e| AppError::Internal(format!("Failed to read web page: {}", e)))?;
        
        let data: serde_json::Value = serde_json::from_str(&content)?;
        Ok(data.get("content").and_then(|v| v.as_str()).map(|s| s.to_string()))
    }

    /// 清理过期的网页内容(保留最近30天)
    pub fn cleanup_old_pages(&self, days: i64) -> AppResult<usize> {
        let cutoff = chrono::Utc::now().timestamp() - (days * 24 * 3600);
        let pages_dir = self.config.root_dir.join("web_pages");
        
        let mut count = 0;
        if let Ok(entries) = fs::read_dir(&pages_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Ok(data) = serde_json::from_str::<serde_json::Value>(&content) {
                            if let Some(timestamp) = data.get("timestamp").and_then(|v| v.as_i64()) {
                                if timestamp < cutoff {
                                    let _ = fs::remove_file(&path);
                                    count += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
        
        info!(count = count, "Cleaned up old web pages");
        Ok(count)
    }

    /// 保存网页摘要
    pub fn save_summary(&self, url: &str, summary: &str) -> AppResult<()> {
        let filename = self.url_to_filename(url);
        let filepath = self.config.root_dir.join("summaries").join(format!("{}.md", filename));
        
        let content = format!(
            "# Summary\n\n**URL:** {}\n\n**Generated At:** {}\n\n---\n\n{}\n",
            url,
            chrono::Utc::now().to_rfc3339(),
            summary
        );
        
        fs::write(&filepath, content)
            .map_err(|e| AppError::Internal(format!("Failed to save summary: {}", e)))?;
        
        info!(url = %url, "Summary saved to sandbox");
        Ok(())
    }

    /// 加载网页摘要
    pub fn load_summary(&self, url: &str) -> AppResult<Option<String>> {
        let filename = self.url_to_filename(url);
        let filepath = self.config.root_dir.join("summaries").join(format!("{}.md", filename));
        
        if !filepath.exists() {
            return Ok(None);
        }
        
        let content = fs::read_to_string(&filepath)
            .map_err(|e| AppError::Internal(format!("Failed to read summary: {}", e)))?;
        
        Ok(Some(content))
    }

    /// 保存记忆
    pub fn save_memory(&self, key: &str, value: &str, category: &str) -> AppResult<()> {
        let filepath = self.config.root_dir.join("memories").join(format!("{}.json", key));
        
        let data = serde_json::json!({
            "key": key,
            "value": value,
            "category": category,
            "created_at": chrono::Utc::now().to_rfc3339(),
            "updated_at": chrono::Utc::now().to_rfc3339()
        });
        
        fs::write(&filepath, serde_json::to_string_pretty(&data)?)
            .map_err(|e| AppError::Internal(format!("Failed to save memory: {}", e)))?;
        
        info!(key = %key, "Memory saved to sandbox");
        Ok(())
    }

    /// 加载记忆
    pub fn load_memory(&self, key: &str) -> AppResult<Option<String>> {
        let filepath = self.config.root_dir.join("memories").join(format!("{}.json", key));
        
        if !filepath.exists() {
            return Ok(None);
        }
        
        let content = fs::read_to_string(&filepath)
            .map_err(|e| AppError::Internal(format!("Failed to read memory: {}", e)))?;
        
        let data: serde_json::Value = serde_json::from_str(&content)?;
        Ok(data.get("value").and_then(|v| v.as_str()).map(|s| s.to_string()))
    }

    /// 搜索记忆
    pub fn search_memories(&self, query: &str) -> AppResult<Vec<(String, String)>> {
        let memories_dir = self.config.root_dir.join("memories");
        let mut results = Vec::new();
        
        if let Ok(entries) = fs::read_dir(&memories_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Ok(data) = serde_json::from_str::<serde_json::Value>(&content) {
                            let key = data.get("key").and_then(|v| v.as_str()).unwrap_or("").to_string();
                            let value = data.get("value").and_then(|v| v.as_str()).unwrap_or("").to_string();
                            
                            if key.contains(query) || value.contains(query) {
                                results.push((key, value));
                            }
                        }
                    }
                }
            }
        }
        
        Ok(results)
    }

    /// 执行 CLI 命令(受限)
    pub fn execute_command(&self, command: &str, args: &[&str]) -> AppResult<String> {
        if !self.config.allow_cli {
            return Err(AppError::Internal("CLI execution is disabled".into()));
        }
        
        // 检查命令是否在白名单中
        if !self.config.allowed_commands.contains(&command.to_string()) {
            return Err(AppError::Internal(format!(
                "Command '{}' is not allowed. Allowed commands: {:?}",
                command, self.config.allowed_commands
            )));
        }
        
        info!(command = %command, args = ?args, "Executing CLI command in sandbox");
        
        let output = Command::new(command)
            .args(args)
            .current_dir(&self.config.root_dir)
            .output()
            .map_err(|e| AppError::Internal(format!("Failed to execute command: {}", e)))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AppError::Internal(format!("Command failed: {}", stderr)));
        }
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.to_string())
    }

    /// 将 URL 转换为安全的文件名
    fn url_to_filename(&self, url: &str) -> String {
        url.replace(|c: char| !c.is_alphanumeric() && c != '-' && c != '_', "_")
    }
}
