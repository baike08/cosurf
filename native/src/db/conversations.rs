//! 会话（Conversations）模块

use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: String,
    pub title: String,
    pub is_pinned: bool,
    pub model_id: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

pub fn create_conversations_table(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS conversations (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL DEFAULT '新对话',
            is_pinned INTEGER NOT NULL DEFAULT 0,
            model_id TEXT,
            created_at INTEGER DEFAULT (strftime('%s', 'now') * 1000),
            updated_at INTEGER DEFAULT (strftime('%s', 'now') * 1000)
        );
        
        CREATE INDEX IF NOT EXISTS idx_conversations_updated ON conversations(updated_at DESC);
        CREATE INDEX IF NOT EXISTS idx_conversations_pinned ON conversations(is_pinned DESC);
        "#
    )?;
    
    Ok(())
}

pub fn list_conversations(conn: &Connection) -> AppResult<Vec<Conversation>> {
    let mut stmt = conn.prepare(
        "SELECT id, title, is_pinned, model_id, created_at, updated_at 
         FROM conversations 
         ORDER BY is_pinned DESC, updated_at DESC"
    )?;
    
    let rows = stmt.query_map([], |row| {
        Ok(Conversation {
            id: row.get(0)?,
            title: row.get(1)?,
            is_pinned: row.get(2)?,
            model_id: row.get(3)?,
            created_at: row.get(4)?,
            updated_at: row.get(5)?,
        })
    })?;
    
    let mut conversations = Vec::new();
    for conv in rows {
        conversations.push(conv?);
    }
    
    Ok(conversations)
}

pub fn get_conversation(conn: &Connection, id: &str) -> AppResult<Option<Conversation>> {
    let mut stmt = conn.prepare(
        "SELECT id, title, is_pinned, model_id, created_at, updated_at 
         FROM conversations WHERE id = ?"
    )?;
    
    let conv = stmt.query_row(params![id], |row| {
        Ok(Conversation {
            id: row.get(0)?,
            title: row.get(1)?,
            is_pinned: row.get(2)?,
            model_id: row.get(3)?,
            created_at: row.get(4)?,
            updated_at: row.get(5)?,
        })
    }).optional()?;
    
    Ok(conv)
}

pub fn create_conversation(
    conn: &Connection,
    title: Option<String>,
    model_id: Option<String>,
) -> AppResult<String> {
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp_millis();
    
    conn.execute(
        "INSERT INTO conversations (id, title, model_id, created_at, updated_at) VALUES (?, ?, ?, ?, ?)",
        params![
            id,
            title.unwrap_or_else(|| "新对话".to_string()),
            model_id,
            now,
            now
        ],
    )?;
    
    Ok(id)
}

pub fn update_conversation(
    conn: &Connection,
    id: &str,
    title: Option<String>,
    is_pinned: Option<bool>,
    model_id: Option<String>,
) -> AppResult<()> {
    let now = chrono::Utc::now().timestamp_millis();
    
    if let Some(title) = title {
        conn.execute(
            "UPDATE conversations SET title = ?, updated_at = ? WHERE id = ?",
            params![title, now, id],
        )?;
    }
    
    if let Some(is_pinned) = is_pinned {
        conn.execute(
            "UPDATE conversations SET is_pinned = ?, updated_at = ? WHERE id = ?",
            params![is_pinned as i32, now, id],
        )?;
    }
    
    if let Some(model_id) = model_id {
        conn.execute(
            "UPDATE conversations SET model_id = ?, updated_at = ? WHERE id = ?",
            params![model_id, now, id],
        )?;
    }
    
    Ok(())
}

pub fn delete_conversation(conn: &Connection, id: &str) -> AppResult<()> {
    // 先删除相关消息
    conn.execute("DELETE FROM messages WHERE conversation_id = ?", params![id])?;
    // 再删除会话
    conn.execute("DELETE FROM conversations WHERE id = ?", params![id])?;
    
    Ok(())
}
