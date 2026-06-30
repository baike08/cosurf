//! 消息（Messages）模块

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub conversation_id: String,
    pub role: String,
    pub content: String,
    pub thinking_content: Option<String>,
    pub attachments: Option<String>,
    pub feedback: Option<String>,
    pub created_at: i64,
}

pub fn create_messages_table(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS messages (
            id TEXT PRIMARY KEY,
            conversation_id TEXT NOT NULL,
            role TEXT NOT NULL,
            content TEXT NOT NULL DEFAULT '',
            thinking_content TEXT,
            attachments TEXT,
            feedback TEXT,
            created_at INTEGER DEFAULT (strftime('%s', 'now') * 1000),
            FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE CASCADE
        );
        
        CREATE INDEX IF NOT EXISTS idx_messages_conversation ON messages(conversation_id);
        CREATE INDEX IF NOT EXISTS idx_messages_created ON messages(created_at);
        "#
    )?;
    
    Ok(())
}

pub fn list_messages(conn: &Connection, conversation_id: &str) -> AppResult<Vec<Message>> {
    let mut stmt = conn.prepare(
        "SELECT id, conversation_id, role, content, thinking_content, attachments, feedback, created_at 
         FROM messages 
         WHERE conversation_id = ? 
         ORDER BY created_at ASC"
    )?;
    
    let rows = stmt.query_map(params![conversation_id], |row| {
        Ok(Message {
            id: row.get(0)?,
            conversation_id: row.get(1)?,
            role: row.get(2)?,
            content: row.get(3)?,
            thinking_content: row.get(4)?,
            attachments: row.get(5)?,
            feedback: row.get(6)?,
            created_at: row.get(7)?,
        })
    })?;
    
    let mut messages = Vec::new();
    for msg in rows {
        messages.push(msg?);
    }
    
    Ok(messages)
}

pub fn create_message(
    conn: &Connection,
    conversation_id: &str,
    role: &str,
    content: &str,
    attachments: Option<String>,
) -> AppResult<String> {
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp_millis();
    
    conn.execute(
        "INSERT INTO messages (id, conversation_id, role, content, attachments, created_at) 
         VALUES (?, ?, ?, ?, ?, ?)",
        params![id, conversation_id, role, content, attachments, now],
    )?;
    
    // 更新会话的更新时间
    conn.execute(
        "UPDATE conversations SET updated_at = ? WHERE id = ?",
        params![now, conversation_id],
    )?;
    
    Ok(id)
}

pub fn append_message_content(
    conn: &Connection,
    message_id: &str,
    delta: &str,
    is_thinking: bool,
) -> AppResult<()> {
    if is_thinking {
        // 追加到 thinking_content
        conn.execute(
            "UPDATE messages SET thinking_content = COALESCE(thinking_content, '') || ? WHERE id = ?",
            params![delta, message_id],
        )?;
    } else {
        // 追加到 content
        conn.execute(
            "UPDATE messages SET content = content || ? WHERE id = ?",
            params![delta, message_id],
        )?;
    }
    
    Ok(())
}

pub fn complete_message(conn: &Connection, message_id: &str) -> AppResult<()> {
    // 消息完成时不需要额外操作，内容已经通过 append 累积
    Ok(())
}

pub fn set_message_feedback(
    conn: &Connection,
    message_id: &str,
    feedback: &str,
) -> AppResult<()> {
    conn.execute(
        "UPDATE messages SET feedback = ? WHERE id = ?",
        params![feedback, message_id],
    )?;
    
    Ok(())
}
