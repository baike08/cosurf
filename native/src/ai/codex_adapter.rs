//! Codex Agent 适配器
//!
//! 将 OpenAI Codex 的核心 Agent 引擎适配到 CoSurf。
//! 提供与现有 stream.rs 兼容的接口。

use std::sync::Arc;
use tokio::sync::mpsc;

use crate::error::{AppError, AppResult};
use tracing::{info, warn, error};

// Codex 核心类型
use codex_core::{
    ThreadManager,
    Config,
    NewThread,
    CodexThread,
};

/// Codex Agent 配置
#[derive(Debug, Clone)]
pub struct CodexAgentConfig {
    /// 模型提供商（openai/openrouter/等）
    pub provider: String,
    /// 模型名称
    pub model: String,
    /// API Key
    pub api_key: String,
    /// Base URL（可选，用于兼容端点）
    pub base_url: Option<String>,
    /// 工作目录
    pub cwd: String,
    /// Codex 家目录（存储状态）
    pub codex_home: String,
}

/// Codex Agent 实例
pub struct CodexAgent {
    thread_manager: Arc<ThreadManager>,
    config: CodexAgentConfig,
}

impl CodexAgent {
    /// 创建新的 Codex Agent 实例
    pub async fn new(config: CodexAgentConfig) -> AppResult<Self> {
        info!("🤖 Initializing Codex Agent with model: {}", config.model);
        
        // 构建 Codex Config
        let codex_config = Self::build_codex_config(&config)?;
        
        // 创建 ThreadManager
        let thread_manager = Arc::new(
            ThreadManager::start(codex_config.clone())
                .await
                .map_err(|e| AppError::Internal(format!("Failed to start ThreadManager: {}", e)))?
        );
        
        info!("✅ Codex Agent initialized");
        
        Ok(Self {
            thread_manager,
            config,
        })
    }
    
    /// 启动一个新的对话线程
    pub async fn start_thread(&self) -> AppResult<CodexThreadHandle> {
        info!("🧵 Starting new Codex thread");
        
        let codex_config = Self::build_codex_config(&self.config)?;
        
        let new_thread = self.thread_manager
            .start_thread(codex_config)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to start thread: {}", e)))?;
        
        info!("✅ Thread started: {:?}", new_thread.thread_id);
        
        Ok(CodexThreadHandle {
            thread: new_thread.thread,
            thread_id: new_thread.thread_id,
        })
    }
    
    /// 发送消息并获取流式响应
    pub async fn send_message_stream(
        &self,
        thread_handle: &mut CodexThreadHandle,
        message: &str,
    ) -> AppResult<mpsc::Receiver<String>> {
        info!("💬 Sending message to Codex thread");
        
        let (tx, rx) = mpsc::channel(100);
        
        // TODO: 实现消息发送和流式响应
        // 这里需要调用 Codex Thread 的 turn 方法
        
        warn!("⚠️  send_message_stream not fully implemented yet");
        
        Ok(rx)
    }
    
    /// 构建 Codex Config
    fn build_codex_config(config: &CodexAgentConfig) -> AppResult<Config> {
        // TODO: 根据 Codex 的 Config 结构构建配置
        // 这需要研究 codex-core 的 Config 类型
        
        warn!("⚠️  build_codex_config needs implementation");
        
        // 临时返回错误，等待完整实现
        Err(AppError::Internal("Codex config building not implemented".into()))
    }
}

/// Codex 线程句柄
pub struct CodexThreadHandle {
    thread: CodexThread,
    thread_id: String,
}

impl CodexThreadHandle {
    /// 获取线程 ID
    pub fn thread_id(&self) -> &str {
        &self.thread_id
    }
    
    /// 取消当前操作
    pub async fn cancel(&self) -> AppResult<()> {
        // TODO: 实现取消逻辑
        Ok(())
    }
}

impl Drop for CodexThreadHandle {
    fn drop(&mut self) {
        info!("🧵 Dropping Codex thread: {}", self.thread_id);
        // TODO: 清理资源
    }
}

/// 全局 Codex Agent 实例（单例）
static GLOBAL_CODEX_AGENT: once_cell::sync::Lazy<tokio::sync::RwLock<Option<Arc<CodexAgent>>>> = 
    once_cell::sync::Lazy::new(|| {
        tokio::sync::RwLock::new(None)
    });

/// 初始化全局 Codex Agent
pub async fn init_global_codex_agent(config: CodexAgentConfig) -> AppResult<()> {
    let agent = CodexAgent::new(config).await?;
    *GLOBAL_CODEX_AGENT.write().await = Some(Arc::new(agent));
    info!("✅ Global Codex Agent initialized");
    Ok(())
}

/// 获取全局 Codex Agent
pub async fn global_codex_agent() -> Option<Arc<CodexAgent>> {
    GLOBAL_CODEX_AGENT.read().await.clone()
}

/// 使用 Codex Agent 进行流式聊天（替代现有的 stream_chat_completion）
pub async fn stream_with_codex(
    message: &str,
    conversation_id: &str,
    // TODO: 添加 callbacks 参数
) -> AppResult<()> {
    let agent = global_codex_agent().await
        .ok_or_else(|| AppError::Internal("Codex Agent not initialized".into()))?;
    
    let mut thread = agent.start_thread().await?;
    let mut stream = agent.send_message_stream(&mut thread, message).await?;
    
    // TODO: 处理流式响应并转发到 callbacks
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    #[ignore] // 需要有效的 API Key
    async fn test_codex_agent_basic_flow() {
        let config = CodexAgentConfig {
            provider: "openai".to_string(),
            model: "gpt-4o".to_string(),
            api_key: "sk-test".to_string(),
            base_url: None,
            cwd: "/tmp".to_string(),
            codex_home: "/tmp/codex".to_string(),
        };
        
        let agent = CodexAgent::new(config).await.unwrap();
        let mut thread = agent.start_thread().await.unwrap();
        
        // TODO: 测试消息发送
    }
}
