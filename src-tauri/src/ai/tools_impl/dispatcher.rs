/// 工具调度器
/// 
/// 根据工具名称路由到对应的实现

use tauri::AppHandle;
use tracing::{info, error};

use crate::error::{AppError, AppResult};
use crate::ai::tools::{ToolCall, ToolResult};

/// 执行工具调用
/// 
/// 根据工具名称分发到对应的实现模块
pub async fn execute(app: &AppHandle, tool_call: &ToolCall) -> AppResult<ToolResult> {
    let tool_name = tool_call.name.as_str();
    
    info!("⚙️  ========== EXECUTE TOOL ==========");
    info!("Tool name: {}", tool_name);
    info!("Tool ID: {}", tool_call.id);
    info!("Arguments: {:?}", tool_call.arguments);
    
    // 检查是否为 Skill 工具（格式：skill_{id}）
    if tool_name.starts_with("skill_") {
        let skill_id = &tool_name[6..]; // 去掉 "skill_" 前缀
        return execute_skill_tool(app, skill_id, tool_call).await;
    }
    
    // 检查是否为 MCP 工具（格式：mcp_{server}_{tool}，通过 registry 查找）
    if tool_name.starts_with("mcp_") {
        return execute_mcp_tool(app, tool_call).await;
    }
    
    // 内置工具调度
    match tool_name {
        "open_url" => {
            super::open_url::execute(app, tool_call).await
        }
        "web_search" => {
            super::web_search::execute(app, tool_call).await
        }
        "summarize_page" => {
            super::summarize_page::execute(app, tool_call).await
        }
        "web_agent" => {
            super::web_agent::execute(app, tool_call).await
        }
        "run_command" => {
            super::run_command::execute(app, tool_call).await
        }
        _ => {
            error!("❌ Unknown tool: {}", tool_name);
            Err(AppError::Internal(format!("Unknown tool: {}", tool_name)))
        }
    }
}

/// 执行 Skill 工具（渐进式加载）
/// 
/// 模型调用 skill_{id} 后，读取完整 SKILL.md 内容作为 tool result 返回
/// Agent Loop 将继续根据 SKILL.md 内容决策下一步操作（MCP tools / 内置工具 / 脚本）
async fn execute_skill_tool(
    app: &AppHandle,
    skill_id: &str,
    tool_call: &ToolCall,
) -> AppResult<ToolResult> {
    use crate::state::AppState;
    use tauri::Manager;

    info!("📖 Lazy loading Skill: {}", skill_id);

    let state = app.state::<AppState>();

    // 获取 Skills 目录（在异步操作之前完成所有数据库访问）
    let skills_dir_str = {
        let db = state.db.lock().map_err(|e| {
            AppError::Internal(format!("Failed to lock database: {}", e))
        })?;
        db.get_skills_directory()?
    }; // db 在这里被 drop

    let skills_dir = std::path::PathBuf::from(skills_dir_str);
    let mut skills_manager = crate::ai::skills::SkillsManager::new(skills_dir);

    // 加载所有 Skills 元数据
    if let Err(e) = skills_manager.load_skills_from_directory() {
        error!("Failed to load skills: {}", e);
        return Ok(ToolResult {
            tool_call_id: tool_call.id.clone(),
            output: format!("Failed to load skills: {}", e),
            success: false,
        });
    }

    // 懒加载完整 SKILL.md 内容
    match skills_manager.load_skill_content(skill_id) {
        Ok(content) => {
            info!(skill_id = %skill_id, bytes = content.len(), "✅ Loaded skill content, returning to Agent Loop");
            // 将 SKILL.md 内容作为 tool result 返回，让模型解读并决定下一步操作
            let output = format!(
                "# SKILL LOADED: {}\n\n{}\n\n---\nNow follow the instructions in this SKILL to complete the task. \
                 Use the available tools (web_search, open_url, web_agent, summarize_page, run_command, or MCP tools) as needed.",
                skill_id, content
            );
            Ok(ToolResult {
                tool_call_id: tool_call.id.clone(),
                output,
                success: true,
            })
        }
        Err(e) => {
            error!("❌ Failed to load skill content: {}", e);
            Ok(ToolResult {
                tool_call_id: tool_call.id.clone(),
                output: format!("Failed to load skill content: {}", e),
                success: false,
            })
        }
    }
}

/// 执行 MCP 工具（直接调用模式）
/// 
/// MCP 工具已在 Agent Loop 中注册为独立的 function，
/// 通过 mcp_tool_registry 查找对应的 server 和原始工具名，直接调用。
async fn execute_mcp_tool(
    app: &AppHandle,
    tool_call: &ToolCall,
) -> AppResult<ToolResult> {
    use crate::state::AppState;
    use tauri::Manager;
    
    let state = app.state::<AppState>();
    
    // 从 registry 查找 server_name 和 original_tool_name
    let (server_name, mcp_tool_name) = {
        let registry = state.mcp_tool_registry.lock().map_err(|e| {
            AppError::Internal(format!("Failed to lock MCP registry: {}", e))
        })?;
        match registry.get(tool_call.name.as_str()) {
            Some((server, tool)) => (server.clone(), tool.clone()),
            None => {
                error!("MCP tool not found in registry: {}", tool_call.name);
                return Ok(ToolResult {
                    tool_call_id: tool_call.id.clone(),
                    output: format!("MCP tool '{}' not found. The MCP server may be disconnected.", tool_call.name),
                    success: false,
                });
            }
        }
    };
    
    info!("🔧 Executing MCP tool: server={}, tool={}", server_name, mcp_tool_name);
    
    // 获取 MCP Server 配置
    let server = {
        let db = state.db.lock().map_err(|e| {
            AppError::Internal(format!("Failed to lock database: {}", e))
        })?;
        
        let servers = match db.list_mcp_servers() {
            Ok(servers) => servers,
            Err(e) => {
                error!("Failed to list MCP servers: {}", e);
                return Ok(ToolResult {
                    tool_call_id: tool_call.id.clone(),
                    output: format!("Failed to list MCP servers: {}", e),
                    success: false,
                });
            }
        };
        
        match servers.iter().find(|s| s.name == server_name) {
            Some(s) => s.clone(),
            None => {
                error!("MCP server not found: {}", server_name);
                return Ok(ToolResult {
                    tool_call_id: tool_call.id.clone(),
                    output: format!("MCP server '{}' not found", server_name),
                    success: false,
                });
            }
        }
    };
    
    // 直接调用 MCP 工具，arguments 就是 LLM 传入的参数
    match call_mcp_server(&server, &mcp_tool_name, &tool_call.arguments).await {
        Ok(output) => {
            info!("✅ MCP tool {}::{} executed successfully", server_name, mcp_tool_name);
            Ok(ToolResult {
                tool_call_id: tool_call.id.clone(),
                output,
                success: true,
            })
        }
        Err(e) => {
            error!("❌ MCP tool {}::{} failed: {}", server_name, mcp_tool_name, e);
            Ok(ToolResult {
                tool_call_id: tool_call.id.clone(),
                output: format!("MCP tool execution failed: {}", e),
                success: false,
            })
        }
    }
}

/// 调用 MCP Server 的工具
async fn call_mcp_server(
    server: &crate::db::settings::McpServerConfig,
    tool_name: &str,
    arguments: &serde_json::Value,
) -> AppResult<String> {
    use crate::ai::skills_executors::mcp::{McpClient, McpTransport};
    
    // 确定传输模式
    let transport = match server.server_type {
        crate::db::settings::McpServerType::Http
        | crate::db::settings::McpServerType::StreamableHttp => McpTransport::StreamableHttp,
        crate::db::settings::McpServerType::Sse => McpTransport::Sse,
        crate::db::settings::McpServerType::Stdio => {
            return Err(AppError::Internal(
                "Stdio mode MCP client not yet implemented in Agent Loop".to_string()
            ));
        }
    };
    
    let url = server.url.as_ref()
        .ok_or_else(|| AppError::Internal("HTTP/StreamableHttp/SSE server missing URL".to_string()))?;
    
    // 创建 MCP 客户端
    let mut client = McpClient::new(url.clone(), transport, None, server.headers.clone());
    
    // 初始化连接
    client.initialize().await?;
    
    // 调用工具
    client.call_tool(tool_name, arguments).await
}
