use rusqlite::params;
use serde::{Deserialize, Serialize};

use crate::db::Database;
use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageAttachment {
    pub id: String,
    #[serde(rename = "type")]
    pub attachment_type: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub id: String,
    pub conversation_id: String,
    pub role: String,
    pub content: String,
    pub thinking_content: String,
    pub status: String,
    pub attachments: Vec<MessageAttachment>,
    pub created_at: String,
    pub updated_at: String,
    /// 用户反馈: "" | "like" | "dislike"
    pub feedback: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateMessageRequest {
    pub conversation_id: String,
    pub role: String,
    pub content: String,
    #[serde(default)]
    pub attachments: Vec<MessageAttachment>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMessageRequest {
    pub content: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct StreamChunk {
    pub conversation_id: String,
    pub message_id: String,
    pub delta: String,
    pub is_thinking: bool,
    pub done: bool,
}

impl Database {
    pub fn list_messages(&self, conversation_id: &str) -> AppResult<Vec<Message>> {
        let mut stmt = self.conn().prepare(
            "SELECT id, conversation_id, role, content, thinking_content, status, attachments, created_at, updated_at, feedback
             FROM messages WHERE conversation_id = ?1 ORDER BY created_at ASC",
        )?;

        let rows = stmt.query_map(params![conversation_id], |row| {
            let attachments_json: String = row.get(6)?;
            let attachments: Vec<MessageAttachment> =
                serde_json::from_str(&attachments_json).unwrap_or_default();
            Ok(Message {
                id: row.get(0)?,
                conversation_id: row.get(1)?,
                role: row.get(2)?,
                content: row.get(3)?,
                thinking_content: row.get(4)?,
                status: row.get(5)?,
                attachments,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
                feedback: row.get(9)?,
            })
        })?;

        let mut messages = Vec::new();
        for row in rows {
            messages.push(row?);
        }
        Ok(messages)
    }

    pub fn get_message(&self, id: &str) -> AppResult<Message> {
        let mut stmt = self.conn().prepare(
            "SELECT id, conversation_id, role, content, thinking_content, status, attachments, created_at, updated_at, feedback
             FROM messages WHERE id = ?1",
        )?;

        stmt.query_row(params![id], |row| {
            let attachments_json: String = row.get(6)?;
            let attachments: Vec<MessageAttachment> =
                serde_json::from_str(&attachments_json).unwrap_or_default();
            Ok(Message {
                id: row.get(0)?,
                conversation_id: row.get(1)?,
                role: row.get(2)?,
                content: row.get(3)?,
                thinking_content: row.get(4)?,
                status: row.get(5)?,
                attachments,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
                feedback: row.get(9)?,
            })
        })
        .map_err(|_| AppError::NotFound(format!("Message {} not found", id)))
    }

    pub fn create_message(&self, req: &CreateMessageRequest) -> AppResult<Message> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let attachments_json = serde_json::to_string(&req.attachments)?;

        self.conn().execute(
            "INSERT INTO messages (id, conversation_id, role, content, thinking_content, status, attachments, created_at, updated_at, feedback)
             VALUES (?1, ?2, ?3, ?4, '', 'pending', ?5, ?6, ?7, '')",
            params![id, req.conversation_id, req.role, req.content, attachments_json, now, now],
        )?;

        self.increment_message_count(&req.conversation_id)?;
        self.get_message(&id)
    }

    pub fn update_message(&self, id: &str, req: &UpdateMessageRequest) -> AppResult<Message> {
        let existing = self.get_message(id)?;
        let now = chrono::Utc::now().to_rfc3339();

        let content = req.content.as_deref().unwrap_or(&existing.content);
        let status = req.status.as_deref().unwrap_or(&existing.status);

        self.conn().execute(
            "UPDATE messages SET content = ?1, status = ?2, updated_at = ?3 WHERE id = ?4",
            params![content, status, now, id],
        )?;

        self.get_message(id)
    }

    pub fn append_message_content(&self, id: &str, delta: &str, is_thinking: bool) -> AppResult<()> {
        let now = chrono::Utc::now().to_rfc3339();
        if is_thinking {
            self.conn().execute(
                "UPDATE messages SET thinking_content = thinking_content || ?1, status = 'streaming', updated_at = ?2 WHERE id = ?3",
                params![delta, now, id],
            )?;
        } else {
            self.conn().execute(
                "UPDATE messages SET content = content || ?1, status = 'streaming', updated_at = ?2 WHERE id = ?3",
                params![delta, now, id],
            )?;
        }
        Ok(())
    }

    pub fn complete_message(&self, id: &str) -> AppResult<()> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn().execute(
            "UPDATE messages SET status = 'complete', updated_at = ?1 WHERE id = ?2",
            params![now, id],
        )?;
        Ok(())
    }

    pub fn delete_message(&self, id: &str) -> AppResult<()> {
        let affected = self.conn().execute("DELETE FROM messages WHERE id = ?1", params![id])?;
        if affected == 0 {
            return Err(AppError::NotFound(format!("Message {} not found", id)));
        }
        Ok(())
    }

    /// 设置消息反馈（点赞/点踩/取消）
    pub fn set_message_feedback(&self, id: &str, feedback: &str) -> AppResult<Message> {
        let now = chrono::Utc::now().to_rfc3339();
        let affected = self.conn().execute(
            "UPDATE messages SET feedback = ?1, updated_at = ?2 WHERE id = ?3",
            params![feedback, now, id],
        )?;
        if affected == 0 {
            return Err(AppError::NotFound(format!("Message {} not found", id)));
        }
        self.get_message(id)
    }
}
