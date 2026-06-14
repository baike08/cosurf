//! 上下文管理器（借鉴 OpenClaw）
//!
//! 功能：
//! - 冻结关键消息（系统提示词、用户原始问题）
//! - 智能压缩可压缩消息
//! - Token 预算控制

use crate::ai::provider::ChatMessage;

/// 上下文管理器
pub struct ContextManager {
    /// 冻结的消息（不会被压缩）
    frozen_messages: Vec<ChatMessage>,
    /// 可压缩的消息
    compressible_messages: Vec<ChatMessage>,
    /// 当前总 Token 数（估算）
    current_tokens: usize,
}

impl ContextManager {
    /// 创建新的上下文管理器
    pub fn new(messages: Vec<ChatMessage>) -> Self {
        let mut frozen = vec![];
        let mut compressible = vec![];
        
        // 分类消息
        for msg in messages {
            if Self::should_freeze(&msg) {
                frozen.push(msg);
            } else {
                compressible.push(msg);
            }
        }
        
        let current_tokens = Self::estimate_all_tokens(&frozen) + Self::estimate_all_tokens(&compressible);
        
        tracing::info!(
            "📦 ContextManager initialized: frozen={}, compressible={}, tokens={}",
            frozen.len(),
            compressible.len(),
            current_tokens
        );
        
        Self {
            frozen_messages: frozen,
            compressible_messages: compressible,
            current_tokens,
        }
    }
    
    /// 判断消息是否应该冻结
    fn should_freeze(msg: &ChatMessage) -> bool {
        // 冻结系统提示词
        if msg.role == "system" {
            return true;
        }
        
        // 冻结用户的第一个问题（原始意图）
        if msg.role == "user" && msg.tool_call_id.is_none() {
            // 简单启发式：如果这是第一条用户消息，冻结它
            // 实际实现中可以更复杂，比如检查是否是会话的第一条用户消息
            return true;
        }
        
        false
    }
    
    /// 估算单个消息的 Token 数（静态方法）
    fn estimate_msg_tokens(msg: &ChatMessage) -> usize {
        // 简单估算：每个字符约 0.5 tokens
        let content_tokens = msg.content.len() / 2;
        let name_tokens = msg.name.as_ref().map(|n| n.len() / 2).unwrap_or(0);
        content_tokens + name_tokens
    }
    
    /// 估算所有消息的 Token 数（静态方法）
    fn estimate_all_tokens(messages: &[ChatMessage]) -> usize {
        messages.iter().map(ContextManager::estimate_msg_tokens).sum()
    }
    
    /// 获取所有消息（冻结 + 可压缩）
    pub fn messages(&self) -> Vec<ChatMessage> {
        let mut all = self.frozen_messages.clone();
        all.extend(self.compressible_messages.clone());
        all
    }
    
    /// 添加工具执行结果
    pub fn add_tool_result(&mut self, tool_call: &crate::ai::tools::ToolCall, result: &crate::ai::tools::ToolResult) {
        let tool_msg = ChatMessage {
            role: "tool".to_string(),
            content: result.output.clone(),
            name: Some(tool_call.name.clone()),
            tool_call_id: Some(tool_call.id.clone()),
        };
        
        let tokens = Self::estimate_msg_tokens(&tool_msg);
        self.current_tokens += tokens;
        self.compressible_messages.push(tool_msg);
        
        tracing::debug!("➕ Added tool result: {} ({} tokens)", tool_call.name, tokens);
    }
    
    /// 添加助手回复
    pub fn add_assistant_message(&mut self, content: String) {
        let msg = ChatMessage {
            role: "assistant".to_string(),
            content,
            name: None,
            tool_call_id: None,
        };
        
        let tokens = Self::estimate_msg_tokens(&msg);
        self.current_tokens += tokens;
        self.compressible_messages.push(msg);
        
        tracing::debug!("➕ Added assistant message ({} tokens)", tokens);
    }
    
    /// 添加用户消息
    pub fn add_user_message(&mut self, content: String) {
        let msg = ChatMessage {
            role: "user".to_string(),
            content,
            name: None,
            tool_call_id: None,
        };
        
        let tokens = Self::estimate_msg_tokens(&msg);
        self.current_tokens += tokens;
        self.compressible_messages.push(msg);
        
        tracing::debug!("➕ Added user message ({} tokens)", tokens);
    }
    
    /// 估算当前 Token 数
    pub fn estimate_tokens(&self) -> usize {
        self.current_tokens
    }
    
    /// 如果需要，压缩上下文
    /// 
    /// 参数:
    /// - token_limit: Token 上限
    /// - target_ratio: 压缩目标比例（例如 0.8 表示压缩到 80%）
    pub fn compress_if_needed(&mut self, token_limit: usize, target_ratio: f64) {
        let target_tokens = (token_limit as f64 * target_ratio) as usize;
        
        if self.current_tokens <= target_tokens {
            tracing::debug!("✅ No compression needed: {} <= {}", self.current_tokens, target_tokens);
            return;
        }
        
        tracing::warn!(
            "🗜️  Compressing context: {} > {} (target: {})",
            self.current_tokens,
            token_limit,
            target_tokens
        );
        
        // 策略1: 移除中间的工具调用结果（保留最近的）
        self.remove_old_tool_results();
        
        // 策略2: 截断长消息
        self.truncate_long_messages();
        
        // 重新计算 Token 数
        self.current_tokens = Self::estimate_all_tokens(&self.compressible_messages);
        
        tracing::info!(
            "✅ Compression complete: {} tokens (frozen: {}, compressible: {})",
            self.current_tokens,
            self.frozen_messages.len(),
            self.compressible_messages.len()
        );
    }
    
    /// 移除旧的工具调用结果（保留最近的 N 个）
    fn remove_old_tool_results(&mut self) {
        let keep_count = 10; // 保留最近 10 个工具结果
        
        // 从后往前找工具消息
        let mut tool_result_indices = vec![];
        for (i, msg) in self.compressible_messages.iter().enumerate() {
            if msg.role == "tool" {
                tool_result_indices.push(i);
            }
        }
        
        // 如果超过限制，移除旧的
        if tool_result_indices.len() > keep_count {
            let remove_count = tool_result_indices.len() - keep_count;
            let indices_to_remove: Vec<usize> = tool_result_indices[..remove_count].to_vec();
            
            tracing::info!("🗑️  Removing {} old tool results", indices_to_remove.len());
            
            // 从后往前删除（避免索引偏移）
            for &idx in indices_to_remove.iter().rev() {
                self.compressible_messages.remove(idx);
            }
        }
    }
    
    /// 截断过长的消息
    fn truncate_long_messages(&mut self) {
        let max_content_length = 2000; // 最大内容长度
        
        for msg in &mut self.compressible_messages {
            if msg.content.len() > max_content_length {
                let original_len = msg.content.len();
                msg.content = format!(
                    "[内容已截断，原长度: {}]\n{}",
                    original_len,
                    &msg.content[..max_content_length]
                );
                
                tracing::debug!("✂️  Truncated message from {} to {}", original_len, max_content_length);
            }
        }
    }
    
    /// 冻结重要消息（在循环中动态识别）
    pub fn freeze_important_messages(&mut self) {
        // 找出重要的工具结果（例如成功的 MCP 工具调用）
        let mut important_indices = vec![];
        
        for (i, msg) in self.compressible_messages.iter().enumerate() {
            // 冻结成功的 MCP 工具结果
            if msg.role == "tool" 
                && msg.name.as_ref().map(|n| n.starts_with("mcp_")).unwrap_or(false)
            {
                important_indices.push(i);
            }
        }
        
        // 移动重要消息到冻结区
        if !important_indices.is_empty() {
            tracing::info!("❄️  Freezing {} important messages", important_indices.len());
            
            // 从后往前移动（避免索引偏移）
            for &idx in important_indices.iter().rev() {
                let msg = self.compressible_messages.remove(idx);
                self.frozen_messages.push(msg);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_context_manager_creation() {
        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: "You are a helpful assistant.".to_string(),
                name: None,
                tool_call_id: None,
            },
            ChatMessage {
                role: "user".to_string(),
                content: "Hello!".to_string(),
                name: None,
                tool_call_id: None,
            },
        ];
        
        let mgr = ContextManager::new(messages);
        assert_eq!(mgr.frozen_messages.len(), 2); // system + first user
        assert_eq!(mgr.compressible_messages.len(), 0);
    }
    
    #[test]
    fn test_token_estimation() {
        let msg = ChatMessage {
            role: "user".to_string(),
            content: "Hello world".to_string(),
            name: None,
            tool_call_id: None,
        };
        
        let tokens = ContextManager::estimate_msg_tokens(&msg);
        assert!(tokens > 0);
    }
}
