//! Codex Agent 适配器
//!
//! 通过 CLI 调用 OpenAI Codex，避免复杂的依赖问题。
//! 使用 stdio/JSON-RPC 进行通信。

use std::process::{Command, Stdio};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::process::Child;

use crate::error::{AppError, AppResult};
use tracing::{info, warn, error};
use serde_json::{json, Value};

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

/// Codex Agent 实例（通过 CLI 调用）
pub struct CodexAgent {
    config: CodexAgentConfig,
    codex_path: String,
}

impl CodexAgent {
    /// 创建新的 Codex Agent 实例
    pub async fn new(config: CodexAgentConfig) -> AppResult<Self> {
        info!("🤖 Initializing Codex Agent (CLI mode) with model: {}", config.model);
        
        // 检查 codex CLI 是否可用
        let codex_path = Self::find_codex_binary()?;
        
        info!("✅ Codex Agent initialized at: {}", codex_path);
        
        Ok(Self {
            config,
            codex_path,
        })
    }
    
    /// 查找 Codex 二进制文件
    fn find_codex_binary() -> AppResult<String> {
        // 优先使用配置的路径
        if let Ok(path) = std::env::var("CODEX_BINARY_PATH") {
            if std::path::Path::new(&path).exists() {
                return Ok(path);
            }
        }
        
        // 尝试在 PATH 中查找
        #[cfg(target_os = "windows")]
        let binary_name = "codex.exe";
        #[cfg(not(target_os = "windows"))]
        let binary_name = "codex";
        
        // TODO: 实现 PATH 搜索逻辑
        // 暂时返回错误，提示用户配置 CODEX_BINARY_PATH
        Err(AppError::Internal(
            "Codex CLI not found. Please set CODEX_BINARY_PATH environment variable.".into()
        ))
    }
    
    /// 启动一个新的对话线程（通过 CLI）
    pub async fn start_thread(&self) -> AppResult<CodexThreadHandle> {
        info!("🧵 Starting new Codex thread via CLI");
        
        // 启动 codex CLI 进程
        let child = tokio::process::Command::new(&self.codex_path)
            .arg("chat")
            .arg("--json")  // JSON 模式
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| AppError::Internal(format!("Failed to spawn codex CLI: {}", e)))?;
        
        let thread_id = format!("codex_cli_{}", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis());
        
        info!("✅ Codex CLI process started: {}", thread_id);
        
        Ok(CodexThreadHandle {
            child: Some(child),
            thread_id,
        })
    }
    
    /// 发送消息并获取流式响应（通过 CLI stdio）
    pub async fn send_message_stream(
        &self,
        thread_handle: &mut CodexThreadHandle,
        message: &str,
    ) -> AppResult<mpsc::Receiver<String>> {
        info!("💬 Sending message to Codex CLI");
        
        let (tx, rx) = mpsc::channel(100);
        
        if let Some(ref mut child) = thread_handle.child {
            // 写入消息到 stdin
            if let Some(mut stdin) = child.stdin.take() {
                use tokio::io::AsyncWriteExt;
                let msg_json = json!({
                    "type": "user_message",
                    "content": message
                }).to_string();
                
                stdin.write_all(msg_json.as_bytes()).await
                    .map_err(|e| AppError::Internal(format!("Failed to write to codex stdin: {}", e)))?;
                stdin.write_all(b"\n").await
                    .map_err(|e| AppError::Internal(format!("Failed to write newline: {}", e)))?;
                
                // 重新设置 stdin（如果需要继续对话）
                child.stdin = Some(stdin);
            }
            
            // 启动后台任务读取 stdout 并转发到 tx
            let mut stdout = child.stdout.take()
                .ok_or_else(|| AppError::Internal("Codex CLI stdout not available".into()))?;
            
            let tx_clone = tx.clone();
            tokio::spawn(async move {
                use tokio::io::{AsyncBufReadExt, BufReader};
                let reader = BufReader::new(stdout);
                let mut lines = reader.lines();
                
                while let Ok(Some(line)) = lines.next_line().await {
                    // 解析 Codex JSON 响应
                    if let Ok(json_value) = serde_json::from_str::<Value>(&line) {
                        // 提取文本内容
                        if let Some(text) = extract_text_from_codex_response(&json_value) {
                            if tx_clone.send(text).await.is_err() {
                                break; // 接收端已关闭
                            }
                        }
                    }
                }
                
                info!("🧵 Codex CLI stream ended");
            });
        }
        
        Ok(rx)
    }
}

/// Codex 线程句柄（CLI 进程）
pub struct CodexThreadHandle {
    child: Option<Child>,
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
        info!("🧵 Dropping Codex CLI thread: {}", self.thread_id);
        // 终止子进程
        if let Some(ref mut child) = self.child {
            let _ = child.start_kill();
        }
    }
}

/// 从 Codex JSON 响应中提取文本内容
fn extract_text_from_codex_response(json: &Value) -> Option<String> {
    // TODO: 根据实际的 Codex JSON 格式解析
    // 这里是一个通用的实现，需要根据实际情况调整
    
    // 尝试常见的字段路径
    if let Some(text) = json.get("content").and_then(|v| v.as_str()) {
        return Some(text.to_string());
    }
    
    if let Some(delta) = json.get("delta").and_then(|v| v.as_str()) {
        return Some(delta.to_string());
    }
    
    if let Some(text) = json.get("text").and_then(|v| v.as_str()) {
        return Some(text.to_string());
    }
    
    None
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
