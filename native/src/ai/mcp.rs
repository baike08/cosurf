//! MCP (Model Context Protocol) 客户端
//!
//! 从 src-tauri/src/ai/mcp.rs 迁移。
//! 用于与 MCP Server 通信，获取外部工具和能力。

use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::error::{AppError, AppResult};

/// MCP 工具定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema", alias = "input_schema")]
    pub input_schema: serde_json::Value,
}

/// MCP 资源定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResource {
    pub uri: String,
    pub name: String,
    pub description: String,
    pub mime_type: Option<String>,
}

/// MCP 客户端配置
#[derive(Debug, Clone)]
pub struct McpConfig {
    pub server_url: String,
    pub api_key: Option<String>,
}

impl McpConfig {
    pub fn new(server_url: String) -> Self {
        Self {
            server_url,
            api_key: None,
        }
    }
}

/// MCP 传输模式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum McpTransport {
    StreamableHttp,
    Sse,
    Stdio,
}

/// MCP 客户端
pub struct McpClient {
    config: McpConfig,
    transport: McpTransport,
    headers: Option<serde_json::Map<String, serde_json::Value>>,
    tools: Vec<McpTool>,
    resources: Vec<McpResource>,
}

impl std::fmt::Debug for McpClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("McpClient")
            .field("config", &self.config)
            .field("transport", &self.transport)
            .field("tools_count", &self.tools.len())
            .field("resources_count", &self.resources.len())
            .finish()
    }
}

impl McpClient {
    pub fn new(
        config: McpConfig,
        transport: McpTransport,
        headers: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Self {
        Self {
            config,
            transport,
            headers,
            tools: Vec::new(),
            resources: Vec::new(),
        }
    }

    /// 初始化并列出可用工具
    pub async fn initialize(&mut self) -> AppResult<()> {
        info!(server = %self.config.server_url, transport = ?self.transport, "Initializing MCP client");

        match self.transport {
            McpTransport::StreamableHttp | McpTransport::Sse => {
                self.initialize_http().await?;
            }
            McpTransport::Stdio => {
                // stdio 模式需要 spawn 子进程，暂不实现
                warn!("MCP stdio transport not yet implemented in native module");
            }
        }

        info!(tools = self.tools.len(), resources = self.resources.len(), "MCP client initialized");
        Ok(())
    }

    /// HTTP/SSE 模式初始化
    async fn initialize_http(&mut self) -> AppResult<()> {
        let client = reqwest::Client::new();

        // 构建 JSON-RPC 请求
        let init_request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "cosurf-native",
                    "version": env!("CARGO_PKG_VERSION")
                }
            }
        });

        let mut req = client.post(&self.config.server_url);

        // 添加自定义 headers
        if let Some(ref headers) = self.headers {
            for (key, value) in headers {
                if let Some(v) = value.as_str() {
                    req = req.header(key.as_str(), v);
                }
            }
        }
        
        // 只有在 headers 中没有认证信息时，才添加 Authorization header
        let has_auth_header = self.headers.as_ref().map_or(false, |h| {
            h.contains_key("Authorization") || h.contains_key("X-API-Key") || h.contains_key("x-api-key")
        });
        
        if !has_auth_header {
            if let Some(ref api_key) = self.config.api_key {
                req = req.header("Authorization", format!("Bearer {}", api_key));
            }
        }

        req = req.json(&init_request);

        let response = req.send().await
            .map_err(|e| AppError::AiProvider(format!("MCP initialize request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::AiProvider(format!(
                "MCP server returned error status: {}",
                response.status()
            )));
        }

        // 解析初始化响应
        let _init_result: serde_json::Value = response.json().await
            .map_err(|e| AppError::AiProvider(format!("Failed to parse MCP init response: {}", e)))?;

        // 列出工具
        let list_request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/list",
            "params": {}
        });

        let mut req = client.post(&self.config.server_url);
        if let Some(ref headers) = self.headers {
            for (key, value) in headers {
                if let Some(v) = value.as_str() {
                    req = req.header(key.as_str(), v);
                }
            }
        }
        
        // 只有在 headers 中没有认证信息时，才添加 Authorization header
        let has_auth_header = self.headers.as_ref().map_or(false, |h| {
            h.contains_key("Authorization") || h.contains_key("X-API-Key") || h.contains_key("x-api-key")
        });
        
        if !has_auth_header {
            if let Some(ref api_key) = self.config.api_key {
                req = req.header("Authorization", format!("Bearer {}", api_key));
            }
        }

        let response = req.json(&list_request).send().await
            .map_err(|e| AppError::AiProvider(format!("MCP tools/list request failed: {}", e)))?;

        tracing::info!("📡 MCP tools/list response status: {}", response.status());
        
        let tools_result: serde_json::Value = response.json().await
            .map_err(|e| AppError::AiProvider(format!("Failed to parse tools/list response: {}", e)))?;

        tracing::info!("📦 MCP tools/list raw response: {}", serde_json::to_string_pretty(&tools_result).unwrap_or_default());

        // 解析工具列表
        if let Some(tools_array) = tools_result.get("result")
            .and_then(|r| r.get("tools"))
            .and_then(|t| t.as_array())
        {
            tracing::info!("✅ Found {} tools in response", tools_array.len());
            for tool_value in tools_array {
                match serde_json::from_value::<McpTool>(tool_value.clone()) {
                    Ok(tool) => {
                        tracing::info!("🔧 Parsed tool: {}", tool.name);
                        self.tools.push(tool);
                    }
                    Err(e) => {
                        tracing::error!("❌ Failed to parse tool: {}, error: {}", 
                            serde_json::to_string_pretty(tool_value).unwrap_or_default(), e);
                    }
                }
            }
        }

        Ok(())
    }

    /// 获取所有可用的工具
    pub fn list_tools(&self) -> &[McpTool] {
        &self.tools
    }

    /// 获取所有可用的资源
    pub fn list_resources(&self) -> &[McpResource] {
        &self.resources
    }

    /// 调用 MCP 工具
    pub async fn call_tool(&self, tool_name: &str, arguments: &serde_json::Value) -> AppResult<serde_json::Value> {
        info!(tool = %tool_name, "Calling MCP tool");

        match self.transport {
            McpTransport::StreamableHttp | McpTransport::Sse => {
                self.call_tool_http(tool_name, arguments).await
            }
            McpTransport::Stdio => {
                Err(AppError::Internal("MCP stdio transport not yet implemented".into()))
            }
        }
    }

    /// HTTP 模式调用工具
    async fn call_tool_http(&self, tool_name: &str, arguments: &serde_json::Value) -> AppResult<serde_json::Value> {
        let client = reqwest::Client::new();

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {
                "name": tool_name,
                "arguments": arguments
            }
        });

        let mut req = client.post(&self.config.server_url);
        if let Some(ref headers) = self.headers {
            for (key, value) in headers {
                if let Some(v) = value.as_str() {
                    req = req.header(key.as_str(), v);
                }
            }
        }
        if let Some(ref api_key) = self.config.api_key {
            req = req.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = req.json(&request).send().await
            .map_err(|e| AppError::AiProvider(format!("MCP tool call failed: {}", e)))?;

        let result: serde_json::Value = response.json().await
            .map_err(|e| AppError::AiProvider(format!("Failed to parse tool call response: {}", e)))?;

        // 提取 result
        if let Some(result_value) = result.get("result") {
            Ok(result_value.clone())
        } else if let Some(error) = result.get("error") {
            Err(AppError::AiProvider(format!("MCP tool error: {}", error)))
        } else {
            Ok(result)
        }
    }

    /// 获取工具 Schema（用于 LLM function calling）
    pub fn get_tool_schemas(&self, server_name: &str) -> Vec<serde_json::Value> {
        let server_safe = server_name.replace("-", "_").replace(" ", "_");

        self.tools.iter().map(|tool| {
            let func_name = format!("mcp_{}_{}", server_safe, tool.name.replace("-", "_"));
            serde_json::json!({
                "type": "function",
                "function": {
                    "name": func_name,
                    "description": format!("[MCP:{}] {}", server_name, tool.description),
                    "parameters": tool.input_schema
                }
            })
        }).collect()
    }
}
