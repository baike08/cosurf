//! 页面缓存模块 (N-API)
//!
//! 从 src-tauri/src/commands/page_cache.rs 迁移。
//! 提供页面内容的文件缓存（SHA256 哈希文件名），支持过期清理。

use napi::bindgen_prelude::*;
use sha2::{Sha256, Digest};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::fs;
use tracing::{info, warn};
use serde::{Deserialize, Serialize};

// ===== 全局缓存目录 =====
lazy_static::lazy_static! {
    static ref CACHE_DIR: Mutex<PathBuf> = Mutex::new(PathBuf::from("."));
}

/// 页面缓存数据结构
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PageCache {
    pub url: String,
    pub title: String,
    pub content: String,
    pub timestamp: i64,
    pub content_length: usize,
}

/// 初始化缓存目录（由 lib.rs native_init 调用）
pub fn init_cache(app_data_dir: &str) -> Result<()> {
    let cache_dir = Path::new(app_data_dir).join("page-cache");
    if !cache_dir.exists() {
        fs::create_dir_all(&cache_dir)
            .map_err(|e| Error::from_reason(format!("Failed to create cache dir: {}", e)))?;
        info!(path = ?cache_dir, "Created page cache directory");
    }
    let mut dir = CACHE_DIR.lock()
        .map_err(|e| Error::from_reason(format!("Lock error: {}", e)))?;
    *dir = cache_dir;
    Ok(())
}

/// 根据 URL/Key 生成缓存文件名（使用 SHA256 哈希）
fn generate_cache_filename(key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    let result = hasher.finalize();
    format!("{:x}.json", result)
}

fn get_cache_dir() -> Result<PathBuf> {
    let dir = CACHE_DIR.lock()
        .map_err(|e| Error::from_reason(format!("Lock error: {}", e)))?;
    Ok(dir.clone())
}

// ============================================================
// N-API 导出
// ============================================================

/// 保存缓存 — N-API
///
/// JS 调用: `native.cacheSave(key, data)`  (camelCase)
#[napi]
pub fn cache_save(key: String, data: String) -> Result<String> {
    let dir = get_cache_dir()?;
    if !dir.exists() {
        fs::create_dir_all(&dir)
            .map_err(|e| Error::from_reason(format!("Failed to create cache dir: {}", e)))?;
    }

    let filename = generate_cache_filename(&key);
    let filepath = dir.join(&filename);

    // data 已经是 JSON 字符串，直接写入
    fs::write(&filepath, &data)
        .map_err(|e| Error::from_reason(format!("Failed to write cache: {}", e)))?;

    info!(key = %key, path = ?filepath, len = data.len(), "Saved page cache");
    Ok(filepath.to_string_lossy().to_string())
}

/// 加载缓存 — N-API
///
/// JS 调用: `native.cacheLoad(key)`  (camelCase)
/// 返回 JSON 字符串，找不到时返回 null (Option<String>)
#[napi]
pub fn cache_load(key: String) -> Result<Option<String>> {
    let dir = get_cache_dir()?;
    let filename = generate_cache_filename(&key);
    let filepath = dir.join(&filename);

    if !filepath.exists() {
        info!(key = %key, "Page cache not found");
        return Ok(None);
    }

    match fs::read_to_string(&filepath) {
        Ok(content) => {
            info!(key = %key, len = content.len(), "Loaded page cache");
            Ok(Some(content))
        }
        Err(e) => {
            warn!(key = %key, error = %e, "Failed to read page cache");
            Ok(None)
        }
    }
}

/// 清理过期缓存 — N-API
///
/// JS 调用: `native.cacheCleanup(maxAgeSeconds)`  (camelCase)
/// max_age_seconds 为 null 时默认 86400 (24小时)
/// 返回清理的文件数量
#[napi]
pub fn cache_cleanup(max_age_seconds: Option<i64>) -> Result<i64> {
    let max_age = max_age_seconds.unwrap_or(86400); // 默认 24 小时
    let dir = get_cache_dir()?;

    if !dir.exists() {
        return Ok(0);
    }

    let mut cleaned_count: i64 = 0;

    let entries = fs::read_dir(&dir)
        .map_err(|e| Error::from_reason(format!("Failed to read cache dir: {}", e)))?;

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }

        if let Ok(metadata) = fs::metadata(&path) {
            if let Ok(modified) = metadata.modified() {
                if let Ok(elapsed) = modified.elapsed() {
                    let age_seconds = elapsed.as_secs() as i64;
                    if age_seconds > max_age {
                        if fs::remove_file(&path).is_ok() {
                            cleaned_count += 1;
                            info!(path = ?path, age_seconds = age_seconds, "Removed expired cache");
                        }
                    }
                }
            }
        }
    }

    info!(cleaned_count = cleaned_count, "Cleaned up expired cache files");
    Ok(cleaned_count)
}
