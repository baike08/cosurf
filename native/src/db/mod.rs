//! SQLite 数据库模块 (N-API)
//!
//! 从 src-tauri/src/db/ 迁移，移除 Tauri 依赖，通过 napi-rs 导出。
//! 包含: conversations, messages, bookmarks, bookmark_folders, history, settings, model_configs, mcp_servers, user_events

// 模块化数据库实体（渐进式重构）
pub mod user_events;

use napi::bindgen_prelude::*;
use rusqlite::{params, Connection};
use std::path::Path;
use std::sync::Mutex;

use crate::error::{AppError, AppResult};

// ===== 全局数据库实例 =====
lazy_static::lazy_static! {
    static ref DATABASE: Mutex<Option<Database>> = Mutex::new(None);
}

fn get_db() -> Result<std::sync::MutexGuard<'static, Option<Database>>> {
    let guard = DATABASE.lock().map_err(|e| Error::from_reason(format!("DB lock error: {}", e)))?;
    if guard.is_none() {
        return Err(Error::from_reason("Database not initialized"));
    }
    Ok(guard)
}

fn with_db<F, T>(f: F) -> Result<T>
where
    F: FnOnce(&Database) -> AppResult<T>,
{
    let guard = get_db()?;
    let db = guard.as_ref().unwrap();
    f(db).map_err(|e| Error::from_reason(e.to_string()))
}

// ===== 数据库结构体 =====
pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(app_data_dir: &Path) -> AppResult<Self> {
        std::fs::create_dir_all(app_data_dir)
            .map_err(|e| AppError::Internal(format!("Failed to create data directory: {}", e)))?;

        let db_path = app_data_dir.join("cosurf.db");
        let conn = Connection::open(&db_path)?;

        conn.execute_batch("PRAGMA journal_mode=WAL;")?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;

        let db = Self { conn };
        db.run_migrations()?;
        Ok(db)
    }

    fn conn(&self) -> &Connection {
        &self.conn
    }

    fn run_migrations(&self) -> AppResult<()> {
        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS conversations (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL DEFAULT 'New Conversation',
                is_pinned INTEGER NOT NULL DEFAULT 0,
                model_id TEXT NOT NULL DEFAULT '',
                message_count INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS messages (
                id TEXT PRIMARY KEY,
                conversation_id TEXT NOT NULL,
                role TEXT NOT NULL CHECK(role IN ('user', 'assistant', 'system')),
                content TEXT NOT NULL DEFAULT '',
                thinking_content TEXT NOT NULL DEFAULT '',
                status TEXT NOT NULL DEFAULT 'pending' CHECK(status IN ('pending', 'streaming', 'complete', 'error')),
                attachments TEXT NOT NULL DEFAULT '[]',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_messages_conversation_id ON messages(conversation_id);

            CREATE TABLE IF NOT EXISTS bookmarks (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                url TEXT NOT NULL,
                favicon TEXT,
                folder_id TEXT,
                sort_order INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS bookmark_folders (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                parent_id TEXT,
                sort_order INTEGER NOT NULL DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS history (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL DEFAULT '',
                url TEXT NOT NULL,
                visited_at TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_history_visited_at ON history(visited_at DESC);

            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS model_configs (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                provider TEXT NOT NULL,
                model_id TEXT NOT NULL,
                api_key TEXT,
                base_url TEXT,
                temperature REAL NOT NULL DEFAULT 0.7,
                top_p REAL NOT NULL DEFAULT 1.0,
                max_tokens INTEGER NOT NULL DEFAULT 4096,
                is_local INTEGER NOT NULL DEFAULT 0,
                is_active INTEGER NOT NULL DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS mcp_servers (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                server_type TEXT NOT NULL DEFAULT 'stdio',
                url TEXT,
                command TEXT,
                args TEXT,
                cwd TEXT,
                env TEXT,
                disabled INTEGER NOT NULL DEFAULT 0,
                timeout INTEGER,
                enabled INTEGER NOT NULL DEFAULT 1,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                headers TEXT
            );

            CREATE INDEX IF NOT EXISTS idx_mcp_servers_enabled ON mcp_servers(enabled);

            -- Agent Prompts 表：存储可配置的 System Prompt
            CREATE TABLE IF NOT EXISTS agent_prompts (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,  -- prompt 名称（唯一标识）
                content TEXT NOT NULL,      -- prompt 内容
                description TEXT,           -- 描述说明
                is_enabled INTEGER NOT NULL DEFAULT 1,  -- 是否启用
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            ",
        )?;

        // 确保 thinking_content 列存在
        self.ensure_column("messages", "thinking_content", "TEXT NOT NULL DEFAULT ''")?;
        self.ensure_column("messages", "feedback", "TEXT NOT NULL DEFAULT ''")?;

        // 初始化默认 Agent Prompts
        self.init_default_agent_prompts()?;

        Ok(())
    }

    fn ensure_column(&self, table: &str, col_name: &str, col_def: &str) -> AppResult<()> {
        let mut stmt = self.conn.prepare(&format!("PRAGMA table_info({})", table))?;
        let exists = stmt
            .query_map([], |row| Ok(row.get::<_, String>(1)?))?
            .any(|name| name.as_deref() == Ok(col_name));

        if !exists {
            self.conn.execute(
                &format!("ALTER TABLE {} ADD COLUMN {} {}", table, col_name, col_def),
                [],
            )?;
        }
        Ok(())
    }

    /// 初始化默认 Agent Prompts
    fn init_default_agent_prompts(&self) -> AppResult<()> {
        let now = chrono::Utc::now().to_rfc3339();
        
        let defaults = vec![
            (
                "main-agent",
                "agent.md",
                include_str!("../../prompts/agent.md"),
                "主 Agent 的 System Prompt，定义角色和可用工具",
            ),
            (
                "memory-extract",
                "memory.md",
                include_str!("../../prompts/memory.md"),
                "用户记忆提取 Prompt，用于构建用户画像",
            ),
            (
                "page-reader",
                "reader.md",
                include_str!("../../prompts/reader.md"),
                "读模块：网页内容分析和摘要生成",
            ),
            (
                "note-taker",
                "remember.md",
                include_str!("../../prompts/remember.md"),
                "记模块：智能笔记生成和知识沉淀",
            ),
            (
                "knowledge-recall",
                "recall.md",
                include_str!("../../prompts/recall.md"),
                "想模块：知识召回和关联检索",
            ),
            (
                "decision-helper",
                "decision.md",
                include_str!("../../prompts/decision.md"),
                "决模块：决策支持和对比分析",
            ),
        ];

        for (id, name, content, description) in defaults {
            self.conn.execute(
                "INSERT OR IGNORE INTO agent_prompts (id, name, content, description, is_enabled, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, 1, ?5, ?6)",
                params![id, name, content, description, now, now],
            )?;
        }

        // 创建用户行为事件表
        tracing::info!("🔧 Creating user_events table in migrations...");
        self.create_user_events_table()?;
        tracing::info!("✅ user_events table migration completed");

        Ok(())
    }
}

// 在 Database::run_migrations 中调用（需要在 run_migrations 末尾添加）
impl Database {
    pub fn create_user_events_table(&self) -> AppResult<()> {
        user_events::create_user_events_table(self.conn())
    }
}

// ===== 初始化 =====
pub fn init_database(app_data_dir: &str) -> Result<()> {
    let db = Database::new(Path::new(app_data_dir))
        .map_err(|e| Error::from_reason(format!("Failed to init database: {}", e)))?;
    let mut guard = DATABASE.lock().map_err(|e| Error::from_reason(e.to_string()))?;
    *guard = Some(db);
    tracing::info!("Database initialized at: {}", app_data_dir);
    Ok(())
}

// ============================================================
// Conversations - N-API 导出
// ============================================================

#[napi]
pub fn db_list_conversations() -> Result<String> {
    with_db(|db| {
        let mut stmt = db.conn().prepare(
            "SELECT id, title, is_pinned, model_id, message_count, created_at, updated_at
             FROM conversations ORDER BY is_pinned DESC, updated_at DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "title": row.get::<_, String>(1)?,
                "isPinned": row.get::<_, i32>(2)? != 0,
                "modelId": row.get::<_, String>(3)?,
                "messageCount": row.get::<_, i64>(4)?,
                "createdAt": row.get::<_, String>(5)?,
                "updatedAt": row.get::<_, String>(6)?,
            }))
        })?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }
        Ok(serde_json::to_string(&items)?)
    })
}

#[napi]
pub fn db_get_conversation(id: String) -> Result<String> {
    with_db(|db| {
        let mut stmt = db.conn().prepare(
            "SELECT id, title, is_pinned, model_id, message_count, created_at, updated_at
             FROM conversations WHERE id = ?1",
        )?;
        let result = stmt.query_row(params![id], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "title": row.get::<_, String>(1)?,
                "isPinned": row.get::<_, i32>(2)? != 0,
                "modelId": row.get::<_, String>(3)?,
                "messageCount": row.get::<_, i64>(4)?,
                "createdAt": row.get::<_, String>(5)?,
                "updatedAt": row.get::<_, String>(6)?,
            }))
        }).map_err(|_| AppError::NotFound(format!("Conversation {} not found", id)))?;
        Ok(serde_json::to_string(&result)?)
    })
}

#[napi]
pub fn db_create_conversation(title: Option<String>, model_id: Option<String>) -> Result<String> {
    with_db(|db| {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let t = title.as_deref().unwrap_or("New Conversation");
        let m = model_id.as_deref().unwrap_or("");
        db.conn().execute(
            "INSERT INTO conversations (id, title, is_pinned, model_id, message_count, created_at, updated_at)
             VALUES (?1, ?2, 0, ?3, 0, ?4, ?5)",
            params![id, t, m, now, now],
        )?;
        let result = serde_json::json!({
            "id": id, "title": t, "isPinned": false, "modelId": m,
            "messageCount": 0, "createdAt": now, "updatedAt": now,
        });
        Ok(serde_json::to_string(&result)?)
    })
}

#[napi]
pub fn db_update_conversation(id: String, title: Option<String>, is_pinned: Option<bool>, model_id: Option<String>) -> Result<String> {
    with_db(|db| {
        let now = chrono::Utc::now().to_rfc3339();
        // Get existing
        let existing: (String, bool, String) = db.conn().query_row(
            "SELECT title, is_pinned, model_id FROM conversations WHERE id = ?1",
            params![id],
            |row| Ok((row.get(0)?, row.get::<_, i32>(1)? != 0, row.get(2)?)),
        ).map_err(|_| AppError::NotFound(format!("Conversation {} not found", id)))?;

        let t = title.as_deref().unwrap_or(&existing.0);
        let p = is_pinned.unwrap_or(existing.1);
        let m = model_id.as_deref().unwrap_or(&existing.2);

        db.conn().execute(
            "UPDATE conversations SET title = ?1, is_pinned = ?2, model_id = ?3, updated_at = ?4 WHERE id = ?5",
            params![t, p as i32, m, now, id],
        )?;
        Ok(serde_json::to_string(&serde_json::json!({
            "id": id, "title": t, "isPinned": p, "modelId": m, "updatedAt": now,
        }))?)
    })
}

#[napi]
pub fn db_delete_conversation(id: String) -> Result<()> {
    with_db(|db| {
        db.conn().execute("DELETE FROM conversations WHERE id = ?1", params![id])?;
        Ok(())
    })
}

// ============================================================
// Messages - N-API 导出
// ============================================================

#[napi]
pub fn db_list_messages(conversation_id: String) -> Result<String> {
    with_db(|db| {
        let mut stmt = db.conn().prepare(
            "SELECT id, conversation_id, role, content, thinking_content, status, attachments, created_at, updated_at, COALESCE(feedback, '')
             FROM messages WHERE conversation_id = ?1 ORDER BY created_at ASC",
        )?;
        let rows = stmt.query_map(params![conversation_id], |row| {
            let attachments_json: String = row.get(6)?;
            let attachments: serde_json::Value = serde_json::from_str(&attachments_json).unwrap_or(serde_json::json!([]));
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "conversationId": row.get::<_, String>(1)?,
                "role": row.get::<_, String>(2)?,
                "content": row.get::<_, String>(3)?,
                "thinkingContent": row.get::<_, String>(4)?,
                "status": row.get::<_, String>(5)?,
                "attachments": attachments,
                "createdAt": row.get::<_, String>(7)?,
                "updatedAt": row.get::<_, String>(8)?,
                "feedback": row.get::<_, String>(9)?,
            }))
        })?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }
        Ok(serde_json::to_string(&items)?)
    })
}

#[napi]
pub fn db_create_message(conversation_id: String, role: String, content: String, attachments: Option<String>) -> Result<String> {
    with_db(|db| {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let att = attachments.as_deref().unwrap_or("[]");
        db.conn().execute(
            "INSERT INTO messages (id, conversation_id, role, content, thinking_content, status, attachments, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, '', 'pending', ?5, ?6, ?7)",
            params![id, conversation_id, role, content, att, now, now],
        )?;
        // Increment message count
        db.conn().execute(
            "UPDATE conversations SET message_count = message_count + 1, updated_at = ?1 WHERE id = ?2",
            params![now, conversation_id],
        )?;
        Ok(serde_json::to_string(&serde_json::json!({
            "id": id, "conversationId": conversation_id, "role": role,
            "content": content, "thinkingContent": "", "status": "pending",
            "createdAt": now, "updatedAt": now, "feedback": "",
        }))?)
    })
}

#[napi]
pub fn db_append_message_content(message_id: String, delta: String, is_thinking: bool) -> Result<()> {
    with_db(|db| {
        let now = chrono::Utc::now().to_rfc3339();
        if is_thinking {
            db.conn().execute(
                "UPDATE messages SET thinking_content = thinking_content || ?1, updated_at = ?2 WHERE id = ?3",
                params![delta, now, message_id],
            )?;
        } else {
            db.conn().execute(
                "UPDATE messages SET content = content || ?1, status = 'streaming', updated_at = ?2 WHERE id = ?3",
                params![delta, now, message_id],
            )?;
        }
        Ok(())
    })
}

#[napi]
pub fn db_complete_message(message_id: String) -> Result<()> {
    with_db(|db| {
        let now = chrono::Utc::now().to_rfc3339();
        db.conn().execute(
            "UPDATE messages SET status = 'complete', updated_at = ?1 WHERE id = ?2",
            params![now, message_id],
        )?;
        Ok(())
    })
}

#[napi]
pub fn db_set_message_feedback(message_id: String, feedback: String) -> Result<()> {
    with_db(|db| {
        let now = chrono::Utc::now().to_rfc3339();
        db.conn().execute(
            "UPDATE messages SET feedback = ?1, updated_at = ?2 WHERE id = ?3",
            params![feedback, now, message_id],
        )?;
        Ok(())
    })
}

// ============================================================
// Settings - N-API 导出
// ============================================================

#[napi]
pub fn db_get_setting(key: String) -> Result<Option<String>> {
    with_db(|db| {
        let mut stmt = db.conn().prepare("SELECT value FROM settings WHERE key = ?1")?;
        let result = stmt.query_row(params![key], |row| row.get::<_, String>(0));
        match result {
            Ok(val) => Ok(Some(val)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(AppError::Database(e)),
        }
    })
}

#[napi]
pub fn db_set_setting(key: String, value: String) -> Result<()> {
    with_db(|db| {
        db.conn().execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            params![key, value],
        )?;
        Ok(())
    })
}

#[napi]
pub fn db_get_settings() -> Result<String> {
    with_db(|db| {
        let mut stmt = db.conn().prepare("SELECT key, value FROM settings")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        let mut map = serde_json::Map::new();
        for row in rows {
            let (k, v) = row?;
            map.insert(k, serde_json::Value::String(v));
        }
        Ok(serde_json::to_string(&serde_json::Value::Object(map))?)
    })
}

// ============================================================
// Bookmarks - N-API 导出
// ============================================================

#[napi]
pub fn db_list_bookmarks(folder_id: Option<String>) -> Result<String> {
    with_db(|db| {
        let (sql, fid) = match &folder_id {
            Some(_) => (
                "SELECT id, title, url, favicon, folder_id, sort_order, created_at
                 FROM bookmarks WHERE folder_id = ?1 ORDER BY sort_order ASC",
                folder_id.as_deref(),
            ),
            None => (
                "SELECT id, title, url, favicon, folder_id, sort_order, created_at
                 FROM bookmarks WHERE folder_id IS NULL ORDER BY sort_order ASC",
                None,
            ),
        };
        let mut stmt = db.conn().prepare(sql)?;
        let rows = match fid {
            Some(f) => stmt.query_map(params![f], map_bookmark)?,
            None => stmt.query_map([], map_bookmark)?,
        };
        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }
        Ok(serde_json::to_string(&items)?)
    })
}

fn map_bookmark(row: &rusqlite::Row) -> rusqlite::Result<serde_json::Value> {
    Ok(serde_json::json!({
        "id": row.get::<_, String>(0)?,
        "title": row.get::<_, String>(1)?,
        "url": row.get::<_, String>(2)?,
        "favicon": row.get::<_, Option<String>>(3)?,
        "folderId": row.get::<_, Option<String>>(4)?,
        "order": row.get::<_, i64>(5)?,
        "createdAt": row.get::<_, String>(6)?,
    }))
}

#[napi]
pub fn db_create_bookmark(title: String, url: String, favicon: Option<String>, folder_id: Option<String>) -> Result<String> {
    with_db(|db| {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let max_order: i64 = db.conn().query_row(
            "SELECT COALESCE(MAX(sort_order), -1) FROM bookmarks WHERE folder_id IS ?1",
            params![folder_id],
            |row| row.get(0),
        )?;
        db.conn().execute(
            "INSERT INTO bookmarks (id, title, url, favicon, folder_id, sort_order, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![id, title, url, favicon, folder_id, max_order + 1, now],
        )?;
        Ok(serde_json::to_string(&serde_json::json!({
            "id": id, "title": title, "url": url, "favicon": favicon,
            "folderId": folder_id, "order": max_order + 1, "createdAt": now,
        }))?)
    })
}

#[napi]
pub fn db_delete_bookmark(id: String) -> Result<()> {
    with_db(|db| {
        db.conn().execute("DELETE FROM bookmarks WHERE id = ?1", params![id])?;
        Ok(())
    })
}

#[napi]
pub fn db_list_bookmark_folders(parent_id: Option<String>) -> Result<String> {
    with_db(|db| {
        let (sql, pid) = match &parent_id {
            Some(_) => (
                "SELECT id, name, parent_id, sort_order FROM bookmark_folders WHERE parent_id = ?1 ORDER BY sort_order ASC",
                parent_id.as_deref(),
            ),
            None => (
                "SELECT id, name, parent_id, sort_order FROM bookmark_folders WHERE parent_id IS NULL ORDER BY sort_order ASC",
                None,
            ),
        };
        let mut stmt = db.conn().prepare(sql)?;
        let rows = match pid {
            Some(p) => stmt.query_map(params![p], map_folder)?,
            None => stmt.query_map([], map_folder)?,
        };
        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }
        Ok(serde_json::to_string(&items)?)
    })
}

fn map_folder(row: &rusqlite::Row) -> rusqlite::Result<serde_json::Value> {
    Ok(serde_json::json!({
        "id": row.get::<_, String>(0)?,
        "name": row.get::<_, String>(1)?,
        "parentId": row.get::<_, Option<String>>(2)?,
        "order": row.get::<_, i64>(3)?,
    }))
}

#[napi]
pub fn db_create_bookmark_folder(name: String, parent_id: Option<String>) -> Result<String> {
    with_db(|db| {
        let id = uuid::Uuid::new_v4().to_string();
        let max_order: i64 = db.conn().query_row(
            "SELECT COALESCE(MAX(sort_order), -1) FROM bookmark_folders WHERE parent_id IS ?1",
            params![parent_id],
            |row| row.get(0),
        )?;
        db.conn().execute(
            "INSERT INTO bookmark_folders (id, name, parent_id, sort_order) VALUES (?1, ?2, ?3, ?4)",
            params![id, name, parent_id, max_order + 1],
        )?;
        Ok(serde_json::to_string(&serde_json::json!({
            "id": id, "name": name, "parentId": parent_id, "order": max_order + 1,
        }))?)
    })
}

#[napi]
pub fn db_delete_bookmark_folder(id: String) -> Result<()> {
    with_db(|db| {
        db.conn().execute("DELETE FROM bookmarks WHERE folder_id = ?1", params![id])?;
        db.conn().execute("DELETE FROM bookmark_folders WHERE parent_id = ?1", params![id])?;
        db.conn().execute("DELETE FROM bookmark_folders WHERE id = ?1", params![id])?;
        Ok(())
    })
}

// ============================================================
// History - N-API 导出
// ============================================================

#[napi]
pub fn db_list_history(limit: Option<i64>, offset: Option<i64>) -> Result<String> {
    with_db(|db| {
        let l = limit.unwrap_or(100);
        let o = offset.unwrap_or(0);
        let mut stmt = db.conn().prepare(
            "SELECT id, title, url, visited_at FROM history ORDER BY visited_at DESC LIMIT ?1 OFFSET ?2",
        )?;
        let rows = stmt.query_map(params![l, o], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "title": row.get::<_, String>(1)?,
                "url": row.get::<_, String>(2)?,
                "visitedAt": row.get::<_, String>(3)?,
            }))
        })?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }
        Ok(serde_json::to_string(&items)?)
    })
}

#[napi]
pub fn db_search_history(query: String, limit: Option<i64>) -> Result<String> {
    with_db(|db| {
        let pattern = format!("%{}%", query);
        let l = limit.unwrap_or(50);
        let mut stmt = db.conn().prepare(
            "SELECT id, title, url, visited_at FROM history
             WHERE title LIKE ?1 OR url LIKE ?1 ORDER BY visited_at DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![pattern, l], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "title": row.get::<_, String>(1)?,
                "url": row.get::<_, String>(2)?,
                "visitedAt": row.get::<_, String>(3)?,
            }))
        })?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }
        Ok(serde_json::to_string(&items)?)
    })
}

#[napi]
pub fn db_add_history(title: String, url: String) -> Result<String> {
    with_db(|db| {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        db.conn().execute(
            "INSERT INTO history (id, title, url, visited_at) VALUES (?1, ?2, ?3, ?4)",
            params![id, title, url, now],
        )?;
        Ok(serde_json::to_string(&serde_json::json!({
            "id": id, "title": title, "url": url, "visitedAt": now,
        }))?)
    })
}

#[napi]
pub fn db_clear_history() -> Result<()> {
    with_db(|db| {
        db.conn().execute("DELETE FROM history", [])?;
        Ok(())
    })
}

#[napi]
pub fn db_delete_history_entry(id: String) -> Result<()> {
    with_db(|db| {
        db.conn().execute("DELETE FROM history WHERE id = ?1", params![id])?;
        Ok(())
    })
}

// ============================================================
// Model Configs - N-API 导出
// ============================================================

#[napi]
pub fn db_list_model_configs() -> Result<String> {
    with_db(|db| {
        let mut stmt = db.conn().prepare(
            "SELECT id, name, provider, model_id, api_key, base_url, temperature, top_p, max_tokens, is_local, is_active
             FROM model_configs ORDER BY is_active DESC, name ASC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "name": row.get::<_, String>(1)?,
                "provider": row.get::<_, String>(2)?,
                "modelId": row.get::<_, String>(3)?,
                "apiKey": row.get::<_, Option<String>>(4)?,
                "baseUrl": row.get::<_, Option<String>>(5)?,
                "temperature": row.get::<_, f64>(6)?,
                "topP": row.get::<_, f64>(7)?,
                "maxTokens": row.get::<_, i64>(8)?,
                "isLocal": row.get::<_, i32>(9)? != 0,
                "isActive": row.get::<_, i32>(10)? != 0,
            }))
        })?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }
        Ok(serde_json::to_string(&items)?)
    })
}

#[napi]
pub fn db_get_active_model() -> Result<Option<String>> {
    with_db(|db| {
        let mut stmt = db.conn().prepare(
            "SELECT id, name, provider, model_id, api_key, base_url, temperature, top_p, max_tokens, is_local, is_active
             FROM model_configs WHERE is_active = 1 LIMIT 1",
        )?;
        let result = stmt.query_row([], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "name": row.get::<_, String>(1)?,
                "provider": row.get::<_, String>(2)?,
                "modelId": row.get::<_, String>(3)?,
                "apiKey": row.get::<_, Option<String>>(4)?,
                "baseUrl": row.get::<_, Option<String>>(5)?,
                "temperature": row.get::<_, f64>(6)?,
                "topP": row.get::<_, f64>(7)?,
                "maxTokens": row.get::<_, i64>(8)?,
                "isLocal": row.get::<_, i32>(9)? != 0,
                "isActive": true,
            }))
        });
        match result {
            Ok(val) => Ok(Some(serde_json::to_string(&val)?)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(AppError::Database(e)),
        }
    })
}

// ============================================================
// Skills Directory / MCP Servers - 便捷导出
// ============================================================

#[napi]
pub fn db_get_skills_directory() -> Result<Option<String>> {
    db_get_setting("skills_directory".to_string())
}

#[napi]
pub fn db_set_skills_directory(dir: String) -> Result<()> {
    db_set_setting("skills_directory".to_string(), dir)
}

#[napi]
pub fn db_list_mcp_servers() -> Result<String> {
    with_db(|db| {
        let mut stmt = db.conn().prepare(
            "SELECT id, name, server_type, url, command, args, cwd, env, disabled, timeout, enabled, created_at, updated_at, headers
             FROM mcp_servers ORDER BY name ASC",
        )?;
        let rows = stmt.query_map([], |row| {
            let headers_str: Option<String> = row.get::<_, Option<String>>(13)?;
            let headers: Option<serde_json::Value> = if let Some(h) = headers_str {
                // 尝试解析 JSON 字符串为 object
                serde_json::from_str(&h).ok()
            } else {
                None
            };
            
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "name": row.get::<_, String>(1)?,
                "serverType": row.get::<_, String>(2)?,
                "url": row.get::<_, Option<String>>(3)?,
                "command": row.get::<_, Option<String>>(4)?,
                "args": row.get::<_, Option<String>>(5)?,
                "cwd": row.get::<_, Option<String>>(6)?,
                "env": row.get::<_, Option<String>>(7)?,
                "disabled": row.get::<_, i32>(8)? != 0,
                "timeout": row.get::<_, Option<i64>>(9)?,
                "enabled": row.get::<_, i32>(10)? != 0,
                "createdAt": row.get::<_, i64>(11)?,
                "updatedAt": row.get::<_, i64>(12)?,
                "headers": headers,
            }))
        })?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }
        Ok(serde_json::to_string(&items)?)
    })
}

// ============================================================
// Model Config - Full CRUD (补全)
// ============================================================

fn map_model_config_json(row: &rusqlite::Row) -> rusqlite::Result<serde_json::Value> {
    Ok(serde_json::json!({
        "id": row.get::<_, String>(0)?,
        "name": row.get::<_, String>(1)?,
        "provider": row.get::<_, String>(2)?,
        "modelId": row.get::<_, String>(3)?,
        "apiKey": row.get::<_, Option<String>>(4)?,
        "baseUrl": row.get::<_, Option<String>>(5)?,
        "temperature": row.get::<_, f64>(6)?,
        "topP": row.get::<_, f64>(7)?,
        "maxTokens": row.get::<_, i64>(8)?,
        "isLocal": row.get::<_, i32>(9)? != 0,
        "isActive": row.get::<_, i32>(10)? != 0,
    }))
}

#[napi]
pub fn db_get_model_config(id: String) -> Result<String> {
    with_db(|db| {
        let mut stmt = db.conn().prepare(
            "SELECT id, name, provider, model_id, api_key, base_url, temperature, top_p, max_tokens, is_local, is_active
             FROM model_configs WHERE id = ?1",
        )?;
        let result = stmt.query_row(params![id], map_model_config_json)
            .map_err(|_| AppError::NotFound(format!("Model config {} not found", id)))?;
        Ok(serde_json::to_string(&result)?)
    })
}

#[napi]
pub fn db_create_model_config(
    name: String,
    provider: String,
    model_id: String,
    api_key: Option<String>,
    base_url: Option<String>,
    temperature: Option<f64>,
    top_p: Option<f64>,
    max_tokens: Option<i64>,
) -> Result<String> {
    with_db(|db| {
        let id = uuid::Uuid::new_v4().to_string();
        db.conn().execute(
            "INSERT INTO model_configs (id, name, provider, model_id, api_key, base_url,
             temperature, top_p, max_tokens, is_local, is_active)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 0, 0)",
            params![
                id,
                name,
                provider,
                model_id,
                api_key,
                base_url,
                temperature.unwrap_or(0.7),
                top_p.unwrap_or(1.0),
                max_tokens.unwrap_or(4096),
            ],
        )?;
        let mut stmt = db.conn().prepare(
            "SELECT id, name, provider, model_id, api_key, base_url, temperature, top_p, max_tokens, is_local, is_active
             FROM model_configs WHERE id = ?1",
        )?;
        let result = stmt.query_row(params![id], map_model_config_json)
            .map_err(|_| AppError::Internal("Failed to read back created model config".into()))?;
        Ok(serde_json::to_string(&result)?)
    })
}

#[napi]
pub fn db_update_model_config(
    id: String,
    name: Option<String>,
    api_key: Option<Option<String>>,
    base_url: Option<Option<String>>,
    temperature: Option<f64>,
    top_p: Option<f64>,
    max_tokens: Option<i64>,
) -> Result<String> {
    with_db(|db| {
        // Fetch existing
        let existing: (String, Option<String>, Option<String>, f64, f64, i64) = db.conn().query_row(
            "SELECT name, api_key, base_url, temperature, top_p, max_tokens FROM model_configs WHERE id = ?1",
            params![id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?)),
        ).map_err(|_| AppError::NotFound(format!("Model config {} not found", id)))?;

        let new_name = name.as_deref().unwrap_or(&existing.0);
        let new_api_key = api_key.flatten().or(existing.1);
        let new_base_url = base_url.flatten().or(existing.2);
        let new_temp = temperature.unwrap_or(existing.3);
        let new_top_p = top_p.unwrap_or(existing.4);
        let new_max = max_tokens.unwrap_or(existing.5);

        db.conn().execute(
            "UPDATE model_configs SET name = ?1, api_key = ?2, base_url = ?3,
             temperature = ?4, top_p = ?5, max_tokens = ?6 WHERE id = ?7",
            params![new_name, new_api_key, new_base_url, new_temp, new_top_p, new_max, id],
        )?;

        let mut stmt = db.conn().prepare(
            "SELECT id, name, provider, model_id, api_key, base_url, temperature, top_p, max_tokens, is_local, is_active
             FROM model_configs WHERE id = ?1",
        )?;
        let result = stmt.query_row(params![id], map_model_config_json)
            .map_err(|_| AppError::Internal("Failed to read back updated model config".into()))?;
        Ok(serde_json::to_string(&result)?)
    })
}

#[napi]
pub fn db_set_active_model(id: String) -> Result<()> {
    with_db(|db| {
        db.conn().execute("UPDATE model_configs SET is_active = 0", [])?;
        let affected = db.conn().execute(
            "UPDATE model_configs SET is_active = 1 WHERE id = ?1",
            params![id],
        )?;
        if affected == 0 {
            return Err(AppError::NotFound(format!("Model config {} not found", id)));
        }
        Ok(())
    })
}

#[napi]
pub fn db_delete_model_config(id: String) -> Result<()> {
    with_db(|db| {
        let affected = db.conn().execute("DELETE FROM model_configs WHERE id = ?1", params![id])?;
        if affected == 0 {
            return Err(AppError::NotFound(format!("Model config {} not found", id)));
        }
        Ok(())
    })
}

// ============================================================
// MCP Server - Full CRUD (补全)
// ============================================================

fn map_mcp_server_json(row: &rusqlite::Row) -> rusqlite::Result<serde_json::Value> {
    let headers_str: Option<String> = row.get::<_, Option<String>>(13)?;
    let headers: Option<serde_json::Value> = if let Some(h) = headers_str {
        // 尝试解析 JSON 字符串为 object
        serde_json::from_str(&h).ok()
    } else {
        None
    };
    
    Ok(serde_json::json!({
        "id": row.get::<_, String>(0)?,
        "name": row.get::<_, String>(1)?,
        "serverType": row.get::<_, String>(2)?,
        "url": row.get::<_, Option<String>>(3)?,
        "command": row.get::<_, Option<String>>(4)?,
        "args": row.get::<_, Option<String>>(5)?,
        "cwd": row.get::<_, Option<String>>(6)?,
        "env": row.get::<_, Option<String>>(7)?,
        "disabled": row.get::<_, i32>(8)? != 0,
        "timeout": row.get::<_, Option<i64>>(9)?,
        "enabled": row.get::<_, i32>(10)? != 0,
        "createdAt": row.get::<_, i64>(11)?,
        "updatedAt": row.get::<_, i64>(12)?,
        "headers": headers,
    }))
}

#[napi]
pub fn db_get_mcp_server(id: String) -> Result<String> {
    with_db(|db| {
        let mut stmt = db.conn().prepare(
            "SELECT id, name, server_type, url, command, args, cwd, env, disabled, timeout, enabled, created_at, updated_at, headers
             FROM mcp_servers WHERE id = ?1",
        )?;
        let result = stmt.query_row(params![id], map_mcp_server_json)
            .map_err(|_| AppError::NotFound(format!("MCP server {} not found", id)))?;
        Ok(serde_json::to_string(&result)?)
    })
}

#[napi]
pub fn db_create_mcp_server(
    name: String,
    server_type: Option<String>,
    url: Option<String>,
    command: Option<String>,
    args: Option<String>,
    cwd: Option<String>,
    env: Option<String>,
    timeout: Option<i64>,
    disabled: Option<bool>,
    enabled: Option<bool>,
    headers: Option<String>,
) -> Result<String> {
    with_db(|db| {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp();
        let st = server_type.as_deref().unwrap_or("stdio");
        let dis = disabled.unwrap_or(false);
        let en = enabled.unwrap_or(!dis);

        db.conn().execute(
            "INSERT INTO mcp_servers (id, name, server_type, url, command, args, cwd, env, disabled, timeout, enabled, created_at, updated_at, headers)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            params![
                id,
                name,
                st,
                url,
                command,
                args,
                cwd,
                env,
                dis as i32,
                timeout,
                en as i32,
                now,
                now,
                headers,
            ],
        )?;

        let mut stmt = db.conn().prepare(
            "SELECT id, name, server_type, url, command, args, cwd, env, disabled, timeout, enabled, created_at, updated_at, headers
             FROM mcp_servers WHERE id = ?1",
        )?;
        let result = stmt.query_row(params![id], map_mcp_server_json)
            .map_err(|_| AppError::Internal("Failed to read back created MCP server".into()))?;
        Ok(serde_json::to_string(&result)?)
    })
}

#[napi]
pub fn db_update_mcp_server(
    id: String,
    name: Option<String>,
    server_type: Option<String>,
    url: Option<Option<String>>,
    command: Option<Option<String>>,
    args: Option<Option<String>>,
    cwd: Option<Option<String>>,
    env: Option<Option<String>>,
    timeout: Option<Option<i64>>,
    disabled: Option<bool>,
    enabled: Option<bool>,
    headers: Option<Option<String>>,
) -> Result<String> {
    with_db(|db| {
        // Fetch existing
        let existing: (String, String, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<i64>, bool, bool, Option<String>) = db.conn().query_row(
            "SELECT name, server_type, url, command, args, cwd, env, timeout, disabled, enabled, headers FROM mcp_servers WHERE id = ?1",
            params![id],
            |row| Ok((
                row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?,
                row.get(5)?, row.get(6)?, row.get(7)?,
                row.get::<_, i32>(8)? != 0,
                row.get::<_, i32>(9)? != 0,
                row.get(10)?,
            )),
        ).map_err(|_| AppError::NotFound(format!("MCP server {} not found", id)))?;

        let now = chrono::Utc::now().timestamp();
        let new_name = name.as_deref().unwrap_or(&existing.0);
        let new_st = server_type.as_deref().unwrap_or(&existing.1);
        let new_url = url.flatten().or(existing.2);
        let new_cmd = command.flatten().or(existing.3);
        let new_args = args.flatten().or(existing.4);
        let new_cwd = cwd.flatten().or(existing.5);
        let new_env = env.flatten().or(existing.6);
        let new_timeout = timeout.flatten().or(existing.7);
        let new_dis = disabled.unwrap_or(existing.8);
        let new_en = enabled.unwrap_or(existing.9);
        let new_headers = headers.flatten().or(existing.10);

        db.conn().execute(
            "UPDATE mcp_servers SET name = ?1, server_type = ?2, url = ?3,
             command = ?4, args = ?5, cwd = ?6, env = ?7, disabled = ?8, timeout = ?9, enabled = ?10, updated_at = ?11, headers = ?13
             WHERE id = ?12",
            params![new_name, new_st, new_url, new_cmd, new_args, new_cwd, new_env, new_dis as i32, new_timeout, new_en as i32, now, id, new_headers],
        )?;

        let mut stmt = db.conn().prepare(
            "SELECT id, name, server_type, url, command, args, cwd, env, disabled, timeout, enabled, created_at, updated_at, headers
             FROM mcp_servers WHERE id = ?1",
        )?;
        let result = stmt.query_row(params![id], map_mcp_server_json)
            .map_err(|_| AppError::Internal("Failed to read back updated MCP server".into()))?;
        Ok(serde_json::to_string(&result)?)
    })
}

#[napi]
pub fn db_delete_mcp_server(id: String) -> Result<()> {
    with_db(|db| {
        let affected = db.conn().execute("DELETE FROM mcp_servers WHERE id = ?1", params![id])?;
        if affected == 0 {
            return Err(AppError::NotFound(format!("MCP server {} not found", id)));
        }
        Ok(())
    })
}

// ============================================================
// Message - Additional functions (补全)
// ============================================================

fn map_message_json(row: &rusqlite::Row) -> rusqlite::Result<serde_json::Value> {
    let attachments_json: String = row.get(6)?;
    let attachments: serde_json::Value = serde_json::from_str(&attachments_json).unwrap_or(serde_json::json!([]));
    Ok(serde_json::json!({
        "id": row.get::<_, String>(0)?,
        "conversationId": row.get::<_, String>(1)?,
        "role": row.get::<_, String>(2)?,
        "content": row.get::<_, String>(3)?,
        "thinkingContent": row.get::<_, String>(4)?,
        "status": row.get::<_, String>(5)?,
        "attachments": attachments,
        "createdAt": row.get::<_, String>(7)?,
        "updatedAt": row.get::<_, String>(8)?,
        "feedback": row.get::<_, String>(9)?,
    }))
}

#[napi]
pub fn db_get_message(id: String) -> Result<String> {
    with_db(|db| {
        let mut stmt = db.conn().prepare(
            "SELECT id, conversation_id, role, content, thinking_content, status, attachments, created_at, updated_at, COALESCE(feedback, '')
             FROM messages WHERE id = ?1",
        )?;
        let result = stmt.query_row(params![id], map_message_json)
            .map_err(|_| AppError::NotFound(format!("Message {} not found", id)))?;
        Ok(serde_json::to_string(&result)?)
    })
}

#[napi]
pub fn db_update_message(id: String, content: Option<String>, status: Option<String>) -> Result<String> {
    with_db(|db| {
        let now = chrono::Utc::now().to_rfc3339();
        // Fetch existing
        let existing: (String, String) = db.conn().query_row(
            "SELECT content, status FROM messages WHERE id = ?1",
            params![id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        ).map_err(|_| AppError::NotFound(format!("Message {} not found", id)))?;

        let new_content = content.as_deref().unwrap_or(&existing.0);
        let new_status = status.as_deref().unwrap_or(&existing.1);

        db.conn().execute(
            "UPDATE messages SET content = ?1, status = ?2, updated_at = ?3 WHERE id = ?4",
            params![new_content, new_status, now, id],
        )?;

        let mut stmt = db.conn().prepare(
            "SELECT id, conversation_id, role, content, thinking_content, status, attachments, created_at, updated_at, COALESCE(feedback, '')
             FROM messages WHERE id = ?1",
        )?;
        let result = stmt.query_row(params![id], map_message_json)
            .map_err(|_| AppError::Internal("Failed to read back updated message".into()))?;
        Ok(serde_json::to_string(&result)?)
    })
}

#[napi]
pub fn db_delete_message(id: String) -> Result<()> {
    with_db(|db| {
        let affected = db.conn().execute("DELETE FROM messages WHERE id = ?1", params![id])?;
        if affected == 0 {
            return Err(AppError::NotFound(format!("Message {} not found", id)));
        }
        Ok(())
    })
}

#[napi]
pub fn db_get_conversation_with_messages(id: String) -> Result<String> {
    with_db(|db| {
        // Get conversation
        let mut conv_stmt = db.conn().prepare(
            "SELECT id, title, is_pinned, model_id, message_count, created_at, updated_at
             FROM conversations WHERE id = ?1",
        )?;
        let conv = conv_stmt.query_row(params![id], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "title": row.get::<_, String>(1)?,
                "isPinned": row.get::<_, i32>(2)? != 0,
                "modelId": row.get::<_, String>(3)?,
                "messageCount": row.get::<_, i64>(4)?,
                "createdAt": row.get::<_, String>(5)?,
                "updatedAt": row.get::<_, String>(6)?,
            }))
        }).map_err(|_| AppError::NotFound(format!("Conversation {} not found", id)))?;

        // Get messages
        let mut msg_stmt = db.conn().prepare(
            "SELECT id, conversation_id, role, content, thinking_content, status, attachments, created_at, updated_at, COALESCE(feedback, '')
             FROM messages WHERE conversation_id = ?1 ORDER BY created_at ASC",
        )?;
        let rows = msg_stmt.query_map(params![id], map_message_json)?;
        let mut messages = Vec::new();
        for row in rows {
            messages.push(row?);
        }

        let mut result = conv;
        if let Some(obj) = result.as_object_mut() {
            obj.insert("messages".to_string(), serde_json::Value::Array(messages));
        }
        Ok(serde_json::to_string(&result)?)
    })
}

// ============================================================
// Agent Prompts - N-API 导出
// ============================================================

#[napi]
pub fn db_list_agent_prompts() -> Result<String> {
    with_db(|db| {
        let mut stmt = db.conn().prepare(
            "SELECT id, name, content, description, is_enabled, created_at, updated_at 
             FROM agent_prompts ORDER BY name ASC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "name": row.get::<_, String>(1)?,
                "content": row.get::<_, String>(2)?,
                "description": row.get::<_, Option<String>>(3)?,
                "isEnabled": row.get::<_, i64>(4)? != 0,
                "createdAt": row.get::<_, String>(5)?,
                "updatedAt": row.get::<_, String>(6)?,
            }))
        })?;
        let prompts: Vec<_> = rows.filter_map(|r| r.ok()).collect();
        Ok(serde_json::to_string(&prompts)?)
    })
}

#[napi]
pub fn db_get_agent_prompt(name: String) -> Result<Option<String>> {
    with_db(|db| {
        let mut stmt = db.conn().prepare(
            "SELECT id, name, content, description, is_enabled, created_at, updated_at 
             FROM agent_prompts WHERE name = ?1 LIMIT 1",
        )?;
        let prompt = match stmt.query_row(params![name], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "name": row.get::<_, String>(1)?,
                "content": row.get::<_, String>(2)?,
                "description": row.get::<_, Option<String>>(3)?,
                "isEnabled": row.get::<_, i64>(4)? != 0,
                "createdAt": row.get::<_, String>(5)?,
                "updatedAt": row.get::<_, String>(6)?,
            }))
        }) {
            Ok(p) => Some(p.to_string()),
            Err(rusqlite::Error::QueryReturnedNoRows) => None,
            Err(e) => return Err(AppError::from(e).into()),
        };
        Ok(prompt)
    })
}

#[napi]
pub fn db_set_agent_prompt(name: String, content: String, description: Option<String>) -> Result<()> {
    with_db(|db| {
        let now = chrono::Utc::now().to_rfc3339();
        db.conn().execute(
            "INSERT INTO agent_prompts (id, name, content, description, is_enabled, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, 1, ?5, ?6)
             ON CONFLICT(name) DO UPDATE SET
             content = excluded.content,
             description = excluded.description,
             updated_at = excluded.updated_at",
            params![
                format!("prompt-{}", name.replace(".", "-")),
                name,
                content,
                description.unwrap_or_default(),
                now,
                now
            ],
        )?;
        Ok(())
    })
}

#[napi]
pub fn db_toggle_agent_prompt(name: String) -> Result<bool> {
    with_db(|db| {
        let current: i64 = db.conn().query_row(
            "SELECT is_enabled FROM agent_prompts WHERE name = ?1",
            params![name],
            |row| row.get(0),
        )?;
        let new_value = if current == 0 { 1 } else { 0 };
        db.conn().execute(
            "UPDATE agent_prompts SET is_enabled = ?1, updated_at = ?2 WHERE name = ?3",
            params![new_value, chrono::Utc::now().to_rfc3339(), name],
        )?;
        Ok(new_value != 0)
    })
}

// ============================================================
// MCP Server Testing
// ============================================================

/// 测试 MCP Server 连接并获取可用工具列表
#[napi]
pub async fn db_test_mcp_server(config_json: String) -> Result<String> {
    use crate::ai::mcp::{McpClient, McpConfig, McpTransport};
    use serde_json::Value;
    
    let config: Value = serde_json::from_str(&config_json).map_err(|e| {
        napi::Error::from_reason(format!("Invalid JSON config: {}", e))
    })?;
    
    let server_type = config.get("serverType")
        .and_then(|v| v.as_str())
        .unwrap_or("stdio");
    
    if server_type == "stdio" {
        // stdio 模式测试
        test_mcp_stdio(
            config.get("command").and_then(|v| v.as_str()).map(|s| s.to_string()),
            config.get("args").and_then(|v| v.as_array()).cloned(),
            config.get("env").and_then(|v| v.as_object()).cloned(),
        ).await
    } else {
        // HTTP/SSE 模式测试
        let url = config.get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| napi::Error::from_reason("URL is required for HTTP type".to_string()))?
            .to_string();
        
        let api_key = config.get("apiKey").and_then(|v| v.as_str()).map(|s| s.to_string());
        
        // 解析 headers：可能是 object 或 string
        let headers = if let Some(headers_value) = config.get("headers") {
            if let Some(obj) = headers_value.as_object() {
                // 已经是 object，直接使用
                tracing::info!("🔧 MCP headers parsed as object: {:?}", obj);
                Some(obj.clone())
            } else if let Some(str_val) = headers_value.as_str() {
                // 是字符串，尝试解析为 JSON object
                match serde_json::from_str::<serde_json::Value>(str_val) {
                    Ok(Value::Object(obj)) => {
                        tracing::info!("🔧 MCP headers parsed from string: {:?}", obj);
                        Some(obj)
                    }
                    Ok(_) => {
                        tracing::warn!("⚠️  MCP headers is not an object after parsing: {}", str_val);
                        None
                    }
                    Err(e) => {
                        tracing::warn!("⚠️  Failed to parse MCP headers string: {}, error: {}", str_val, e);
                        None
                    }
                }
            } else {
                tracing::warn!("⚠️  MCP headers is neither object nor string");
                None
            }
        } else {
            tracing::info!("ℹ️  No MCP headers provided");
            None
        };
        
        let transport = match server_type.to_lowercase().as_str() {
            "sse" => McpTransport::Sse,
            _ => McpTransport::StreamableHttp,
        };
        
        tracing::info!("🚀 Initializing MCP client: url={}, transport={:?}, has_headers={}", 
            url, transport, headers.is_some());
        
        let mcp_config = McpConfig {
            server_url: url,
            api_key,
        };
        
        let mut client = McpClient::new(mcp_config, transport, headers);
        
        client.initialize().await.map_err(|e| {
            tracing::error!("❌ MCP initialization failed: {}", e);
            napi::Error::from_reason(format!("Failed to initialize MCP connection: {}", e))
        })?;
        
        let tools = client.list_tools();
        tracing::info!("✅ MCP client initialized: tools={}", tools.len());
        
        // 返回与 stdio 模式一致的格式
        let result = serde_json::json!({
            "success": true,
            "tools": tools
        });
        
        Ok(serde_json::to_string(&result)?)
    }
}

/// 通过 stdio 测试 MCP Server
async fn test_mcp_stdio(
    command: Option<String>,
    args: Option<Vec<serde_json::Value>>,
    env: Option<serde_json::Map<String, serde_json::Value>>,
) -> Result<String> {
    use tokio::process::Command;
    use tokio::io::{AsyncWriteExt, AsyncBufReadExt, BufReader};
    use std::time::Duration;
    
    let cmd = command.ok_or_else(|| {
        napi::Error::from_reason("Command is required for stdio type".to_string())
    })?;
    
    let cmd_args: Vec<String> = args.unwrap_or_default()
        .into_iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect();
    
    // Windows 上 npx/npm/pnpm 是 .cmd 文件，需要通过 cmd /c 执行
    let (final_cmd, final_args) = if cfg!(target_os = "windows") {
        if cmd.ends_with(".cmd") || cmd == "npx" || cmd == "npm" || cmd == "pnpm" {
            let mut all_args = vec!["/c".to_string(), cmd.clone()];
            all_args.extend(cmd_args);
            ("cmd".to_string(), all_args)
        } else {
            (cmd, cmd_args)
        }
    } else {
        (cmd, cmd_args)
    };
    
    let mut child_cmd = Command::new(&final_cmd);
    child_cmd.args(&final_args);
    
    // 设置环境变量
    if let Some(ref env_vars) = env {
        for (key, value) in env_vars {
            if let Some(val_str) = value.as_str() {
                child_cmd.env(key, val_str);
            }
        }
    }
    
    child_cmd.stdin(std::process::Stdio::piped());
    child_cmd.stdout(std::process::Stdio::piped());
    child_cmd.stderr(std::process::Stdio::piped());
    
    let mut child = child_cmd.spawn().map_err(|e| {
        napi::Error::from_reason(format!("Failed to start process '{}': {}", final_cmd, e))
    })?;
    
    let stdin = child.stdin.take().ok_or_else(|| {
        napi::Error::from_reason("Failed to get stdin".to_string())
    })?;
    
    let stdout = child.stdout.take().ok_or_else(|| {
        napi::Error::from_reason("Failed to get stdout".to_string())
    })?;
    
    let stderr = child.stderr.take().ok_or_else(|| {
        napi::Error::from_reason("Failed to get stderr".to_string())
    })?;
    
    let mut reader = BufReader::new(stdout);
    let mut stderr_reader = BufReader::new(stderr);
    let mut stdin_writer = stdin;
    
    // 发送 initialize 请求
    let init_request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "cosurf-native",
                "version": "0.1.0"
            }
        }
    });
    
    let init_msg = format!("{}\n", serde_json::to_string(&init_request)?);
    stdin_writer.write_all(init_msg.as_bytes()).await.map_err(|e| {
        napi::Error::from_reason(format!("Failed to write to stdin: {}", e))
    })?;
    
    // 读取响应（带超时）
    let mut response_line = String::new();
    tokio::select! {
        result = reader.read_line(&mut response_line) => {
            result.map_err(|e| napi::Error::from_reason(format!("Failed to read response: {}", e)))?;
        }
        _ = tokio::time::sleep(Duration::from_secs(10)) => {
            return Err(napi::Error::from_reason("Timeout waiting for initialize response".to_string()));
        }
    }
    
    let init_response: serde_json::Value = serde_json::from_str(&response_line).map_err(|e| {
        napi::Error::from_reason(format!("Invalid initialize response: {}", e))
    })?;
    
    // 检查是否有错误
    if let Some(error) = init_response.get("error") {
        return Err(napi::Error::from_reason(format!("Initialize failed: {}", error)));
    }
    
    // 发送 initialized 通知
    let initialized_notification = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized"
    });
    
    let notif_msg = format!("{}\n", serde_json::to_string(&initialized_notification)?);
    stdin_writer.write_all(notif_msg.as_bytes()).await.map_err(|e| {
        napi::Error::from_reason(format!("Failed to send initialized notification: {}", e))
    })?;
    
    // 请求工具列表
    let tools_request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list"
    });
    
    let tools_msg = format!("{}\n", serde_json::to_string(&tools_request)?);
    stdin_writer.write_all(tools_msg.as_bytes()).await.map_err(|e| {
        napi::Error::from_reason(format!("Failed to request tools list: {}", e))
    })?;
    
    // 读取工具列表响应
    let mut tools_line = String::new();
    tokio::select! {
        result = reader.read_line(&mut tools_line) => {
            result.map_err(|e| napi::Error::from_reason(format!("Failed to read tools response: {}", e)))?;
        }
        _ = tokio::time::sleep(Duration::from_secs(10)) => {
            return Err(napi::Error::from_reason("Timeout waiting for tools list".to_string()));
        }
    }
    
    let tools_response: serde_json::Value = serde_json::from_str(&tools_line).map_err(|e| {
        napi::Error::from_reason(format!("Invalid tools response: {}", e))
    })?;
    
    // 终止子进程
    let _ = child.kill().await;
    
    // 返回结果
    let result = serde_json::json!({
        "success": true,
        "tools": tools_response.get("result").and_then(|r| r.get("tools")).cloned(),
        "initResponse": init_response.get("result").cloned()
    });
    
    Ok(serde_json::to_string(&result)?)
}

// ============================================================
// MCP Server Loading
// ============================================================

/// 加载所有启用的 MCP Servers（供前端调用）
#[napi]
pub fn db_load_mcp_servers(servers_json: String) -> Result<()> {
    tracing::info!("🚀 db_load_mcp_servers called with {} bytes", servers_json.len());
    
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            // TODO: 实现 MCP Server 加载逻辑
            tracing::info!("MCP servers loading requested");
        });
    });
    
    Ok(())
}

// ===== User Events (用户行为事件) =====

/// 插入用户行为事件
#[napi]
pub fn db_insert_user_event(event_json: String) -> Result<()> {
    let event: user_events::UserEvent = serde_json::from_str(&event_json)
        .map_err(|e| Error::from_reason(format!("Failed to parse event: {}", e)))?;
    
    with_db(|db| {
        user_events::insert_user_event(db.conn(), &event)
    })?;
    
    Ok(())
}

/// 批量插入用户行为事件
#[napi]
pub fn db_batch_insert_user_events(events_json: String) -> Result<u32> {
    let events: Vec<user_events::UserEvent> = serde_json::from_str(&events_json)
        .map_err(|e| Error::from_reason(format!("Failed to parse events: {}", e)))?;
    
    let count = with_db(|db| {
        user_events::batch_insert_user_events(db.conn(), &events)
    })?;
    
    Ok(count as u32)
}

/// 清理超过 3 天的旧事件
#[napi]
pub fn db_cleanup_old_user_events() -> Result<u32> {
    let count = with_db(|db| {
        user_events::cleanup_old_user_events(db.conn(), 3) // 保留 3 天
    })?;
    
    Ok(count as u32)
}

/// 获取最近的用户行为事件
#[napi]
pub fn db_get_user_events(hours: i64, limit: i64) -> Result<String> {
    let end_time = chrono::Utc::now().timestamp_millis();
    let start_time = end_time - (hours * 60 * 60 * 1000);
    
    let events = with_db(|db| {
        user_events::get_user_events(db.conn(), start_time, end_time, limit)
    })?;
    
    serde_json::to_string(&events)
        .map_err(|e| Error::from_reason(format!("Failed to serialize events: {}", e)))
}

/// 获取事件统计
#[napi]
pub fn db_get_event_stats(event_type: String, days: i64) -> Result<String> {
    let event_type: user_events::EventType = event_type.parse()
        .map_err(|e| Error::from_reason(e))?;
    
    let stats = with_db(|db| {
        user_events::get_event_stats(db.conn(), &event_type, days)
    })?;
    
    serde_json::to_string(&stats)
        .map_err(|e| Error::from_reason(format!("Failed to serialize stats: {}", e)))
}

/// 获取页面停留统计
#[napi]
pub fn db_get_page_stay_stats(url: String, days: i64) -> Result<String> {
    let stats = with_db(|db| {
        user_events::get_page_stay_stats(db.conn(), &url, days)
    })?;
    
    serde_json::to_string(&stats)
        .map_err(|e| Error::from_reason(format!("Failed to serialize stats: {}", e)))
}

/// 获取最活跃的标签页
#[napi]
pub fn db_get_most_active_tabs(limit: i64) -> Result<String> {
    let tabs = with_db(|db| {
        user_events::get_most_active_tabs(db.conn(), limit)
    })?;
    
    serde_json::to_string(&tabs)
        .map_err(|e| Error::from_reason(format!("Failed to serialize tabs: {}", e)))
}
