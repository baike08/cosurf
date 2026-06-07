use tauri::State;
use tracing::info;

use crate::db::settings::{CreateModelConfigRequest, ModelConfig, UpdateModelConfigRequest};
use crate::db::settings::{McpServerConfig, CreateMcpServerRequest, UpdateMcpServerRequest};
use crate::error::ErrorResponse;
use crate::state::AppState;

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Result<serde_json::Value, ErrorResponse> {
    let db = state.db.lock().map_err(|e| ErrorResponse {
        code: "LOCK_ERROR".into(),
        message: e.to_string(),
    })?;
    db.get_all_settings().map_err(|e| ErrorResponse::from(e))
}

#[tauri::command]
pub fn get_setting(state: State<'_, AppState>, key: String) -> Result<Option<String>, ErrorResponse> {
    let db = state.db.lock().map_err(|e| ErrorResponse {
        code: "LOCK_ERROR".into(),
        message: e.to_string(),
    })?;
    db.get_setting(&key).map_err(|e| ErrorResponse::from(e))
}

#[tauri::command]
pub fn set_setting(state: State<'_, AppState>, key: String, value: String) -> Result<(), ErrorResponse> {
    let db = state.db.lock().map_err(|e| ErrorResponse {
        code: "LOCK_ERROR".into(),
        message: e.to_string(),
    })?;
    db.set_setting(&key, &value).map_err(|e| ErrorResponse::from(e))
}

#[tauri::command]
pub fn list_model_configs(state: State<'_, AppState>) -> Result<Vec<ModelConfig>, ErrorResponse> {
    let db = state.db.lock().map_err(|e| ErrorResponse {
        code: "LOCK_ERROR".into(),
        message: e.to_string(),
    })?;
    db.list_model_configs().map_err(|e| ErrorResponse::from(e))
}

#[tauri::command]
pub fn get_model_config(state: State<'_, AppState>, id: String) -> Result<ModelConfig, ErrorResponse> {
    let db = state.db.lock().map_err(|e| ErrorResponse {
        code: "LOCK_ERROR".into(),
        message: e.to_string(),
    })?;
    db.get_model_config(&id).map_err(|e| ErrorResponse::from(e))
}

#[tauri::command]
pub fn get_active_model(state: State<'_, AppState>) -> Result<Option<ModelConfig>, ErrorResponse> {
    let db = state.db.lock().map_err(|e| ErrorResponse {
        code: "LOCK_ERROR".into(),
        message: e.to_string(),
    })?;
    db.get_active_model_config().map_err(|e| ErrorResponse::from(e))
}

#[tauri::command]
pub fn create_model_config(
    state: State<'_, AppState>,
    request: CreateModelConfigRequest,
) -> Result<ModelConfig, ErrorResponse> {
    let db = state.db.lock().map_err(|e| ErrorResponse {
        code: "LOCK_ERROR".into(),
        message: e.to_string(),
    })?;
    db.create_model_config(&request).map_err(|e| ErrorResponse::from(e))
}

#[tauri::command]
pub fn update_model_config(
    state: State<'_, AppState>,
    id: String,
    request: UpdateModelConfigRequest,
) -> Result<ModelConfig, ErrorResponse> {
    let db = state.db.lock().map_err(|e| ErrorResponse {
        code: "LOCK_ERROR".into(),
        message: e.to_string(),
    })?;
    db.update_model_config(&id, &request)
        .map_err(|e| ErrorResponse::from(e))
}

#[tauri::command]
pub fn set_active_model(state: State<'_, AppState>, id: String) -> Result<(), ErrorResponse> {
    let db = state.db.lock().map_err(|e| ErrorResponse {
        code: "LOCK_ERROR".into(),
        message: e.to_string(),
    })?;
    db.set_active_model(&id).map_err(|e| ErrorResponse::from(e))
}

#[tauri::command]
pub fn delete_model_config(state: State<'_, AppState>, id: String) -> Result<(), ErrorResponse> {
    let db = state.db.lock().map_err(|e| ErrorResponse {
        code: "LOCK_ERROR".into(),
        message: e.to_string(),
    })?;
    db.delete_model_config(&id).map_err(|e| ErrorResponse::from(e))
}

// ==================== Skills 配置命令 ====================

/// 获取 Skills 目录路径
#[tauri::command]
pub fn get_skills_directory(state: State<'_, AppState>) -> Result<String, ErrorResponse> {
    let db = state.db.lock().map_err(|e| ErrorResponse {
        code: "LOCK_ERROR".into(),
        message: e.to_string(),
    })?;
    db.get_skills_directory().map_err(|e| ErrorResponse::from(e))
}

/// 设置 Skills 目录路径
#[tauri::command]
pub fn set_skills_directory(
    state: State<'_, AppState>,
    directory: String,
) -> Result<(), ErrorResponse> {
    info!(directory = %directory, "Setting skills directory");
    
    // 保存配置到数据库
    let db = state.db.lock().map_err(|e| ErrorResponse {
        code: "LOCK_ERROR".into(),
        message: e.to_string(),
    })?;
    db.set_skills_directory(&directory).map_err(|e| ErrorResponse::from(e))?;
    
    // 重新初始化 SkillsManager
    let skills_dir = std::path::PathBuf::from(&directory);
    
    // 确保目录存在
    if !skills_dir.exists() {
        std::fs::create_dir_all(&skills_dir)
            .map_err(|e| ErrorResponse {
                code: "IO_ERROR".into(),
                message: format!("Failed to create directory: {}", e),
            })?;
    }
    
    // 创建新的 SkillsManager 并加载 Skills
    let mut new_manager = crate::ai::skills::SkillsManager::new(skills_dir.clone());
    match new_manager.load_skills_from_directory() {
        Ok(count) => {
            info!(count, path = ?skills_dir, "Reloaded skills from new directory");
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to load skills from new directory");
        }
    }
    
    // 替换旧的 manager
    let mut manager = state.skills_manager.lock().map_err(|e| ErrorResponse {
        code: "LOCK_ERROR".into(),
        message: e.to_string(),
    })?;
    *manager = new_manager;
    
    Ok(())
}

// ==================== IQS API Key 配置命令 ====================

/// 获取阿里云 IQS API Key
#[tauri::command]
pub fn get_iqs_api_key(state: State<'_, AppState>) -> Result<Option<String>, ErrorResponse> {
    let db = state.db.lock().map_err(|e| ErrorResponse {
        code: "LOCK_ERROR".into(),
        message: e.to_string(),
    })?;
    let result = db.get_iqs_api_key().map_err(|e| ErrorResponse::from(e));
    tracing::info!("get_iqs_api_key called, result: {:?}", result);
    result
}

/// 设置阿里云 IQS API Key
#[tauri::command]
pub fn set_iqs_api_key(
    state: State<'_, AppState>,
    api_key: String,
) -> Result<(), ErrorResponse> {
    tracing::info!("set_iqs_api_key called with key length: {}", api_key.len());
    let db = state.db.lock().map_err(|e| ErrorResponse {
        code: "LOCK_ERROR".into(),
        message: e.to_string(),
    })?;
    let result = db.set_iqs_api_key(&api_key).map_err(|e| ErrorResponse::from(e));
    tracing::info!("set_iqs_api_key result: {:?}", result);
    result
}

// ==================== MCP Server 配置命令 ====================

/// 列出所有 MCP Servers
#[tauri::command]
pub fn list_mcp_servers(state: State<'_, AppState>) -> Result<Vec<McpServerConfig>, ErrorResponse> {
    let db = state.db.lock().map_err(|e| ErrorResponse {
        code: "LOCK_ERROR".into(),
        message: e.to_string(),
    })?;
    db.list_mcp_servers().map_err(|e| ErrorResponse::from(e))
}

/// 获取单个 MCP Server
#[tauri::command]
pub fn get_mcp_server(
    state: State<'_, AppState>,
    id: String,
) -> Result<McpServerConfig, ErrorResponse> {
    let db = state.db.lock().map_err(|e| ErrorResponse {
        code: "LOCK_ERROR".into(),
        message: e.to_string(),
    })?;
    db.get_mcp_server(&id).map_err(|e| ErrorResponse::from(e))
}

/// 创建 MCP Server
#[tauri::command]
pub fn create_mcp_server(
    state: State<'_, AppState>,
    request: CreateMcpServerRequest,
) -> Result<McpServerConfig, ErrorResponse> {
    let db = state.db.lock().map_err(|e| ErrorResponse {
        code: "LOCK_ERROR".into(),
        message: e.to_string(),
    })?;
    db.create_mcp_server(&request).map_err(|e| ErrorResponse::from(e))
}

/// 更新 MCP Server
#[tauri::command]
pub fn update_mcp_server(
    state: State<'_, AppState>,
    id: String,
    request: UpdateMcpServerRequest,
) -> Result<McpServerConfig, ErrorResponse> {
    let db = state.db.lock().map_err(|e| ErrorResponse {
        code: "LOCK_ERROR".into(),
        message: e.to_string(),
    })?;
    db.update_mcp_server(&id, &request).map_err(|e| ErrorResponse::from(e))
}

/// 删除 MCP Server
#[tauri::command]
pub fn delete_mcp_server(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), ErrorResponse> {
    let db = state.db.lock().map_err(|e| ErrorResponse {
        code: "LOCK_ERROR".into(),
        message: e.to_string(),
    })?;
    db.delete_mcp_server(&id).map_err(|e| ErrorResponse::from(e))
}

use crate::ai::skills_executors::command_utils::{build_enhanced_path, resolve_command};

/// 测试 MCP Server 连接并获取可用工具列表
#[tauri::command]
pub async fn test_mcp_server(
    server_type: String,
    server_url: Option<String>,
    command: Option<String>,
    args: Option<Vec<String>>,
    env: Option<std::collections::HashMap<String, String>>,
    api_key: Option<String>,
    headers: Option<serde_json::Map<String, serde_json::Value>>,
) -> Result<serde_json::Value, ErrorResponse> {
    if server_type == "stdio" {
        // stdio 模式：启动子进程，通过 stdin/stdout 通信
        test_mcp_stdio_server(command, args, env).await
    } else {
        // http / streamableHttp / sse 模式
        let url = server_url.unwrap_or_default();
        if url.is_empty() {
            return Err(ErrorResponse {
                code: "INVALID_URL".into(),
                message: "Server URL is required for HTTP type".into(),
            });
        }
        use crate::ai::skills_executors::mcp::{McpClient, McpTransport};
        
        // 确定传输模式
        let transport = match server_type.to_lowercase().as_str() {
            "sse" => McpTransport::Sse,
            _ => McpTransport::StreamableHttp, // http, streamableHttp 都用 StreamableHttp
        };
        
        let mut client = McpClient::new(url, transport, api_key, headers);
        client.initialize().await.map_err(|e| ErrorResponse {
            code: "INIT_FAILED".into(),
            message: format!("Failed to initialize MCP connection: {}", e),
        })?;
        let tools = client.list_tools().await.map_err(|e| ErrorResponse {
            code: "LIST_TOOLS_FAILED".into(),
            message: format!("Failed to list tools: {}", e),
        })?;
        Ok(tools)
    }
}

/// 通过 stdio 测试 MCP Server
async fn test_mcp_stdio_server(
    command: Option<String>,
    args: Option<Vec<String>>,
    env: Option<std::collections::HashMap<String, String>>,
) -> Result<serde_json::Value, ErrorResponse> {
    use tokio::process::Command;
    use tokio::io::{AsyncWriteExt, AsyncBufReadExt, BufReader};
    use std::time::Duration;
    
    let cmd = command.ok_or_else(|| ErrorResponse {
        code: "INVALID_CONFIG".into(),
        message: "Command is required for stdio type".into(),
    })?;
    
    let cmd_args = args.unwrap_or_default();
    
    // 构建增强的 PATH（包含常见的 Node.js 安装位置）
    let enhanced_path = build_enhanced_path();
    
    // Windows 上 npx/npm/pnpm 是 .cmd 文件，需要通过 cmd /c 执行
    let (final_cmd, final_args) = resolve_command(&cmd, &cmd_args);
    
    let mut child_cmd = Command::new(&final_cmd);
    child_cmd.args(&final_args);
    
    // 设置增强的 PATH 环境变量
    child_cmd.env("PATH", &enhanced_path);
    
    // 设置用户指定的环境变量
    if let Some(ref env_vars) = env {
        for (key, value) in env_vars {
            child_cmd.env(key, value);
        }
    }
    
    child_cmd.stdin(std::process::Stdio::piped());
    child_cmd.stdout(std::process::Stdio::piped());
    child_cmd.stderr(std::process::Stdio::piped());
    
    let mut child = child_cmd.spawn().map_err(|e| ErrorResponse {
        code: "SPAWN_FAILED".into(),
        message: format!("Failed to start process '{}': {}", cmd, e),
    })?;
    
    let mut stdin = child.stdin.take().ok_or_else(|| ErrorResponse {
        code: "STDIN_ERROR".into(),
        message: "Failed to open stdin".into(),
    })?;
    
    let stdout = child.stdout.take().ok_or_else(|| ErrorResponse {
        code: "STDOUT_ERROR".into(),
        message: "Failed to open stdout".into(),
    })?;
    let mut reader = BufReader::new(stdout);
    
    // 发送 initialize 请求
    let init_request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": { "roots": { "listChanged": false } },
            "clientInfo": { "name": "CoSurf", "version": env!("CARGO_PKG_VERSION") }
        }
    });
    
    let init_msg = format!("{}\n", serde_json::to_string(&init_request).unwrap());
    stdin.write_all(init_msg.as_bytes()).await.map_err(|e| ErrorResponse {
        code: "WRITE_ERROR".into(),
        message: format!("Failed to write to stdin: {}", e),
    })?;
    stdin.flush().await.map_err(|e| ErrorResponse {
        code: "WRITE_ERROR".into(),
        message: format!("Failed to flush stdin: {}", e),
    })?;
    
    // 读取 initialize 响应（带超时）
    let mut line = String::new();
    let read_result = tokio::time::timeout(Duration::from_secs(15), reader.read_line(&mut line)).await;
    
    match read_result {
        Ok(Ok(0)) | Ok(Err(_)) => {
            // 进程退出或读取失败
            let _ = child.kill().await;
            return Err(ErrorResponse {
                code: "INIT_FAILED".into(),
                message: "Process exited or did not respond to initialize".into(),
            });
        }
        Err(_) => {
            let _ = child.kill().await;
            return Err(ErrorResponse {
                code: "TIMEOUT".into(),
                message: "MCP server did not respond within 15 seconds".into(),
            });
        }
        Ok(Ok(_)) => { /* 成功读取 */ }
    }
    
    // 验证 initialize 响应
    let init_response: serde_json::Value = serde_json::from_str(line.trim()).map_err(|_| ErrorResponse {
        code: "PARSE_ERROR".into(),
        message: format!("Invalid JSON response - got: {}", line.trim()),
    })?;
    
    if let Some(error) = init_response.get("error") {
        let _ = child.kill().await;
        return Err(ErrorResponse {
            code: "INIT_ERROR".into(),
            message: format!("MCP error: {}", error),
        });
    }
    
    // 发送 initialized 通知
    let initialized_notification = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized"
    });
    let notif_msg = format!("{}\n", serde_json::to_string(&initialized_notification).unwrap());
    stdin.write_all(notif_msg.as_bytes()).await.ok();
    stdin.flush().await.ok();
    
    // 发送 tools/list 请求
    let tools_request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list",
        "params": {}
    });
    let tools_msg = format!("{}\n", serde_json::to_string(&tools_request).unwrap());
    stdin.write_all(tools_msg.as_bytes()).await.map_err(|e| ErrorResponse {
        code: "WRITE_ERROR".into(),
        message: format!("Failed to write tools request: {}", e),
    })?;
    stdin.flush().await.ok();
    
    // 读取 tools/list 响应
    let mut tools_line = String::new();
    let tools_result = tokio::time::timeout(Duration::from_secs(15), reader.read_line(&mut tools_line)).await;
    
    let _ = child.kill().await;
    
    match tools_result {
        Ok(Ok(n)) if n > 0 => {
            let tools_response: serde_json::Value = serde_json::from_str(tools_line.trim()).map_err(|e| ErrorResponse {
                code: "PARSE_ERROR".into(),
                message: format!("Invalid tools response: {} - got: {}", e, tools_line.trim()),
            })?;
            
            if let Some(result) = tools_response.get("result") {
                Ok(result.clone())
            } else if let Some(error) = tools_response.get("error") {
                Err(ErrorResponse {
                    code: "LIST_TOOLS_FAILED".into(),
                    message: format!("MCP error: {}", error),
                })
            } else {
                Err(ErrorResponse {
                    code: "LIST_TOOLS_FAILED".into(),
                    message: format!("No result in response: {}", tools_line.trim()),
                })
            }
        }
        Ok(Err(e)) => Err(ErrorResponse {
            code: "READ_ERROR".into(),
            message: format!("Failed to read tools response: {}", e),
        }),
        Err(_) => Err(ErrorResponse {
            code: "TIMEOUT".into(),
            message: "MCP server did not respond to tools/list within 15 seconds".into(),
        }),
        _ => Err(ErrorResponse {
            code: "READ_ERROR".into(),
            message: "Process closed stdout".into(),
        }),
    }
}

/// 从 JSON 配置批量导入 MCP Servers
/// 
/// 支持开源标准 MCP JSON 格式：
/// ```json
/// {
///   "mcpServers": {
///     "server-name": {
///       "type": "streamableHttp",   // 或 "stdio", "sse", "http"
///       "url": "https://...",
///       "headers": { "X-API-Key": "..." },
///       "command": "npx",           // stdio 模式
///       "args": ["-y", "..."],
///       "env": { "KEY": "val" },
///       "disabled": false,
///       "timeout": 30
///     }
///   }
/// }
/// ```
#[tauri::command]
pub fn import_mcp_servers_from_json(
    state: State<'_, AppState>,
    json_content: String,
) -> Result<Vec<McpServerConfig>, ErrorResponse> {
    let db = state.db.lock().map_err(|e| ErrorResponse {
        code: "LOCK_ERROR".into(),
        message: e.to_string(),
    })?;
    
    // 解析 JSON
    let parsed: serde_json::Value = serde_json::from_str(&json_content)
        .map_err(|e| ErrorResponse {
            code: "PARSE_ERROR".into(),
            message: format!("Failed to parse JSON: {}", e),
        })?;
    
    let mcp_servers = parsed.get("mcpServers")
        .and_then(|v| v.as_object())
        .ok_or_else(|| ErrorResponse {
            code: "INVALID_FORMAT".into(),
            message: "JSON must contain 'mcpServers' object".to_string(),
        })?;
    
    let mut created_servers = Vec::new();
    
    for (name, server_config) in mcp_servers {
        // 解析 type（支持 streamableHttp, sse, http, stdio）
        let server_type = server_config.get("type")
            .and_then(|v| v.as_str())
            .map(|s| crate::db::settings::parse_mcp_type_str(s))
            .unwrap_or_else(|| {
                // 智能推断：有 url 则默认 streamableHttp，有 command 则默认 stdio
                if server_config.get("url").is_some() {
                    crate::db::settings::McpServerType::StreamableHttp
                } else {
                    crate::db::settings::McpServerType::Stdio
                }
            });
        
        // 解析 url
        let url = server_config.get("url")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        
        // 解析 headers（如 X-API-Key）
        let headers = server_config.get("headers")
            .and_then(|v| v.as_object())
            .cloned();
        
        // 解析 command
        let command = server_config.get("command")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        
        // 解析 args
        let args = server_config.get("args")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect());
        
        // 解析 cwd
        let cwd = server_config.get("cwd")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        
        // 解析 env
        let env = server_config.get("env")
            .and_then(|v| v.as_object())
            .cloned();
        
        // 解析 disabled
        let disabled = server_config.get("disabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        
        // 解析 timeout
        let timeout = server_config.get("timeout")
            .and_then(|v| v.as_u64());
        
        let req = CreateMcpServerRequest {
            name: name.clone(),
            server_type: Some(server_type),
            url,
            headers,
            command,
            args,
            cwd,
            env,
            disabled: Some(disabled),
            timeout,
            enabled: Some(!disabled),
        };
        
        match db.create_mcp_server(&req) {
            Ok(server) => {
                tracing::info!("Imported MCP server: {} (type: {:?})", name, server.server_type);
                created_servers.push(server);
            }
            Err(e) => {
                tracing::warn!("Failed to create MCP server {}: {}", name, e);
            }
        }
    }
    
    Ok(created_servers)
}
