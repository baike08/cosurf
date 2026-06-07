/// MCP 客户端
/// 负责与 MCP Server 通信
/// 
/// 遵循 MCP (Model Context Protocol) 开源标准：
/// - https://modelcontextprotocol.io/
/// - 支持 Streamable HTTP 传输（POST JSON-RPC 到 URL）
/// - 支持 SSE (Server-Sent Events) 传输（GET 建立 SSE → 获取 endpoint → POST）
/// - 支持 JSON-RPC 2.0 协议

use tracing::{info, warn};
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::error::{AppError, AppResult};

// ==================== MCP 协议数据结构 ====================

/// JSON-RPC 请求
#[derive(Debug, Serialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: u64,
    method: String,
    params: serde_json::Value,
}

/// JSON-RPC 响应
#[derive(Debug, Deserialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Option<u64>,
    result: Option<serde_json::Value>,
    error: Option<JsonRpcError>,
}

/// JSON-RPC 错误
#[derive(Debug, Deserialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    data: Option<serde_json::Value>,
}

/// MCP 初始化请求
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct InitializeRequest {
    protocol_version: String,
    capabilities: ClientCapabilities,
    client_info: ClientInfo,
}

#[derive(Debug, Serialize)]
struct ClientCapabilities {
    roots: Option<RootsCapability>,
    sampling: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RootsCapability {
    list_changed: bool,
}

#[derive(Debug, Serialize)]
struct ClientInfo {
    name: String,
    version: String,
}

/// MCP 工具调用请求
#[derive(Debug, Serialize)]
struct CallToolRequest {
    name: String,
    arguments: serde_json::Value,
}

// ==================== 传输模式 ====================

/// MCP 传输模式
#[derive(Debug, Clone, PartialEq)]
pub enum McpTransport {
    /// Streamable HTTP: 直接 POST JSON-RPC 到 URL
    /// 响应可以是 application/json 或 text/event-stream
    StreamableHttp,
    /// SSE: 先 GET 建立 SSE 连接获取 endpoint，再 POST 到 endpoint
    Sse,
}

// ==================== MCP 客户端 ====================

/// MCP 客户端
pub struct McpClient {
    server_url: String,
    transport: McpTransport,
    http_client: reqwest::Client,
    /// SSE 模式下的 POST endpoint URL
    sse_endpoint: Option<String>,
    /// 请求 ID 计数器
    next_id: u64,
}

impl McpClient {
    /// 创建新的 MCP 客户端
    pub fn new(
        server_url: String,
        transport: McpTransport,
        api_key: Option<String>,
        custom_headers: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Self {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "Content-Type",
            reqwest::header::HeaderValue::from_static("application/json"),
        );
        headers.insert(
            "Accept",
            reqwest::header::HeaderValue::from_static("application/json, text/event-stream"),
        );
        
        // 设置 API Key（兼容旧参数）
        if let Some(ref key) = api_key {
            if !key.is_empty() {
                headers.insert(
                    "Authorization",
                    reqwest::header::HeaderValue::from_str(&format!("Bearer {}", key))
                        .unwrap_or_else(|_| reqwest::header::HeaderValue::from_static("")),
                );
            }
        }
        
        // 设置自定义 headers（如 X-API-Key）
        if let Some(ref custom) = custom_headers {
            for (key, value) in custom {
                if let Some(val_str) = value.as_str() {
                    if let (Ok(name), Ok(val)) = (
                        reqwest::header::HeaderName::from_bytes(key.as_bytes()),
                        reqwest::header::HeaderValue::from_str(val_str),
                    ) {
                        headers.insert(name, val);
                    }
                }
            }
        }
        
        let http_client = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(60))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        
        Self {
            server_url,
            transport,
            http_client,
            sse_endpoint: None,
            next_id: 1,
        }
    }
    
    fn next_request_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
    
    /// 初始化 MCP 连接
    pub async fn initialize(&mut self) -> AppResult<()> {
        info!(server_url = %self.server_url, transport = ?self.transport, "Initializing MCP connection");
        
        // SSE 模式：先建立 SSE 连接获取 endpoint
        if self.transport == McpTransport::Sse {
            self.connect_sse().await?;
        }
        
        let init_request = InitializeRequest {
            protocol_version: "2024-11-05".to_string(),
            capabilities: ClientCapabilities {
                roots: Some(RootsCapability {
                    list_changed: false,
                }),
                sampling: None,
            },
            client_info: ClientInfo {
                name: "CoSurf".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
        };
        
        let response = self.send_request("initialize", serde_json::to_value(init_request)?).await?;
        info!(response = ?response, "MCP initialize response");
        
        // 发送 initialized 通知
        self.send_notification("initialized", serde_json::json!({})).await?;
        
        info!("MCP connection initialized successfully");
        Ok(())
    }
    
    /// 调用 MCP 工具
    pub async fn call_tool(&mut self, tool_name: &str, arguments: &serde_json::Value) -> AppResult<String> {
        info!(tool_name = %tool_name, "Calling MCP tool");
        
        let call_request = CallToolRequest {
            name: tool_name.to_string(),
            arguments: arguments.clone(),
        };
        
        let response = self.send_request("tools/call", serde_json::to_value(call_request)?).await?;
        info!(tool_name = %tool_name, response_keys = ?response.as_object().map(|o| o.keys().collect::<Vec<_>>()), "MCP tool call response");
        
        // send_request 已提取 JSON-RPC 的 result 字段，
        // response 即 MCP 工具返回值：{ content: [{ type: "text", text: "..." }], isError?: bool }
        
        // 检查 MCP 工具级错误
        if response.get("isError").and_then(|v| v.as_bool()).unwrap_or(false) {
            let error_text = response.get("content")
                .and_then(|c| c.as_array())
                .and_then(|arr| arr.first())
                .and_then(|item| item.get("text"))
                .and_then(|t| t.as_str())
                .unwrap_or("Unknown MCP tool error");
            return Err(AppError::Internal(format!("MCP tool error: {}", error_text)));
        }
        
        // MCP 工具返回格式：{ content: [{ type: "text", text: "..." }] }
        if let Some(content) = response.get("content") {
            if let Some(content_array) = content.as_array() {
                let texts: Vec<String> = content_array
                    .iter()
                    .filter_map(|item| {
                        item.get("text")
                            .and_then(|t| t.as_str())
                            .map(|s| s.to_string())
                    })
                    .collect();
                
                if !texts.is_empty() {
                    return Ok(texts.join("\n"));
                }
            }
        }
        
        // 非标准格式，直接返回 JSON
        Ok(serde_json::to_string_pretty(&response)?)
    }
    
    /// 列出可用工具
    pub async fn list_tools(&mut self) -> AppResult<serde_json::Value> {
        info!("Listing available MCP tools");
        
        let response = self.send_request("tools/list", serde_json::json!({})).await?;
        
        Ok(response)
    }
    
    // ==================== Streamable HTTP 传输 ====================
    
    /// Streamable HTTP: POST JSON-RPC 到 URL
    /// 支持 application/json 和 text/event-stream 两种响应格式
    async fn send_streamable_http(&mut self, method: &str, params: serde_json::Value) -> AppResult<serde_json::Value> {
        let request_id = self.next_request_id();
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: request_id,
            method: method.to_string(),
            params,
        };
        
        let url = self.server_url.trim_end_matches('/').to_string();
        info!("StreamableHTTP POST to: {} method: {} id: {}", url, method, request_id);
        
        let response = self.http_client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to send MCP request to {}: {}", url, e)))?;
        
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::Internal(
                format!("MCP server returned error {}: {} (url: {})", status, body, url)
            ));
        }
        
        // 检查 Content-Type 决定如何解析
        let content_type = response.headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("application/json")
            .to_string();
        
        if content_type.contains("text/event-stream") {
            // SSE 流式响应：从事件流中提取 JSON-RPC 响应
            Self::parse_sse_response(response, Some(request_id)).await
        } else {
            // 标准 JSON 响应
            Self::parse_json_response(response).await
        }
    }
    
    // ==================== SSE 传输 ====================
    
    /// SSE: 建立 SSE 连接，获取 POST endpoint
    async fn connect_sse(&mut self) -> AppResult<()> {
        let url = self.server_url.trim_end_matches('/').to_string();
        info!("SSE: Connecting to {} for endpoint", url);
        
        // GET 请求建立 SSE 连接
        let response = self.http_client
            .get(&url)
            .header("Accept", "text/event-stream")
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to connect SSE endpoint {}: {}", url, e)))?;
        
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::Internal(
                format!("SSE endpoint returned error {}: {} (url: {})", status, body, url)
            ));
        }
        
        // 从 SSE 流中读取 endpoint 事件
        let endpoint = Self::read_sse_endpoint(response, &url).await?;
        
        info!(endpoint = %endpoint, "SSE: Got endpoint URL");
        self.sse_endpoint = Some(endpoint);
        
        Ok(())
    }
    
    /// 从 SSE 流中读取 endpoint 事件
    async fn read_sse_endpoint(response: reqwest::Response, base_url: &str) -> AppResult<String> {
        use futures::StreamExt;
        use tokio::io::AsyncBufReadExt;
        
        let byte_stream = response.bytes_stream();
        // 将 reqwest 的 bytes stream 转为 AsyncRead
        let mapped = futures::StreamExt::map(byte_stream, |r| {
            r.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
        });
        let reader = tokio_util::io::StreamReader::new(mapped);
        let mut buf_reader = tokio::io::BufReader::new(reader);
        let mut line = String::new();
        
        // 逐行读取 SSE 事件，等待 endpoint 事件（最多 30 秒）
        let timeout = tokio::time::timeout(Duration::from_secs(30), async {
            loop {
                line.clear();
                match buf_reader.read_line(&mut line).await {
                    Ok(0) => {
                        return Err(AppError::Internal(
                            "SSE connection closed before receiving endpoint".to_string()
                        ));
                    }
                    Ok(_) => {
                        let trimmed = line.trim();
                        if let Some(data) = trimmed.strip_prefix("data:") {
                            let data = data.trim();
                            if !data.is_empty() {
                                // 尝试解析 endpoint
                                if let Some(ep) = Self::resolve_endpoint(data, base_url) {
                                    return Ok(ep);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        return Err(AppError::Internal(
                            format!("Error reading SSE stream: {}", e)
                        ));
                    }
                }
            }
        });
        
        match timeout.await {
            Ok(result) => result,
            Err(_) => Err(AppError::Internal(
                "Timed out waiting for SSE endpoint event (30s)".to_string()
            )),
        }
    }
    
    /// 解析 endpoint URL（支持相对路径和完整 URL）
    fn resolve_endpoint(data: &str, base_url: &str) -> Option<String> {
        if data.is_empty() {
            return None;
        }
        
        // 完整 URL
        if data.starts_with("http://") || data.starts_with("https://") {
            return Some(data.to_string());
        }
        
        // 相对路径：基于 base_url 拼接
        if let Ok(base) = url::Url::parse(base_url) {
            if let Ok(resolved) = base.join(data) {
                return Some(resolved.to_string());
            }
        }
        
        // 兜底拼接
        let base = base_url.trim_end_matches('/');
        let path = if data.starts_with('/') { data.to_string() } else { format!("/{}", data) };
        Some(format!("{}{}", base, path))
    }
    
    /// SSE: POST JSON-RPC 到 endpoint
    async fn send_sse(&mut self, method: &str, params: serde_json::Value) -> AppResult<serde_json::Value> {
        let endpoint = self.sse_endpoint.as_ref()
            .ok_or_else(|| AppError::Internal(
                "SSE session not established. Call initialize() first.".to_string()
            ))?.clone();
        
        let request_id = self.next_request_id();
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: request_id,
            method: method.to_string(),
            params,
        };
        
        info!("SSE POST to: {} method: {} id: {}", endpoint, method, request_id);
        
        let response = self.http_client
            .post(&endpoint)
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to POST to SSE endpoint {}: {}", endpoint, e)))?;
        
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::Internal(
                format!("SSE endpoint returned error {}: {} (url: {})", status, body, endpoint)
            ));
        }
        
        // SSE 模式响应可能是 JSON 或 SSE 流
        let content_type = response.headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("application/json")
            .to_string();
        
        if content_type.contains("text/event-stream") {
            Self::parse_sse_response(response, Some(request_id)).await
        } else {
            Self::parse_json_response(response).await
        }
    }
    
    // ==================== 通用解析方法 ====================
    
    /// 解析标准 JSON-RPC 响应
    async fn parse_json_response(response: reqwest::Response) -> AppResult<serde_json::Value> {
        let json_response: JsonRpcResponse = response.json().await
            .map_err(|e| AppError::Internal(format!("Failed to parse MCP JSON response: {}", e)))?;
        
        if let Some(error) = json_response.error {
            return Err(AppError::Internal(
                format!("MCP error {}: {}", error.code, error.message)
            ));
        }
        
        json_response.result.ok_or_else(|| {
            AppError::Internal("MCP response has no result".to_string())
        })
    }
    
    /// 解析 SSE 流式响应，提取 JSON-RPC 结果
    async fn parse_sse_response(response: reqwest::Response, expected_id: Option<u64>) -> AppResult<serde_json::Value> {
        let body = response.text().await
            .map_err(|e| AppError::Internal(format!("Failed to read SSE response body: {}", e)))?;
        
        info!("SSE response body (first 500 chars): {}", &body[..body.len().min(500)]);
        
        // 解析 SSE 事件流
        let mut last_result: Option<serde_json::Value> = None;
        
        for line in body.lines() {
            let line = line.trim();
            if let Some(data) = line.strip_prefix("data:") {
                let data = data.trim();
                if data.is_empty() || data == "[DONE]" {
                    continue;
                }
                // 尝试解析为 JSON-RPC 响应
                if let Ok(rpc_response) = serde_json::from_str::<JsonRpcResponse>(data) {
                    // 如果指定了 expected_id，验证 ID 匹配
                    if let Some(eid) = expected_id {
                        if let Some(rid) = rpc_response.id {
                            if rid != eid {
                                continue; // 跳过不匹配的响应
                            }
                        }
                    }
                    if let Some(error) = rpc_response.error {
                        return Err(AppError::Internal(
                            format!("MCP error {}: {}", error.code, error.message)
                        ));
                    }
                    if let Some(result) = rpc_response.result {
                        last_result = Some(result);
                    }
                }
            }
        }
        
        last_result.ok_or_else(|| {
            AppError::Internal("No valid JSON-RPC result found in SSE stream".to_string())
        })
    }
    
    // ==================== 统一接口 ====================
    
    /// 发送 JSON-RPC 请求（根据传输模式自动选择）
    async fn send_request(&mut self, method: &str, params: serde_json::Value) -> AppResult<serde_json::Value> {
        match self.transport {
            McpTransport::StreamableHttp => self.send_streamable_http(method, params).await,
            McpTransport::Sse => self.send_sse(method, params).await,
        }
    }
    
    /// 发送通知（不需要响应）
    async fn send_notification(&self, method: &str, params: serde_json::Value) -> AppResult<()> {
        let notification = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params
        });
        
        let url = match (&self.transport, &self.sse_endpoint) {
            (McpTransport::StreamableHttp, _) => self.server_url.trim_end_matches('/').to_string(),
            (McpTransport::Sse, Some(ep)) => ep.clone(),
            (McpTransport::Sse, None) => self.server_url.trim_end_matches('/').to_string(),
        };
        
        let _ = self.http_client
            .post(&url)
            .json(&notification)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to send MCP notification: {}", e)))?;
        
        Ok(())
    }
}
