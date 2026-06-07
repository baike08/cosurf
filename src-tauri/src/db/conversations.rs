use rusqlite::params;
use serde::{Deserialize, Serialize};

use crate::db::Database;
use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Conversation {
    pub id: String,
    pub title: String,
    pub is_pinned: bool,
    pub model_id: String,
    pub message_count: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateConversationRequest {
    pub title: Option<String>,
    pub model_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateConversationRequest {
    pub title: Option<String>,
    pub is_pinned: Option<bool>,
    pub model_id: Option<String>,
}

impl Database {
    pub fn list_conversations(&self) -> AppResult<Vec<Conversation>> {
        let mut stmt = self.conn().prepare(
            "SELECT id, title, is_pinned, model_id, message_count, created_at, updated_at
             FROM conversations ORDER BY is_pinned DESC, updated_at DESC",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(Conversation {
                id: row.get(0)?,
                title: row.get(1)?,
                is_pinned: row.get::<_, i32>(2)? != 0,
                model_id: row.get(3)?,
                message_count: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })?;

        let mut conversations = Vec::new();
        for row in rows {
            conversations.push(row?);
        }
        Ok(conversations)
    }

    pub fn get_conversation(&self, id: &str) -> AppResult<Conversation> {
        let mut stmt = self.conn().prepare(
            "SELECT id, title, is_pinned, model_id, message_count, created_at, updated_at
             FROM conversations WHERE id = ?1",
        )?;

        stmt.query_row(params![id], |row| {
            Ok(Conversation {
                id: row.get(0)?,
                title: row.get(1)?,
                is_pinned: row.get::<_, i32>(2)? != 0,
                model_id: row.get(3)?,
                message_count: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })
        .map_err(|_| AppError::NotFound(format!("Conversation {} not found", id)))
    }

    pub fn create_conversation(&self, req: &CreateConversationRequest) -> AppResult<Conversation> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let title = req.title.as_deref().unwrap_or("New Conversation");
        let model_id = req.model_id.as_deref().unwrap_or("");

        self.conn().execute(
            "INSERT INTO conversations (id, title, is_pinned, model_id, message_count, created_at, updated_at)
             VALUES (?1, ?2, 0, ?3, 0, ?4, ?5)",
            params![id, title, model_id, now, now],
        )?;

        self.get_conversation(&id)
    }

    pub fn update_conversation(&self, id: &str, req: &UpdateConversationRequest) -> AppResult<Conversation> {
        let existing = self.get_conversation(id)?;
        let now = chrono::Utc::now().to_rfc3339();

        let title = req.title.as_deref().unwrap_or(&existing.title);
        let is_pinned = req.is_pinned.unwrap_or(existing.is_pinned);
        let model_id = req.model_id.as_deref().unwrap_or(&existing.model_id);

        self.conn().execute(
            "UPDATE conversations SET title = ?1, is_pinned = ?2, model_id = ?3, updated_at = ?4 WHERE id = ?5",
            params![title, is_pinned as i32, model_id, now, id],
        )?;

        self.get_conversation(id)
    }

    pub fn delete_conversation(&self, id: &str) -> AppResult<()> {
        let affected = self.conn().execute("DELETE FROM conversations WHERE id = ?1", params![id])?;
        if affected == 0 {
            return Err(AppError::NotFound(format!("Conversation {} not found", id)));
        }
        Ok(())
    }

    pub fn increment_message_count(&self, conversation_id: &str) -> AppResult<()> {
        self.conn().execute(
            "UPDATE conversations SET message_count = message_count + 1, updated_at = ?1 WHERE id = ?2",
            params![chrono::Utc::now().to_rfc3339(), conversation_id],
        )?;
        Ok(())
    }
}
