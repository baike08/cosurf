use rusqlite::params;
use serde::{Deserialize, Serialize};

use crate::db::Database;
use crate::error::AppResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEntry {
    pub id: String,
    pub title: String,
    pub url: String,
    pub visited_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddHistoryRequest {
    pub title: String,
    pub url: String,
}

impl Database {
    pub fn list_history(&self, limit: i64, offset: i64) -> AppResult<Vec<HistoryEntry>> {
        let mut stmt = self.conn().prepare(
            "SELECT id, title, url, visited_at FROM history
             ORDER BY visited_at DESC LIMIT ?1 OFFSET ?2",
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
        for row in rows {
            entries.push(row?);
        }
        Ok(entries)
    }

    pub fn search_history(&self, query: &str, limit: i64) -> AppResult<Vec<HistoryEntry>> {
        let pattern = format!("%{}%", query);
        let mut stmt = self.conn().prepare(
            "SELECT id, title, url, visited_at FROM history
             WHERE title LIKE ?1 OR url LIKE ?1
             ORDER BY visited_at DESC LIMIT ?2",
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
        for row in rows {
            entries.push(row?);
        }
        Ok(entries)
    }

    pub fn add_history(&self, req: &AddHistoryRequest) -> AppResult<HistoryEntry> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        // 先删除相同 URL 的旧记录（URL 去重）
        self.conn().execute(
            "DELETE FROM history WHERE url = ?1",
            params![req.url],
        )?;

        // 插入新记录
        self.conn().execute(
            "INSERT INTO history (id, title, url, visited_at) VALUES (?1, ?2, ?3, ?4)",
            params![id, req.title, req.url, now],
        )?;

        Ok(HistoryEntry {
            id,
            title: req.title.clone(),
            url: req.url.clone(),
            visited_at: now,
        })
    }

    pub fn clear_history(&self) -> AppResult<()> {
        self.conn().execute("DELETE FROM history", [])?;
        Ok(())
    }

    pub fn delete_history_entry(&self, id: &str) -> AppResult<()> {
        self.conn().execute("DELETE FROM history WHERE id = ?1", params![id])?;
        Ok(())
    }
}
