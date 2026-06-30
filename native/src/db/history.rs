//! 浏览历史（History）模块

use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::error::AppResult;

/// 历史记录结构
#[derive(Debug, Clone)]
pub struct HistoryEntry {
    pub id: String,
    pub title: String,
    pub url: String,
    pub visited_at: String,
}

/// 创建历史表
pub fn create_history_table(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS history (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            url TEXT NOT NULL,
            visited_at TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_history_visited ON history(visited_at DESC);
        CREATE INDEX IF NOT EXISTS idx_history_url ON history(url);
        "#
    )?;
    
    Ok(())
}

/// 列出历史记录（分页）
pub fn list_history(conn: &Connection, limit: i64, offset: i64) -> AppResult<Vec<HistoryEntry>> {
    let mut stmt = conn.prepare(
        "SELECT id, title, url, visited_at FROM history ORDER BY visited_at DESC LIMIT ?1 OFFSET ?2"
    )?;
    
    let rows = stmt.query_map(params![limit, offset], |row| {
        Ok(HistoryEntry {
            id: row.get(0)?,
            title: row.get(1)?,
            url: row.get(2)?,
            visited_at: row.get(3)?,
        })
    })?;
    
    let mut entries = Vec::new();
    for entry in rows {
        entries.push(entry?);
    }
    
    Ok(entries)
}

/// 搜索历史记录
pub fn search_history(conn: &Connection, query: &str, limit: i64) -> AppResult<Vec<HistoryEntry>> {
    let pattern = format!("%{}%", query);
    
    let mut stmt = conn.prepare(
        "SELECT id, title, url, visited_at FROM history
         WHERE title LIKE ?1 OR url LIKE ?1 ORDER BY visited_at DESC LIMIT ?2"
    )?;
    
    let rows = stmt.query_map(params![pattern, limit], |row| {
        Ok(HistoryEntry {
            id: row.get(0)?,
            title: row.get(1)?,
            url: row.get(2)?,
            visited_at: row.get(3)?,
        })
    })?;
    
    let mut entries = Vec::new();
    for entry in rows {
        entries.push(entry?);
    }
    
    Ok(entries)
}

/// 添加历史记录
pub fn add_history(conn: &Connection, title: String, url: String) -> AppResult<HistoryEntry> {
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    
    conn.execute(
        "INSERT INTO history (id, title, url, visited_at) VALUES (?1, ?2, ?3, ?4)",
        params![id, title, url, now],
    )?;
    
    Ok(HistoryEntry {
        id,
        title,
        url,
        visited_at: now,
    })
}

/// 清空所有历史记录
pub fn clear_history(conn: &Connection) -> AppResult<()> {
    conn.execute("DELETE FROM history", [])?;
    Ok(())
}

/// 删除单条历史记录
pub fn delete_history_entry(conn: &Connection, id: &str) -> AppResult<()> {
    conn.execute("DELETE FROM history WHERE id = ?1", params![id])?;
    Ok(())
}
