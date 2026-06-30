//! 统一的工具注册表
//!
//! 借鉴 Codex 的工具系统设计，提供统一的工具接口和注册机制。
//! 整合 Skills、MCP、内置工具到统一的注册表中。

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use serde_json::Value;
use crate::error::{AppError, AppResult};
use tracing::{info, warn};
use async_trait::async_trait;

/// 工具执行结果
#[derive(Debug, Clone)]
pub struct ToolResult {
    /// 输出内容
    pub output: String,
    /// 是否成功
    pub success: bool,
    /// 错误信息（如果失败）
    pub error: Option<String>,
    /// 执行耗时（毫秒）
    pub duration_ms: Option<u64>,
}

impl ToolResult {
    pub fn success(output: impl Into<String>) -> Self {
        Self {
            output: output.into(),
            success: true,
            error: None,
            duration_ms: None,
        }
    }

    pub fn error(error: impl Into<String>) -> Self {
        Self {
            output: String::new(),
            success: false,
            error: Some(error.into()),
            duration_ms: None,
        }
    }
}

/// 统一的工具 Trait
///
/// 所有工具（Skills、MCP、内置工具）都需要实现这个 trait
#[async_trait]
pub trait Tool: Send + Sync {
    /// 工具名称（唯一标识）
    fn name(&self) -> &str;
    
    /// 工具描述（用于 LLM 理解）
    fn description(&self) -> &str;
    
    /// JSON Schema（用于参数验证）
    fn schema(&self) -> Value;
    
    /// 执行工具
    async fn execute(&self, args: &Value) -> AppResult<ToolResult>;
    
    /// 是否需要用户确认（默认不需要）
    fn requires_approval(&self) -> bool {
        false
    }
}

/// 工具注册表
///
/// 管理所有可用的工具，提供统一的查询和执行接口
pub struct ToolRegistry {
    tools: RwLock<HashMap<String, Arc<dyn Tool>>>,
}

impl ToolRegistry {
    /// 创建新的工具注册表
    pub fn new() -> Self {
        Self {
            tools: RwLock::new(HashMap::new()),
        }
    }

    /// 注册一个工具
    pub async fn register(&self, tool: Arc<dyn Tool>) {
        let name = tool.name().to_string();
        info!("🔧 Registering tool: {}", name);
        self.tools.write().await.insert(name, tool);
    }

    /// 获取工具的 Schema
    pub async fn get_schema(&self, name: &str) -> Option<Value> {
        self.tools.read().await.get(name).map(|t| t.schema())
    }

    /// 获取所有工具的 Schemas（用于发送给 LLM）
    pub async fn get_all_schemas(&self) -> Vec<Value> {
        self.tools.read().await.values()
            .map(|t| t.schema())
            .collect()
    }

    /// 执行工具
    pub async fn execute(&self, name: &str, args: &Value) -> AppResult<ToolResult> {
        let start_time = std::time::Instant::now();
        
        let tool = self.tools.read().await.get(name)
            .cloned()
            .ok_or_else(|| AppError::NotFound(format!("Tool '{}' not found", name)))?;
        
        info!("🔧 Executing tool: {} with args: {}", name, serde_json::to_string(args).unwrap_or_default());
        
        let result = tool.execute(args).await?;
        
        let duration_ms = start_time.elapsed().as_millis() as u64;
        info!("✅ Tool {} completed in {}ms", name, duration_ms);
        
        // 返回带有耗时的结果
        Ok(ToolResult {
            duration_ms: Some(duration_ms),
            ..result
        })
    }

    /// 检查工具是否存在
    pub async fn has_tool(&self, name: &str) -> bool {
        self.tools.read().await.contains_key(name)
    }

    /// 获取所有工具名称
    pub async fn list_tools(&self) -> Vec<String> {
        self.tools.read().await.keys().cloned().collect()
    }

    /// 清空所有工具
    pub async fn clear(&self) {
        self.tools.write().await.clear();
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局工具注册表实例
static GLOBAL_REGISTRY: once_cell::sync::Lazy<Arc<ToolRegistry>> = 
    once_cell::sync::Lazy::new(|| Arc::new(ToolRegistry::new()));

/// 获取全局工具注册表
pub fn global_registry() -> &'static Arc<ToolRegistry> {
    &GLOBAL_REGISTRY
}

/// 初始化全局注册表（在应用启动时调用）
pub async fn init_global_registry() -> AppResult<()> {
    let registry = global_registry();
    
    // TODO: 在这里注册所有工具
    // 1. 注册内置工具
    // 2. 注册 Skills
    // 3. 注册 MCP 工具
    
    info!("✅ Global tool registry initialized with {} tools", 
          registry.list_tools().await.len());
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // 测试工具实现
    struct EchoTool;

    impl Tool for EchoTool {
        fn name(&self) -> &str {
            "echo"
        }

        fn description(&self) -> &str {
            "Echo back the input message"
        }

        fn schema(&self) -> Value {
            json!({
                "type": "object",
                "properties": {
                    "message": {
                        "type": "string",
                        "description": "Message to echo"
                    }
                },
                "required": ["message"]
            })
        }

        async fn execute(&self, args: &Value) -> AppResult<ToolResult> {
            let message = args["message"].as_str().unwrap_or("");
            Ok(ToolResult::success(format!("Echo: {}", message)))
        }
    }

    #[tokio::test]
    async fn test_tool_registry() {
        let registry = ToolRegistry::new();
        
        // 注册工具
        registry.register(Arc::new(EchoTool)).await;
        
        // 检查工具存在
        assert!(registry.has_tool("echo").await);
        
        // 获取 schema
        let schema = registry.get_schema("echo").await;
        assert!(schema.is_some());
        
        // 执行工具
        let result = registry.execute("echo", &json!({"message": "Hello"})).await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.success);
        assert_eq!(result.output, "Echo: Hello");
    }
}
