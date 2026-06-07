use std::path::{Path, PathBuf};
use std::fs;
use serde::{Deserialize, Serialize};
use chrono::Utc;
use sha2::{Sha256, Digest};
use tracing::{info, warn, error};
use tauri::Manager;

/// 页面缓存数据结构
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PageCache {
    pub url: String,
    pub title: String,
    pub content: String,
    pub timestamp: i64,
    pub content_length: usize,
}

/// 获取用户数据目录路径
pub fn get_user_data_dir(custom_path: Option<&str>) -> PathBuf {
    if let Some(path) = custom_path {
        if !path.is_empty() {
            return PathBuf::from(path);
        }
    }
    
    // 默认使用系统临时目录
    let temp_dir = std::env::temp_dir();
    temp_dir.join("cosurf").join("data").join("pages")
}

/// 确保用户数据目录存在
pub fn ensure_user_data_dir(custom_path: Option<&str>) -> Result<PathBuf, String> {
    let dir = get_user_data_dir(custom_path);
    
    if !dir.exists() {
        fs::create_dir_all(&dir).map_err(|e| {
            format!("Failed to create user data directory: {}", e)
        })?;
        info!(path = ?dir, "Created user data directory");
    }
    
    Ok(dir)
}

/// 根据 URL 生成缓存文件名（使用 SHA256 哈希）
fn generate_cache_filename(url: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(url.as_bytes());
    let result = hasher.finalize();
    format!("{:x}.json", result)
}

/// 保存页面内容到缓存文件
pub fn save_page_cache(
    custom_path: Option<&str>,
    url: &str,
    title: &str,
    content: &str,
) -> Result<PathBuf, String> {
    let dir = ensure_user_data_dir(custom_path)?;
    let filename = generate_cache_filename(url);
    let filepath = dir.join(&filename);
    
    let cache = PageCache {
        url: url.to_string(),
        title: title.to_string(),
        content: content.to_string(),
        timestamp: Utc::now().timestamp(),
        content_length: content.len(),
    };
    
    let json = serde_json::to_string_pretty(&cache).map_err(|e| {
        format!("Failed to serialize page cache: {}", e)
    })?;
    
    fs::write(&filepath, json).map_err(|e| {
        format!("Failed to write page cache file: {}", e)
    })?;
    
    info!(
        url = %url,
        path = ?filepath,
        content_length = content.len(),
        "Saved page cache"
    );
    
    Ok(filepath)
}

/// 从缓存文件读取页面内容
pub fn load_page_cache(custom_path: Option<&str>, url: &str) -> Option<PageCache> {
    let dir = get_user_data_dir(custom_path);
    let filename = generate_cache_filename(url);
    let filepath = dir.join(&filename);
    
    if !filepath.exists() {
        info!(url = %url, "Page cache not found");
        return None;
    }
    
    match fs::read_to_string(&filepath) {
        Ok(json) => {
            match serde_json::from_str::<PageCache>(&json) {
                Ok(cache) => {
                    info!(
                        url = %url,
                        age_seconds = Utc::now().timestamp() - cache.timestamp,
                        "Loaded page cache"
                    );
                    Some(cache)
                }
                Err(e) => {
                    warn!(url = %url, error = %e, "Failed to parse page cache");
                    None
                }
            }
        }
        Err(e) => {
            warn!(url = %url, error = %e, "Failed to read page cache file");
            None
        }
    }
}

/// 清理过期的缓存文件（超过 24 小时）
pub fn cleanup_expired_cache(custom_path: Option<&str>, max_age_seconds: i64) -> Result<usize, String> {
    let dir = ensure_user_data_dir(custom_path)?;
    let now = Utc::now().timestamp();
    let mut cleaned_count = 0;
    
    for entry in fs::read_dir(&dir).map_err(|e| {
        format!("Failed to read cache directory: {}", e)
    })? {
        let entry = entry.map_err(|e| {
            format!("Failed to read directory entry: {}", e)
        })?;
        
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            if let Ok(metadata) = fs::metadata(&path) {
                if let Ok(modified) = metadata.modified() {
                    if let Ok(elapsed) = modified.elapsed() {
                        let age_seconds = elapsed.as_secs() as i64;
                        if age_seconds > max_age_seconds {
                            if fs::remove_file(&path).is_ok() {
                                cleaned_count += 1;
                                info!(path = ?path, age_seconds = age_seconds, "Removed expired cache");
                            }
                        }
                    }
                }
            }
        }
    }
    
    info!(cleaned_count = cleaned_count, "Cleaned up expired cache files");
    Ok(cleaned_count)
}

/// Tauri 命令：保存页面缓存
#[tauri::command]
pub async fn save_page_cache_command(
    app: tauri::AppHandle,
    url: String,
    title: String,
    content: String,
) -> Result<String, String> {
    // 获取用户配置的路径
    let user_data_path = if let Some(state) = app.try_state::<crate::state::AppState>() {
        if let Ok(db) = state.db.lock() {
            db.get_setting("user_data_path")
                .ok()
                .flatten()
                .unwrap_or_default()
        } else {
            String::new()
        }
    } else {
        String::new()
    };
    
    let custom_path = if user_data_path.is_empty() {
        None
    } else {
        Some(user_data_path.as_str())
    };
    
    match save_page_cache(custom_path, &url, &title, &content) {
        Ok(path) => Ok(path.to_string_lossy().to_string()),
        Err(e) => Err(e),
    }
}

/// Tauri 命令：加载页面缓存
#[tauri::command]
pub async fn load_page_cache_command(
    app: tauri::AppHandle,
    url: String,
) -> Result<Option<PageCache>, String> {
    // 获取用户配置的路径
    let user_data_path = if let Some(state) = app.try_state::<crate::state::AppState>() {
        if let Ok(db) = state.db.lock() {
            db.get_setting("user_data_path")
                .ok()
                .flatten()
                .unwrap_or_default()
        } else {
            String::new()
        }
    } else {
        String::new()
    };
    
    let custom_path = if user_data_path.is_empty() {
        None
    } else {
        Some(user_data_path.as_str())
    };
    
    Ok(load_page_cache(custom_path, &url))
}

/// Tauri 命令：清理过期缓存
#[tauri::command]
pub async fn cleanup_expired_cache_command(
    app: tauri::AppHandle,
    max_age_seconds: Option<i64>,
) -> Result<usize, String> {
    let max_age = max_age_seconds.unwrap_or(86400); // 默认 24 小时
    
    // 获取用户配置的路径
    let user_data_path = if let Some(state) = app.try_state::<crate::state::AppState>() {
        if let Ok(db) = state.db.lock() {
            db.get_setting("user_data_path")
                .ok()
                .flatten()
                .unwrap_or_default()
        } else {
            String::new()
        }
    } else {
        String::new()
    };
    
    let custom_path = if user_data_path.is_empty() {
        None
    } else {
        Some(user_data_path.as_str())
    };
    
    cleanup_expired_cache(custom_path, max_age)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_generate_cache_filename() {
        let url1 = "https://www.baidu.com";
        let url2 = "https://www.baidu.com";
        let url3 = "https://www.google.com";
        
        assert_eq!(generate_cache_filename(url1), generate_cache_filename(url2));
        assert_ne!(generate_cache_filename(url1), generate_cache_filename(url3));
    }
    
    #[test]
    fn test_get_user_data_dir() {
        let dir = get_user_data_dir(None);
        assert!(dir.ends_with("cosurf/data/pages"));
    }
}
