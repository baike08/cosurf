pub mod bookmarks;
pub mod conversations;
pub mod history;
pub mod messages;
pub mod settings;

use rusqlite::Connection;

use crate::error::{AppError, AppResult};

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(app_data_dir: &std::path::Path) -> AppResult<Self> {
        std::fs::create_dir_all(app_data_dir).map_err(|e| {
            AppError::Internal(format!("Failed to create data directory: {}", e))
        })?;

        let db_path = app_data_dir.join("cosurf.db");
        let conn = Connection::open(&db_path)?;

        conn.execute_batch("PRAGMA journal_mode=WAL;")?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;

        let db = Self { conn };
        db.run_migrations()?;
        Ok(db)
    }

    #[cfg(test)]
    pub fn in_memory() -> AppResult<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;
        let db = Self { conn };
        db.run_migrations()?;
        Ok(db)
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
            ",
        )?;
        
        // 确保 thinking_content 字段存在（如果不存在则添加）
        self.ensure_thinking_content_column()?;
        
        // 数据迁移：将旧格式的混合内容分离到 thinking_content 和 content
        self.migrate_thinking_content()?;
        
        // 确保 mcp_servers 表包含所有必需列
        self.ensure_mcp_server_columns()?;
        
        // 确保 messages 表包含 feedback 列
        self.ensure_message_column("feedback", "TEXT NOT NULL DEFAULT ''")?;
        
        Ok(())
    }

    fn ensure_thinking_content_column(&self) -> AppResult<()> {
        // 检查字段是否存在
        let mut stmt = self.conn.prepare(
            "PRAGMA table_info(messages)"
        )?;
        
        let column_exists = stmt.query_map([], |row| {
            Ok(row.get::<_, String>(1)?)
        })?
        .any(|name| name.as_deref() == Ok("thinking_content"));
        
        if !column_exists {
            tracing::info!("Adding thinking_content column to messages table");
            self.conn.execute(
                "ALTER TABLE messages ADD COLUMN thinking_content TEXT NOT NULL DEFAULT ''",
                []
            )?;
        }
        
        Ok(())
    }

    fn migrate_thinking_content(&self) -> AppResult<()> {
        // 检查是否需要迁移（查找包含 thinking marker 的旧消息）
        let mut stmt = self.conn.prepare(
            "SELECT id, content FROM messages WHERE role = 'assistant' AND thinking_content = '' AND (content LIKE ?1 OR content LIKE ?2)"
        )?;
        
        let rows = stmt.query_map(
            rusqlite::params!["💭 Thinking...", "%\n\n💭💬\n\n%"],
            |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            }
        )?;
        
        let mut migrated_count = 0;
        for result in rows {
            if let Ok((id, content)) = result {
                // 解析旧格式的内容
                let thinking_marker = "💭 Thinking...\n";
                let separator = "\n\n💭💬\n\n";
                
                if content.starts_with(thinking_marker) {
                    let after_marker = &content[thinking_marker.len()..];
                    if let Some(sep_idx) = after_marker.find(separator) {
                        let thinking = &after_marker[..sep_idx];
                        let response = &after_marker[sep_idx + separator.len()..];
                        
                        // 更新数据库，分离 thinking 和 response
                        let now = chrono::Utc::now().to_rfc3339();
                        self.conn.execute(
                            "UPDATE messages SET thinking_content = ?1, content = ?2, updated_at = ?3 WHERE id = ?4",
                            rusqlite::params![thinking, response, now, id]
                        )?;
                        migrated_count += 1;
                    }
                }
            }
        }
        
        if migrated_count > 0 {
            tracing::info!("Migrated {} messages to separate thinking_content field", migrated_count);
        }
        
        Ok(())
    }

    /// 确保指定表包含指定列（通用方法）
    fn ensure_message_column(&self, col_name: &str, col_def: &str) -> AppResult<()> {
        let mut stmt = self.conn.prepare("PRAGMA table_info(messages)")?;
        let column_exists = stmt.query_map([], |row| {
            Ok(row.get::<_, String>(1)?)
        })?
        .any(|name| name.as_deref() == Ok(col_name));
        
        if !column_exists {
            tracing::info!("Adding {} column to messages table", col_name);
            self.conn.execute(
                &format!("ALTER TABLE messages ADD COLUMN {} {}", col_name, col_def),
                []
            )?;
        }
        Ok(())
    }

    /// 确保 mcp_servers 表包含 server_type 列
    fn ensure_mcp_server_columns(&self) -> AppResult<()> {
        // 获取现有列名
        let mut stmt = self.conn.prepare("PRAGMA table_info(mcp_servers)")?;
        let existing_columns: Vec<String> = stmt.query_map([], |row| {
            Ok(row.get::<_, String>(1)?)
        })?
        .filter_map(|name| name.ok())
        .collect();
        
        // 需要确保存在的列及其定义
        let required_columns = [
            ("server_type", "TEXT NOT NULL DEFAULT 'stdio'"),
            ("url", "TEXT"),
            ("cwd", "TEXT"),
            ("timeout", "INTEGER"),
            ("enabled", "INTEGER NOT NULL DEFAULT 1"),
            ("headers", "TEXT"),
        ];
        
        for (col_name, col_def) in &required_columns {
            if !existing_columns.iter().any(|c| c == col_name) {
                tracing::info!("Adding {} column to mcp_servers table", col_name);
                self.conn.execute(
                    &format!("ALTER TABLE mcp_servers ADD COLUMN {} {}", col_name, col_def),
                    []
                )?;
            }
        }
        
        Ok(())
    }

    pub fn conn(&self) -> &Connection {
        &self.conn
    }
}
