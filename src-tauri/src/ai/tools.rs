use serde::{Deserialize, Serialize};

/// AI 工具调用定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

/// 工具调用结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub tool_call_id: String,
    pub output: String,
    pub success: bool,
}

/// 可用的内置工具列表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BuiltInTool {
    /// 总结当前页面内容
    SummarizePage,
    /// 在当前页面执行操作（点击、填写表单等）
    WebAgent,
    /// 打开新的网页
    OpenUrl,
    /// 翻译页面内容
    Translate,
    /// 导出为 Markdown
    ExportMarkdown,
    /// 联网搜索
    WebSearch,
    /// 执行 shell 命令
    RunCommand,
}

impl BuiltInTool {
    pub fn name(&self) -> &str {
        match self {
            Self::SummarizePage => "summarize_page",
            Self::WebAgent => "web_agent",
            Self::OpenUrl => "open_url",
            Self::Translate => "translate",
            Self::ExportMarkdown => "export_markdown",
            Self::WebSearch => "web_search",
            Self::RunCommand => "run_command",
        }
    }

    pub fn description(&self) -> &str {
        match self {
            Self::SummarizePage => "总结当前页面的主要内容",
            Self::WebAgent => "在当前网页执行自动化操作，如点击按钮、填写表单",
            Self::OpenUrl => "打开新的网页URL",
            Self::Translate => "翻译当前页面内容为指定语言",
            Self::ExportMarkdown => "将当前页面内容导出为 Markdown 格式",
            Self::WebSearch => "搜索互联网获取最新信息",
            Self::RunCommand => "在系统终端执行 shell 命令（支持 Windows cmd / Linux/macOS sh），捕获 stdout/stderr 返回结果",
        }
    }

    pub fn parameters(&self) -> serde_json::Value {
        match self {
            Self::SummarizePage => {
                serde_json::json!({
                    "type": "object",
                    "properties": {
                        "max_length": {
                            "type": "integer",
                            "description": "最大摘要长度（字符数），默认 500"
                        }
                    }
                })
            }
            Self::WebAgent => {
                serde_json::json!({
                    "type": "object",
                    "properties": {
                        "action": {
                            "type": "string",
                            "enum": ["click", "fill", "select", "scroll", "wait"],
                            "description": "要执行的操作类型"
                        },
                        "selector": {
                            "type": "string",
                            "description": "CSS 选择器"
                        },
                        "value": {
                            "type": "string",
                            "description": "填写的值（仅 fill 操作需要）"
                        }
                    },
                    "required": ["action", "selector"]
                })
            }
            Self::OpenUrl => {
                serde_json::json!({
                    "type": "object",
                    "properties": {
                        "url": {
                            "type": "string",
                            "description": "要打开的网页URL，必须以 http:// 或 https:// 开头"
                        }
                    },
                    "required": ["url"]
                })
            }
            Self::Translate => {
                serde_json::json!({
                    "type": "object",
                    "properties": {
                        "target_language": {
                            "type": "string",
                            "description": "目标语言，如 'zh', 'en', 'ja'"
                        }
                    },
                    "required": ["target_language"]
                })
            }
            Self::ExportMarkdown => {
                serde_json::json!({
                    "type": "object",
                    "properties": {}
                })
            }
            Self::WebSearch => {
                serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "搜索查询词"
                        },
                        "engine_type": {
                            "type": "string",
                            "enum": ["Generic", "News", "Academic"],
                            "description": "搜索引擎类型，默认 Generic",
                            "default": "Generic"
                        },
                        "time_range": {
                            "type": "string",
                            "enum": ["OneDay", "OneWeek", "OneMonth", "OneYear", "NoLimit"],
                            "description": "时间范围，默认 OneWeek",
                            "default": "OneWeek"
                        },
                        "max_results": {
                            "type": "integer",
                            "description": "最大结果数（1-20），默认 5",
                            "minimum": 1,
                            "maximum": 20,
                            "default": 5
                        }
                    },
                    "required": ["query"]
                })
            }
            Self::RunCommand => {
                serde_json::json!({
                    "type": "object",
                    "properties": {
                        "command": {
                            "type": "string",
                            "description": "要执行的 shell 命令（Windows 上通过 cmd /C 执行，Linux/macOS 上通过 sh -c 执行）"
                        },
                        "working_dir": {
                            "type": "string",
                            "description": "命令执行的工作目录（可选，默认使用系统默认目录）"
                        },
                        "timeout": {
                            "type": "integer",
                            "description": "命令超时时间（秒），默认 30 秒，最大 120 秒",
                            "minimum": 1,
                            "maximum": 120,
                            "default": 30
                        }
                    },
                    "required": ["command"]
                })
            }
        }
    }

    /// 转换为 OpenAI function calling 格式
    pub fn to_openai_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "function",
            "function": {
                "name": self.name(),
                "description": self.description(),
                "parameters": self.parameters()
            }
        })
    }
}

/// 获取所有可用工具的 schema（同步版本，仅内置工具）
pub fn get_available_tools_schemas() -> Vec<serde_json::Value> {
    vec![
        BuiltInTool::SummarizePage.to_openai_schema(),
        BuiltInTool::WebAgent.to_openai_schema(),
        BuiltInTool::OpenUrl.to_openai_schema(),
        BuiltInTool::Translate.to_openai_schema(),
        BuiltInTool::ExportMarkdown.to_openai_schema(),
        BuiltInTool::WebSearch.to_openai_schema(),
        BuiltInTool::RunCommand.to_openai_schema(),
    ]
}

/// 获取所有可用工具的 schema（异步版本，包含 Skills 和 MCP）
pub async fn get_available_tools_schemas_async(app: &tauri::AppHandle) -> Vec<serde_json::Value> {
    let mut schemas = get_available_tools_schemas();
    
    // 添加 Skills 工具
    if let Some(skills_schemas) = get_skills_tool_schemas_async(app).await {
        schemas.extend(skills_schemas);
    }
    
    // 添加 MCP Server 工具
    if let Some(mcp_schemas) = get_mcp_tools_schemas_async(app).await {
        schemas.extend(mcp_schemas);
    }
    
    schemas
}

/// 从 Skills 系统获取工具 schema（异步版本，仅返回 description）
async fn get_skills_tool_schemas_async(app: &tauri::AppHandle) -> Option<Vec<serde_json::Value>> {
    use crate::state::AppState;
    use tauri::Manager;

    let state = app.state::<AppState>();

    // 使用全局 SkillsManager（避免重复创建和扫描目录）
    let skills_manager = match state.skills_manager.lock() {
        Ok(mgr) => mgr,
        Err(e) => {
            tracing::warn!("Failed to lock skills manager: {}", e);
            return None;
        }
    };

    // 获取所有启用的 Skills
    let enabled_skills = skills_manager.get_enabled_skills();

    if enabled_skills.is_empty() {
        return None;
    }

    let mut schemas = Vec::new();

    for skill in enabled_skills {
        // 渐进式加载：仅将 description 暴露给模型，不暴露完整 SKILL.md
        // 模型调用 skill_{id} 后，dispatcher 会懒加载完整内容作为 tool result 返回
        let schema = serde_json::json!({
            "type": "function",
            "function": {
                "name": format!("skill_{}", skill.id),
                "description": skill.description,
                "parameters": {
                    "type": "object",
                    "properties": {},
                    "additionalProperties": true
                }
            }
        });

        schemas.push(schema);
    }

    Some(schemas)
}

/// 从 MCP Servers 获取工具 schema（异步版本）
/// 
/// 连接每个启用的 MCP Server，拉取真实的 tools/list，
/// 将每个 MCP 工具直接注册为 Agent Loop 的独立 function。
/// 命名规则：mcp_{server_safe_name}_{tool_name}
async fn get_mcp_tools_schemas_async(app: &tauri::AppHandle) -> Option<Vec<serde_json::Value>> {
    use crate::state::AppState;
    use crate::ai::skills_executors::mcp::McpTransport;
    use tauri::Manager;
    
    let state = app.state::<AppState>();
    
    // 获取所有启用的 MCP Servers
    let mcp_servers = {
        let db = match state.db.lock() {
            Ok(db) => db,
            Err(_) => return None,
        };
        match db.list_mcp_servers() {
            Ok(servers) => servers.into_iter()
                .filter(|s| s.enabled && !s.disabled)
                .collect::<Vec<_>>(),
            Err(_) => return None,
        }
    };
    
    if mcp_servers.is_empty() {
        return None;
    }
    
    let mut schemas = Vec::new();
    let mut registry = std::collections::HashMap::new();
    
    for server in &mcp_servers {
        let server_safe_name = server.name.replace("-", "_").replace(" ", "_");
        
        // 确定传输模式
        let transport = match server.server_type {
            crate::db::settings::McpServerType::Http
            | crate::db::settings::McpServerType::StreamableHttp => McpTransport::StreamableHttp,
            crate::db::settings::McpServerType::Sse => McpTransport::Sse,
            crate::db::settings::McpServerType::Stdio => {
                // stdio 模式：启动子进程并获取工具列表
                tracing::info!("🔧 MCP server '{}' is stdio type, connecting via subprocess", server.name);
                
                let command = match &server.command {
                    Some(cmd) => cmd.clone(),
                    None => {
                        tracing::warn!("MCP server '{}' has no command for stdio mode, skipping", server.name);
                        continue;
                    }
                };
                
                let args: Vec<String> = server.args.clone().unwrap_or_default();
                
                // 转换 env: Option<Map<String, Value>> -> Option<HashMap<String, String>>
                let env = server.env.as_ref().map(|env_map| {
                    env_map.iter()
                        .filter_map(|(k, v)| {
                            v.as_str().map(|s| (k.clone(), s.to_string()))
                        })
                        .collect::<std::collections::HashMap<String, String>>()
                });
                
                // 连接 stdio MCP Server（带超时保护）
                let tools_result = tokio::time::timeout(
                    std::time::Duration::from_secs(15),
                    fetch_stdio_mcp_server_tools(command, args, env)
                ).await;
                
                let mcp_tools = match tools_result {
                    Ok(Ok(tools)) => {
                        tracing::info!("MCP server '{}' (stdio) returned {} tools", server.name, tools.len());
                        tools
                    }
                    Ok(Err(e)) => {
                        tracing::warn!("Failed to fetch tools from MCP server '{}' (stdio): {}", server.name, e);
                        continue;
                    }
                    Err(_) => {
                        tracing::warn!("Timeout fetching tools from MCP server '{}' (stdio)", server.name);
                        continue;
                    }
                };
                
                // 注册 stdio 工具
                for tool in mcp_tools {
                    register_mcp_tool(&mut schemas, &mut registry, &server_safe_name, &server.name, tool);
                }
                
                continue; // 已处理完毕，跳过后续的 HTTP 逻辑
            }
        };
        
        let url = match &server.url {
            Some(u) => u.clone(),
            None => {
                tracing::warn!("MCP server '{}' has no URL, skipping", server.name);
                continue;
            }
        };
        
        // 连接 MCP Server 拉取真实工具列表（带超时保护）
        let tools_result = tokio::time::timeout(
            std::time::Duration::from_secs(15),
            fetch_mcp_server_tools(url.clone(), transport.clone(), server.headers.clone())
        ).await;
        
        let mcp_tools = match tools_result {
            Ok(Ok(tools)) => {
                tracing::info!("MCP server '{}' returned {} tools", server.name, tools.len());
                tools
            }
            Ok(Err(e)) => {
                tracing::warn!("Failed to fetch tools from MCP server '{}': {}", server.name, e);
                continue;
            }
            Err(_) => {
                tracing::warn!("Timeout fetching tools from MCP server '{}'", server.name);
                continue;
            }
        };
        
        // 将每个 MCP 工具注册为独立的 function
        for tool in mcp_tools {
            register_mcp_tool(&mut schemas, &mut registry, &server_safe_name, &server.name, tool);
        }
    }
    
    // 更新全局 registry（供 dispatcher 路由使用）
    if let Ok(mut reg) = state.mcp_tool_registry.lock() {
        *reg = registry;
    }
    
    if schemas.is_empty() {
        return None;
    }
    
    Some(schemas)
}

/// 注册单个 MCP 工具到 schema 和 registry
fn register_mcp_tool(
    schemas: &mut Vec<serde_json::Value>,
    registry: &mut std::collections::HashMap<String, (String, String)>,
    server_safe_name: &str,
    server_name: &str,
    tool: serde_json::Value,
) {
    let tool_name = match tool.get("name").and_then(|v| v.as_str()) {
        Some(n) => n,
        None => return,
    };
    let tool_desc = tool.get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let input_schema = tool.get("inputSchema")
        .cloned()
        .unwrap_or(serde_json::json!({
            "type": "object",
            "properties": {},
            "additionalProperties": true
        }));
    
    // 函数名: mcp_{server}_{tool}
    let func_name = format!("mcp_{}_{}", server_safe_name, tool_name.replace("-", "_"));
    let description = format!("[MCP:{}] {}", server_name, tool_desc);
    
    registry.insert(func_name.clone(), (server_name.to_string(), tool_name.to_string()));
    
    let schema = serde_json::json!({
        "type": "function",
        "function": {
            "name": func_name,
            "description": description,
            "parameters": input_schema
        }
    });
    
    schemas.push(schema);
}

/// 连接 MCP Server 并拉取 tools/list
async fn fetch_mcp_server_tools(
    url: String,
    transport: crate::ai::skills_executors::mcp::McpTransport,
    headers: Option<serde_json::Map<String, serde_json::Value>>,
) -> crate::error::AppResult<Vec<serde_json::Value>> {
    use crate::ai::skills_executors::mcp::McpClient;
    
    let mut client = McpClient::new(url, transport, None, headers);
    client.initialize().await?;
    let result = client.list_tools().await?;
    
    // MCP tools/list 返回: { tools: [{ name, description, inputSchema }, ...] }
    let tools = result.get("tools")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    
    Ok(tools)
}

/// 通过 stdio 连接 MCP Server 并拉取 tools/list
async fn fetch_stdio_mcp_server_tools(
    command: String,
    args: Vec<String>,
    env: Option<std::collections::HashMap<String, String>>,
) -> crate::error::AppResult<Vec<serde_json::Value>> {
    use tokio::process::Command;
    use tokio::io::{AsyncWriteExt, AsyncBufReadExt, BufReader};
    use std::time::Duration;
    
    tracing::info!("🚀 Starting stdio MCP server: {} {:?}", command, args);
    
    // 启动子进程
    let mut child = Command::new(&command)
        .args(&args)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| crate::error::AppError::Internal(format!("Failed to spawn process: {}", e)))?;
    
    let mut stdin = child.stdin.take()
        .ok_or_else(|| crate::error::AppError::Internal("Failed to get stdin".to_string()))?;
    let stdout = child.stdout.take()
        .ok_or_else(|| crate::error::AppError::Internal("Failed to get stdout".to_string()))?;
    let stderr = child.stderr.take()
        .ok_or_else(|| crate::error::AppError::Internal("Failed to get stderr".to_string()))?;
    
    let mut reader = BufReader::new(stdout);
    let mut stderr_reader = BufReader::new(stderr);
    
    // 读取 stderr 日志（非阻塞）
    tokio::spawn(async move {
        let mut line = String::new();
        while let Ok(n) = stderr_reader.read_line(&mut line).await {
            if n == 0 { break; }
            tracing::debug!("MCP stderr: {}", line.trim());
            line.clear();
        }
    });
    
    // 发送 initialize 请求
    let init_request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "cosurf-agent",
                "version": "0.1.0"
            }
        }
    });
    
    let init_str = serde_json::to_string(&init_request)
        .map_err(|e| crate::error::AppError::Internal(format!("JSON serialization failed: {}", e)))?;
    stdin.write_all(init_str.as_bytes()).await
        .map_err(|e| crate::error::AppError::Internal(format!("Failed to write to stdin: {}", e)))?;
    stdin.write_all(b"\n").await
        .map_err(|e| crate::error::AppError::Internal(format!("Failed to write newline: {}", e)))?;
    stdin.flush().await
        .map_err(|e| crate::error::AppError::Internal(format!("Failed to flush stdin: {}", e)))?;
    
    tracing::debug!("Sent initialize request");
    
    // 读取 initialize 响应
    let mut response_line = String::new();
    match tokio::time::timeout(Duration::from_secs(5), reader.read_line(&mut response_line)).await {
        Ok(Ok(_)) => {},
        Ok(Err(e)) => return Err(crate::error::AppError::Internal(format!("Failed to read response: {}", e))),
        Err(_) => return Err(crate::error::AppError::Internal("Timeout waiting for initialize response".to_string())),
    }
    
    let init_response: serde_json::Value = serde_json::from_str(&response_line)
        .map_err(|e| crate::error::AppError::Internal(format!("Invalid initialize response: {}", e)))?;
    
    tracing::debug!("Received initialize response: {}", serde_json::to_string_pretty(&init_response).unwrap_or_default());
    
    // 发送 initialized 通知
    let initialized_notification = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized"
    });
    
    let notif_str = serde_json::to_string(&initialized_notification)
        .map_err(|e| crate::error::AppError::Internal(format!("JSON serialization failed: {}", e)))?;
    stdin.write_all(notif_str.as_bytes()).await
        .map_err(|e| crate::error::AppError::Internal(format!("Failed to write notification: {}", e)))?;
    stdin.write_all(b"\n").await
        .map_err(|e| crate::error::AppError::Internal(format!("Failed to write newline: {}", e)))?;
    stdin.flush().await
        .map_err(|e| crate::error::AppError::Internal(format!("Failed to flush stdin: {}", e)))?;
    
    tracing::debug!("Sent initialized notification");
    
    // 发送 tools/list 请求
    let tools_request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list",
        "params": {}
    });
    
    let tools_str = serde_json::to_string(&tools_request)
        .map_err(|e| crate::error::AppError::Internal(format!("JSON serialization failed: {}", e)))?;
    stdin.write_all(tools_str.as_bytes()).await
        .map_err(|e| crate::error::AppError::Internal(format!("Failed to write tools request: {}", e)))?;
    stdin.write_all(b"\n").await
        .map_err(|e| crate::error::AppError::Internal(format!("Failed to write newline: {}", e)))?;
    stdin.flush().await
        .map_err(|e| crate::error::AppError::Internal(format!("Failed to flush stdin: {}", e)))?;
    
    tracing::debug!("Sent tools/list request");
    
    // 读取 tools/list 响应
    let mut tools_line = String::new();
    match tokio::time::timeout(Duration::from_secs(5), reader.read_line(&mut tools_line)).await {
        Ok(Ok(_)) => {},
        Ok(Err(e)) => return Err(crate::error::AppError::Internal(format!("Failed to read tools response: {}", e))),
        Err(_) => return Err(crate::error::AppError::Internal("Timeout waiting for tools/list response".to_string())),
    }
    
    let tools_response: serde_json::Value = serde_json::from_str(&tools_line)
        .map_err(|e| crate::error::AppError::Internal(format!("Invalid tools response: {}", e)))?;
    
    tracing::debug!("Received tools/list response");
    
    // 关闭子进程
    let _ = child.kill().await;
    
    // 提取工具列表
    let tools = tools_response.get("result")
        .and_then(|r| r.get("tools"))
        .and_then(|t| t.as_array())
        .cloned()
        .unwrap_or_default();
    
    tracing::info!("✅ Fetched {} tools from stdio MCP server", tools.len());
    
    Ok(tools)
}
