//! Checkpoint 管理器 - Agent Loop 状态持久化
//!
//! 功能：
//! - 创建检查点（保存 Agent Loop 的中间状态）
//! - 回滚到指定检查点（失败时恢复状态）
//! - 清理过期检查点（自动维护）

use rusqlite::{Connection, params, OptionalExtension};
use serde::{Deserialize, Serialize};
use tracing::{info, warn, error};
use uuid::Uuid;

use crate::ai::provider::ChatMessage;
use crate::ai::tools::{ToolCall, ToolResult};
use crate::error::{AppError, AppResult};

/// 检查点数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub id: String,
    pub conversation_id: String,
    pub iteration: u32,
    pub timestamp: i64,
    
    /// 新增的消息（增量）
    pub new_messages: Vec<ChatMessage>,
    
    /// 文件变更记录
    pub file_changes: Vec<FileChange>,
    
    /// 工具执行结果记录
    pub tool_results: Vec<ToolResultRecord>,
}

/// 文件变更记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileChange {
    /// 创建了新文件
    Created { 
        path: String,
        backup_path: Option<String>, // 通常为空
    },
    /// 修改了现有文件
    Modified { 
        path: String,
        backup_path: String, // 备份文件路径
    },
    /// 删除了文件
    Deleted { 
        path: String,
        backup_path: String, // 备份文件路径
    },
}

impl FileChange {
    /// 获取原始文件路径
    pub fn get_path(&self) -> &str {
        match self {
            Self::Created { path, .. } => path,
            Self::Modified { path, .. } => path,
            Self::Deleted { path, .. } => path,
        }
    }
    
    /// 获取备份文件路径（如果有）
    pub fn get_backup_path(&self) -> Option<&str> {
        match self {
            Self::Created { backup_path, .. } => backup_path.as_deref(),
            Self::Modified { backup_path, .. } => Some(backup_path),
            Self::Deleted { backup_path, .. } => Some(backup_path),
        }
    }
}

/// 工具执行结果记录（简化版，用于持久化）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResultRecord {
    pub tool_call_id: String,
    pub tool_name: String,
    pub output: String,
    pub success: bool,
}

impl ToolResultRecord {
    /// 从 ToolCall 和 ToolResult 创建记录
    pub fn new(tool_call: &ToolCall, result: &ToolResult) -> Self {
        Self {
            tool_call_id: tool_call.id.clone(),
            tool_name: tool_call.name.clone(),
            output: result.output.clone(),
            success: result.success,
        }
    }
}

/// 检查点管理器
pub struct CheckpointManager {
    db: Connection,
}

impl CheckpointManager {
    /// 创建新的检查点管理器
    pub fn new(db_path: &str) -> AppResult<Self> {
        let db = Connection::open(db_path)
            .map_err(|e| AppError::Internal(format!("Failed to open checkpoint DB: {}", e)))?;
        
        // 初始化表结构
        Self::init_schema(&db)?;
        
        info!("✅ CheckpointManager initialized at: {}", db_path);
        
        Ok(Self { db })
    }
    
    /// 初始化数据库表结构
    fn init_schema(db: &Connection) -> AppResult<()> {
        db.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS checkpoints (
                id TEXT PRIMARY KEY,
                conversation_id TEXT NOT NULL,
                iteration INTEGER NOT NULL,
                data TEXT NOT NULL,
                created_at INTEGER NOT NULL
            );
            
            CREATE INDEX IF NOT EXISTS idx_checkpoints_conversation 
            ON checkpoints(conversation_id);
            
            CREATE INDEX IF NOT EXISTS idx_checkpoints_created_at 
            ON checkpoints(created_at);
            "
        ).map_err(|e| AppError::Internal(format!("Failed to create checkpoints table: {}", e)))?;
        
        Ok(())
    }
    
    /// 创建新的检查点
    pub fn create_checkpoint(
        &mut self,
        conversation_id: &str,
        iteration: u32,
        new_messages: Vec<ChatMessage>,
        file_changes: Vec<FileChange>,
        tool_results: Vec<ToolResultRecord>,
    ) -> AppResult<String> {
        let checkpoint_id = Uuid::new_v4().to_string();
        let timestamp = chrono::Utc::now().timestamp();
        
        let checkpoint = Checkpoint {
            id: checkpoint_id.clone(),
            conversation_id: conversation_id.to_string(),
            iteration,
            timestamp,
            new_messages,
            file_changes,
            tool_results,
        };
        
        // 序列化为 JSON
        let json_data = serde_json::to_string(&checkpoint)
            .map_err(|e| AppError::Internal(format!("Failed to serialize checkpoint: {}", e)))?;
        
        // 保存到 SQLite
        self.db.execute(
            "INSERT INTO checkpoints (id, conversation_id, iteration, data, created_at) 
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                checkpoint.id,
                checkpoint.conversation_id,
                checkpoint.iteration as i64,
                json_data,
                checkpoint.timestamp
            ],
        ).map_err(|e| AppError::Internal(format!("Failed to save checkpoint: {}", e)))?;
        
        info!("📸 Checkpoint created: {} (conversation={}, iteration={})", 
            checkpoint_id, conversation_id, iteration);
        
        Ok(checkpoint_id)
    }
    
    /// 获取最新的检查点
    pub fn get_latest_checkpoint(&self, conversation_id: &str) -> AppResult<Option<Checkpoint>> {
        let row_opt = self.db.query_row(
            "SELECT data FROM checkpoints 
             WHERE conversation_id = ?1 
             ORDER BY created_at DESC 
             LIMIT 1",
            params![conversation_id],
            |row| row.get::<_, String>(0),
        ).optional()
        .map_err(|e| AppError::Internal(format!("Failed to query checkpoint: {}", e)))?;
        
        match row_opt {
            Some(json_data) => {
                let checkpoint: Checkpoint = serde_json::from_str(&json_data)
                    .map_err(|e| AppError::Internal(format!("Failed to deserialize checkpoint: {}", e)))?;
                
                info!("📥 Loaded latest checkpoint: {} (iteration={})", 
                    checkpoint.id, checkpoint.iteration);
                
                Ok(Some(checkpoint))
            }
            None => {
                Ok(None)
            }
        }
    }
    
    /// 获取指定检查点
    pub fn get_checkpoint(&self, checkpoint_id: &str) -> AppResult<Option<Checkpoint>> {
        let row_opt = self.db.query_row(
            "SELECT data FROM checkpoints WHERE id = ?1",
            params![checkpoint_id],
            |row| row.get::<_, String>(0),
        ).optional()
        .map_err(|e| AppError::Internal(format!("Failed to query checkpoint: {}", e)))?;
        
        match row_opt {
            Some(json_data) => {
                let checkpoint: Checkpoint = serde_json::from_str(&json_data)
                    .map_err(|e| AppError::Internal(format!("Failed to deserialize checkpoint: {}", e)))?;
                
                Ok(Some(checkpoint))
            }
            None => Ok(None),
        }
    }
    
    /// 回滚到指定检查点（返回检查点数据）
    pub fn rollback_to_checkpoint(&self, checkpoint_id: &str) -> AppResult<Checkpoint> {
        let checkpoint = self.get_checkpoint(checkpoint_id)?
            .ok_or_else(|| AppError::Internal(format!("Checkpoint not found: {}", checkpoint_id)))?;
        
        warn!("🔄 Rolling back to checkpoint: {} (iteration={})", 
            checkpoint.id, checkpoint.iteration);
        
        Ok(checkpoint)
    }
    
    /// 删除指定检查点
    pub fn delete_checkpoint(&mut self, checkpoint_id: &str) -> AppResult<()> {
        self.db.execute(
            "DELETE FROM checkpoints WHERE id = ?1",
            params![checkpoint_id],
        ).map_err(|e| AppError::Internal(format!("Failed to delete checkpoint: {}", e)))?;
        
        info!("🗑️  Deleted checkpoint: {}", checkpoint_id);
        
        Ok(())
    }
    
    /// 清理过期检查点（保留最近 N 小时）
    pub fn cleanup_old_checkpoints(&mut self, retention_hours: i64) -> AppResult<usize> {
        let cutoff_timestamp = chrono::Utc::now().timestamp() - (retention_hours * 3600);
        
        let deleted_count = self.db.execute(
            "DELETE FROM checkpoints WHERE created_at < ?1",
            params![cutoff_timestamp],
        ).map_err(|e| AppError::Internal(format!("Failed to cleanup checkpoints: {}", e)))?;
        
        if deleted_count > 0 {
            info!("🧹 Cleaned up {} old checkpoints (older than {} hours)", 
                deleted_count, retention_hours);
        }
        
        Ok(deleted_count)
    }
    
    /// 获取会话的检查点数量
    pub fn get_checkpoint_count(&self, conversation_id: &str) -> AppResult<usize> {
        let count: i64 = self.db.query_row(
            "SELECT COUNT(*) FROM checkpoints WHERE conversation_id = ?1",
            params![conversation_id],
            |row| row.get(0),
        ).map_err(|e| AppError::Internal(format!("Failed to count checkpoints: {}", e)))?;
        
        Ok(count as usize)
    }
    
    /// 列出会话的所有检查点（按时间排序）
    pub fn list_checkpoints(&self, conversation_id: &str) -> AppResult<Vec<CheckpointSummary>> {
        let mut stmt = self.db.prepare(
            "SELECT id, iteration, created_at FROM checkpoints 
             WHERE conversation_id = ?1 
             ORDER BY created_at ASC"
        ).map_err(|e| AppError::Internal(format!("Failed to prepare statement: {}", e)))?;
        
        let rows = stmt.query_map(params![conversation_id], |row| {
            Ok(CheckpointSummary {
                id: row.get(0)?,
                iteration: row.get(1)?,
                created_at: row.get(2)?,
            })
        }).map_err(|e| AppError::Internal(format!("Failed to query checkpoints: {}", e)))?;
        
        let summaries: Result<Vec<_>, _> = rows.collect();
        Ok(summaries?)
    }
}

// ============================================================
// 文件备份与回滚工具函数
// ============================================================

/// 备份文件（如果需要）
/// 
/// 如果文件存在，创建备份；如果不存在，返回 None
pub fn backup_file_if_needed(file_path: &str) -> AppResult<Option<FileChange>> {
    let path = std::path::Path::new(file_path);
    
    if !path.exists() {
        // 文件不存在，不需要备份
        return Ok(None);
    }
    
    // 创建备份文件
    let backup_dir = std::env::temp_dir().join("cosurf-checkpoint-backups");
    std::fs::create_dir_all(&backup_dir)
        .map_err(|e| AppError::Internal(format!("Failed to create backup dir: {}", e)))?;
    
    let backup_filename = format!("{}_{}.bak", 
        path.file_name().unwrap_or_default().to_string_lossy(),
        chrono::Utc::now().timestamp_millis()
    );
    let backup_path = backup_dir.join(backup_filename);
    
    // 复制文件到备份目录
    std::fs::copy(path, &backup_path)
        .map_err(|e| AppError::Internal(format!("Failed to backup file: {}", e)))?;
    
    info!("📦 Backed up file: {} -> {}", file_path, backup_path.display());
    
    Ok(Some(FileChange::Modified {
        path: file_path.to_string(),
        backup_path: backup_path.to_string_lossy().to_string(),
    }))
}

/// 从备份恢复文件
pub fn restore_from_backup(file_change: &FileChange) -> AppResult<()> {
    match file_change {
        FileChange::Created { path, .. } => {
            // 删除创建的文件
            if std::path::Path::new(path).exists() {
                std::fs::remove_file(path)
                    .map_err(|e| AppError::Internal(format!("Failed to delete created file: {}", e)))?;
                info!("🗑️  Deleted created file: {}", path);
            }
        }
        FileChange::Modified { path, backup_path } => {
            // 从备份恢复
            if std::path::Path::new(backup_path).exists() {
                std::fs::copy(backup_path, path)
                    .map_err(|e| AppError::Internal(format!("Failed to restore file: {}", e)))?;
                
                // 清理备份文件
                let _ = std::fs::remove_file(backup_path);
                
                info!("🔄 Restored file from backup: {}", path);
            } else {
                warn!("⚠️  Backup file not found: {}", backup_path);
            }
        }
        FileChange::Deleted { path, backup_path } => {
            // 从备份恢复删除的文件
            if std::path::Path::new(backup_path).exists() {
                std::fs::copy(backup_path, path)
                    .map_err(|e| AppError::Internal(format!("Failed to restore deleted file: {}", e)))?;
                
                // 清理备份文件
                let _ = std::fs::remove_file(backup_path);
                
                info!("🔄 Restored deleted file: {}", path);
            } else {
                warn!("⚠️  Backup file not found: {}", backup_path);
            }
        }
    }
    
    Ok(())
}

/// 批量回滚文件变更
pub fn rollback_file_changes(file_changes: &[FileChange]) -> AppResult<()> {
    for change in file_changes {
        if let Err(e) = restore_from_backup(change) {
            error!("❌ Failed to rollback file change: {:?}, error: {}", change, e);
            // 继续回滚其他文件，不中断
        }
    }
    
    Ok(())
}

/// 清理过期备份文件（保留最近 N 小时）
pub fn cleanup_old_backups(retention_hours: i64) -> AppResult<usize> {
    let backup_dir = std::env::temp_dir().join("cosurf-checkpoint-backups");
    
    if !backup_dir.exists() {
        return Ok(0);
    }
    
    let cutoff_time = chrono::Utc::now().timestamp_millis() - (retention_hours * 3600 * 1000);
    let mut deleted_count = 0;
    
    for entry in std::fs::read_dir(&backup_dir)
        .map_err(|e| AppError::Internal(format!("Failed to read backup dir: {}", e)))?
    {
        let entry = entry
            .map_err(|e| AppError::Internal(format!("Failed to read dir entry: {}", e)))?;
        
        let path = entry.path();
        
        // 只处理 .bak 文件
        if path.extension().and_then(|e| e.to_str()) == Some("bak") {
            // 检查文件修改时间
            if let Ok(metadata) = std::fs::metadata(&path) {
                if let Ok(modified) = metadata.modified() {
                    if let Ok(duration) = modified.duration_since(std::time::UNIX_EPOCH) {
                        let file_timestamp = duration.as_millis() as i64;
                        
                        if file_timestamp < cutoff_time {
                            if std::fs::remove_file(&path).is_ok() {
                                deleted_count += 1;
                            }
                        }
                    }
                }
            }
        }
    }
    
    if deleted_count > 0 {
        info!("🧹 Cleaned up {} old backup files", deleted_count);
    }
    
    Ok(deleted_count)
}

/// 检查点摘要（轻量级，用于列表展示）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointSummary {
    pub id: String,
    pub iteration: u32,
    pub created_at: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    
    fn setup_test_db() -> (CheckpointManager, String) {
        // 使用 Windows 兼容的临时目录
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join(format!("test_checkpoint_{}.db", Uuid::new_v4()));
        let db_path_str = db_path.to_string_lossy().to_string();
        let mgr = CheckpointManager::new(&db_path_str).unwrap();
        (mgr, db_path_str)
    }
    
    fn cleanup_test_db(db_path: &str) {
        let _ = fs::remove_file(db_path);
    }
    
    #[test]
    fn test_create_and_get_checkpoint() {
        let (mut mgr, db_path) = setup_test_db();
        
        // 创建检查点
        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: "Test message".to_string(),
            name: None,
            tool_call_id: None,
        }];
        
        let tool_results = vec![ToolResultRecord {
            tool_call_id: "test-id".to_string(),
            tool_name: "test_tool".to_string(),
            output: "Test output".to_string(),
            success: true,
        }];
        
        let checkpoint_id = mgr.create_checkpoint(
            "test-conversation",
            1,
            messages.clone(),
            vec![], // file_changes
            tool_results.clone(),
        ).unwrap();
        
        // 获取检查点
        let checkpoint = mgr.get_checkpoint(&checkpoint_id).unwrap().unwrap();
        
        assert_eq!(checkpoint.conversation_id, "test-conversation");
        assert_eq!(checkpoint.iteration, 1);
        assert_eq!(checkpoint.new_messages.len(), 1);
        assert_eq!(checkpoint.tool_results.len(), 1);
        
        cleanup_test_db(&db_path);
    }
    
    #[test]
    fn test_get_latest_checkpoint() {
        let (mut mgr, db_path) = setup_test_db();
        
        // 创建多个检查点，每次等待确保时间戳不同
        for i in 1..=3 {
            mgr.create_checkpoint(
                "test-conversation",
                i,
                vec![],
                vec![], // file_changes
                vec![],
            ).unwrap();
            
            // 等待 10ms 确保时间戳不同
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        
        // 获取最新的检查点
        let latest = mgr.get_latest_checkpoint("test-conversation").unwrap().unwrap();
        
        assert_eq!(latest.iteration, 3);
        
        cleanup_test_db(&db_path);
    }
    
    #[test]
    fn test_rollback_to_checkpoint() {
        let (mut mgr, db_path) = setup_test_db();
        
        let checkpoint_id = mgr.create_checkpoint(
            "test-conversation",
            5,
            vec![],
            vec![], // file_changes
            vec![],
        ).unwrap();
        
        // 回滚到检查点
        let checkpoint = mgr.rollback_to_checkpoint(&checkpoint_id).unwrap();
        
        assert_eq!(checkpoint.id, checkpoint_id);
        assert_eq!(checkpoint.iteration, 5);
        
        cleanup_test_db(&db_path);
    }
    
    #[test]
    fn test_cleanup_old_checkpoints() {
        let (mut mgr, db_path) = setup_test_db();
        
        // 创建检查点
        mgr.create_checkpoint("test", 1, vec![], vec![], vec![]).unwrap();
        
        // 等待确保时间戳已设置
        std::thread::sleep(std::time::Duration::from_millis(10));
        
        // 清理过期检查点（保留 0 小时，即全部删除）
        let deleted = mgr.cleanup_old_checkpoints(0).unwrap();
        
        assert_eq!(deleted, 1);
        
        // 验证已删除
        let count = mgr.get_checkpoint_count("test").unwrap();
        assert_eq!(count, 0);
        
        cleanup_test_db(&db_path);
    }
    
    #[test]
    fn test_list_checkpoints() {
        let (mut mgr, db_path) = setup_test_db();
        
        // 创建多个检查点
        for i in 1..=3 {
            mgr.create_checkpoint("test", i, vec![], vec![], vec![]).unwrap();
        }
        
        // 列出所有检查点
        let summaries = mgr.list_checkpoints("test").unwrap();
        
        assert_eq!(summaries.len(), 3);
        assert_eq!(summaries[0].iteration, 1);
        assert_eq!(summaries[1].iteration, 2);
        assert_eq!(summaries[2].iteration, 3);
        
        cleanup_test_db(&db_path);
    }
    
    #[test]
    fn test_backup_and_restore_file() {
        use std::io::Write;
        
        // 创建测试文件
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join(format!("test_file_{}.txt", Uuid::new_v4()));
        let test_content = "Hello, Checkpoint!";
        
        // 写入测试内容
        let mut file = fs::File::create(&test_file).unwrap();
        file.write_all(test_content.as_bytes()).unwrap();
        drop(file);
        
        // 备份文件
        let file_path = test_file.to_string_lossy().to_string();
        let change_opt = backup_file_if_needed(&file_path).unwrap();
        
        assert!(change_opt.is_some());
        let change = change_opt.unwrap();
        
        match &change {
            FileChange::Modified { path, backup_path } => {
                assert_eq!(path, &file_path);
                assert!(std::path::Path::new(backup_path).exists());
            }
            _ => panic!("Expected Modified variant"),
        }
        
        // 修改文件内容
        let mut file = fs::File::create(&test_file).unwrap();
        file.write_all(b"Modified content").unwrap();
        drop(file);
        
        // 从备份恢复
        restore_from_backup(&change).unwrap();
        
        // 验证恢复成功
        let restored_content = fs::read_to_string(&test_file).unwrap();
        assert_eq!(restored_content, test_content);
        
        // 清理
        let _ = fs::remove_file(&test_file);
    }
    
    #[test]
    fn test_file_creation_rollback() {
        use std::io::Write;
        
        // 模拟创建文件的回滚
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join(format!("test_new_file_{}.txt", Uuid::new_v4()));
        let file_path = test_file.to_string_lossy().to_string();
        
        // 创建文件
        let mut file = fs::File::create(&test_file).unwrap();
        file.write_all(b"New file content").unwrap();
        drop(file);
        
        assert!(test_file.exists());
        
        // 回滚（删除创建的文件）
        let change = FileChange::Created {
            path: file_path.clone(),
            backup_path: None,
        };
        restore_from_backup(&change).unwrap();
        
        // 验证文件已删除
        assert!(!test_file.exists());
    }
    
    #[test]
    fn test_batch_file_rollback() {
        use std::io::Write;
        
        let temp_dir = std::env::temp_dir();
        
        // 创建多个测试文件
        let mut changes = vec![];
        let mut files = vec![];
        
        for i in 0..3 {
            let test_file = temp_dir.join(format!("test_batch_{}_{}.txt", i, Uuid::new_v4()));
            let file_path = test_file.to_string_lossy().to_string();
            
            // 创建并备份
            let mut file = fs::File::create(&test_file).unwrap();
            file.write_all(format!("Content {}", i).as_bytes()).unwrap();
            drop(file);
            
            if let Some(change) = backup_file_if_needed(&file_path).unwrap() {
                changes.push(change);
                files.push(test_file);
            }
        }
        
        // 修改所有文件
        for file in &files {
            let mut f = fs::File::create(file).unwrap();
            f.write_all(b"Modified").unwrap();
            drop(f);
        }
        
        // 批量回滚
        rollback_file_changes(&changes).unwrap();
        
        // 验证所有文件已恢复
        for (i, file) in files.iter().enumerate() {
            let content = fs::read_to_string(file).unwrap();
            assert_eq!(content, format!("Content {}", i));
        }
        
        // 清理
        for file in &files {
            let _ = fs::remove_file(file);
        }
    }
}
