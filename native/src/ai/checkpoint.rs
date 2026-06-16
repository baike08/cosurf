//! Checkpoint 管理器 - Agent Loop 状态持久化
//!
//! 功能：
//! - 创建检查点（保存 Agent Loop 的中间状态）
//! - 回滚到指定检查点（失败时恢复状态）
//! - 清理过期检查点（自动维护）

use rusqlite::{Connection, params, OptionalExtension};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};
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
    
    /// 工具执行结果记录
    pub tool_results: Vec<ToolResultRecord>,
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
        mgr.create_checkpoint("test", 1, vec![], vec![]).unwrap();
        
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
            mgr.create_checkpoint("test", i, vec![], vec![]).unwrap();
        }
        
        // 列出所有检查点
        let summaries = mgr.list_checkpoints("test").unwrap();
        
        assert_eq!(summaries.len(), 3);
        assert_eq!(summaries[0].iteration, 1);
        assert_eq!(summaries[1].iteration, 2);
        assert_eq!(summaries[2].iteration, 3);
        
        cleanup_test_db(&db_path);
    }
}
