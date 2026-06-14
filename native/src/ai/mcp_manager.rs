//! MCP 工具管理器
//! 
//! 负责：
//! 1. 加载所有启用的 MCP Servers 的工具
//! 2. 注册工具到 Agent Loop（命名格式 mcp_{server}_{tool}）
//! 3. 维护路由映射表：function_name → (server_config, original_tool_name)
//! 4. 执行 MCP 工具调用

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tracing::{info, error};

use crate::ai::mcp::{McpClient, McpConfig, McpTransport};
use crate::error::{AppError, AppResult};

/// MCP Server 配置（从数据库读取）
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct McpServerConfig {
    pub id: String,
    pub name: String,
    #[serde(rename = "serverType")]
    pub server_type: String,
    pub url: Option<String>,
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    pub env: Option<HashMap<String, String>>,
    pub headers: Option<HashMap<String, String>>,
    pub enabled: bool,
}

/// MCP 工具路由信息
#[derive(Debug, Clone)]
pub struct McpToolRoute {
    pub server_id: String,
    pub server_name: String,
    pub original_tool_name: String,
    pub client: Arc<Mutex<McpClient>>,
}

/// MCP 工具管理器
pub struct McpToolManager {
    /// 路由表：function_name → McpToolRoute
    routes: HashMap<String, McpToolRoute>,
    /// 工具 schemas（用于 LLM function calling）
    tool_schemas: Vec<serde_json::Value>,
}

impl McpToolManager {
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
            tool_schemas: Vec::new(),
        }
    }

    /// 加载所有启用的 MCP Servers 的工具
    pub async fn load_all_servers(&mut self, servers: Vec<McpServerConfig>) -> AppResult<()> {
        info!("🔄 Loading tools from {} MCP servers", servers.len());
        
        self.routes.clear();
        self.tool_schemas.clear();
        
        for server in servers {
            if !server.enabled {
                info!("⏭️  Skipping disabled MCP server: {}", server.name);
                continue;
            }
            
            match self.load_server_tools(server).await {
                Ok(count) => {
                    info!("✅ Loaded {} tools from MCP server", count);
                }
                Err(e) => {
                    error!("❌ Failed to load tools from MCP server: {}", e);
                }
            }
        }
        
        info!("📊 Total MCP tools loaded: {}", self.tool_schemas.len());
        Ok(())
    }

    /// 加载单个 MCP Server 的工具
    async fn load_server_tools(&mut self, server: McpServerConfig) -> AppResult<usize> {
        info!("🔧 Loading tools from MCP server: {} (type={})", server.name, server.server_type);
        
        // 创建 MCP Client
        let client = match self.create_mcp_client(&server).await {
            Ok(client) => Arc::new(Mutex::new(client)),
            Err(e) => {
                return Err(AppError::Internal(format!("Failed to create MCP client: {}", e)));
            }
        };
        
        // 获取工具列表
        let tools = {
            let c = client.lock().unwrap();
            c.list_tools().to_vec()
        };
        
        info!("📦 Found {} tools from server {}", tools.len(), server.name);
        
        // 注册每个工具
        let mut count = 0;
        for tool in &tools {
            let function_name = format!("mcp_{}_{}", server.name.replace("-", "_").replace(" ", "_"), tool.name);
            
            // 添加到路由表
            self.routes.insert(function_name.clone(), McpToolRoute {
                server_id: server.id.clone(),
                server_name: server.name.clone(),
                original_tool_name: tool.name.clone(),
                client: client.clone(),
            });
            
            // 生成工具 schema
            let schema = serde_json::json!({
                "type": "function",
                "function": {
                    "name": function_name,
                    "description": format!("[{}] {}", server.name, tool.description),
                    "parameters": tool.input_schema
                }
            });
            
            self.tool_schemas.push(schema);
            count += 1;
        }
        
        Ok(count)
    }

    /// 创建 MCP Client
    async fn create_mcp_client(&self, server: &McpServerConfig) -> AppResult<McpClient> {
        let transport = match server.server_type.to_lowercase().as_str() {
            "sse" => McpTransport::Sse,
            "streamablehttp" | "http" => McpTransport::StreamableHttp,
            "stdio" => McpTransport::Stdio,
            _ => McpTransport::StreamableHttp,
        };
        
        let config = McpConfig {
            server_url: server.url.clone().unwrap_or_default(),
            api_key: None, // API Key 应该在 headers 中
        };
        
        // 转换 headers 为 serde_json::Map
        let headers = if let Some(ref h) = server.headers {
            let mut map = serde_json::Map::new();
            for (key, value) in h {
                map.insert(key.clone(), serde_json::Value::String(value.clone()));
            }
            Some(map)
        } else {
            None
        };
        
        let mut client = McpClient::new(config, transport, headers);
        
        client.initialize().await?;
        
        Ok(client)
    }

    /// 获取所有工具 schemas（用于 LLM function calling）
    pub fn get_tool_schemas(&self) -> Vec<serde_json::Value> {
        tracing::info!("📊 McpToolManager::get_tool_schemas: returning {} schemas (routes={})", 
            self.tool_schemas.len(), self.routes.len());
        self.tool_schemas.clone()
    }

    /// 执行 MCP 工具调用
    pub async fn execute_tool(&self, function_name: &str, arguments: &serde_json::Value) -> AppResult<String> {
        let route = self.routes.get(function_name)
            .ok_or_else(|| AppError::Internal(format!("Unknown MCP tool: {}", function_name)))?;
        
        info!("🔧 Executing MCP tool: {} (server={}, original_tool={})", 
            function_name, route.server_name, route.original_tool_name);
        
        let client = route.client.lock().unwrap();
        
        // 调用 MCP 工具
        match client.call_tool(&route.original_tool_name, arguments).await {
            Ok(result) => {
                info!("✅ MCP tool executed successfully");
                Ok(serde_json::to_string(&result).unwrap_or_default())
            }
            Err(e) => {
                error!("❌ MCP tool execution failed: {}", e);
                Err(AppError::Internal(format!("MCP tool execution failed: {}", e)))
            }
        }
    }
}

// 实现 Clone for Arc<Mutex<McpToolManager>>
impl Clone for McpToolManager {
    fn clone(&self) -> Self {
        // 注意：这里不能直接 clone，因为 routes 包含 Arc<Mutex<McpClient>>
        // 实际使用时应该用 Arc<Mutex<McpToolManager>>
        panic!("McpToolManager should be wrapped in Arc<Mutex<>>")
    }
}
