//! AI 工具系统 — 定义、Schema 生成、执行调度
//!
//! 从 src-tauri/src/ai/tools.rs 迁移。

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
    SummarizePage,
    WebAgent,
    OpenUrl,
    Translate,
    ExportMarkdown,
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
            Self::RunCommand => "在系统终端执行 shell 命令（支持 Windows cmd / Linux/macOS sh），捕获 stdout/stderr 返回结果",
        }
    }

    pub fn parameters(&self) -> serde_json::Value {
        match self {
            Self::SummarizePage => serde_json::json!({
                "type": "object",
                "properties": {
                    "max_length": {
                        "type": "integer",
                        "description": "最大摘要长度（字符数），默认 500"
                    }
                }
            }),
            Self::WebAgent => serde_json::json!({
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": ["click", "fill", "select", "scroll", "wait"],
                        "description": "要执行的操作类型"
                    },
                    "selector": { "type": "string", "description": "CSS 选择器" },
                    "value": { "type": "string", "description": "填写的值（仅 fill 操作需要）" }
                },
                "required": ["action", "selector"]
            }),
            Self::OpenUrl => serde_json::json!({
                "type": "object",
                "properties": {
                    "url": { "type": "string", "description": "要打开的网页URL，必须以 http:// 或 https:// 开头" }
                },
                "required": ["url"]
            }),
            Self::Translate => serde_json::json!({
                "type": "object",
                "properties": {
                    "target_language": { "type": "string", "description": "目标语言，如 'zh', 'en', 'ja'" }
                },
                "required": ["target_language"]
            }),
            Self::ExportMarkdown => serde_json::json!({
                "type": "object",
                "properties": {}
            }),
            Self::RunCommand => serde_json::json!({
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
            }),
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

/// 获取所有可用内置工具的 schema
pub fn get_builtin_tools_schemas() -> Vec<serde_json::Value> {
    vec![
        BuiltInTool::SummarizePage.to_openai_schema(),
        BuiltInTool::WebAgent.to_openai_schema(),
        BuiltInTool::OpenUrl.to_openai_schema(),
        BuiltInTool::Translate.to_openai_schema(),
        BuiltInTool::ExportMarkdown.to_openai_schema(),
        BuiltInTool::RunCommand.to_openai_schema(),
    ]
}

/// 获取所有可用工具的 schema（内置 + Skills + MCP）
pub fn get_all_tools_schemas(skills_schemas: Vec<serde_json::Value>) -> Vec<serde_json::Value> {
    let mut schemas = get_builtin_tools_schemas();
    schemas.extend(skills_schemas);
    // MCP 工具 schema 在运行时动态添加
    schemas
}

/// 转换工具调用格式：将嵌套的 function.name/function.arguments 提升到顶层
pub fn normalize_tool_call_format(tc: &serde_json::Value) -> serde_json::Value {
    if let Some(obj) = tc.as_object() {
        let mut normalized = serde_json::Map::new();

        if let Some(id) = obj.get("id") {
            normalized.insert("id".to_string(), id.clone());
        }

        if let Some(function) = obj.get("function") {
            if let Some(func_obj) = function.as_object() {
                if let Some(name) = func_obj.get("name") {
                    normalized.insert("name".to_string(), name.clone());
                }
                if let Some(args_value) = func_obj.get("arguments") {
                    if let Some(args_str) = args_value.as_str() {
                        match serde_json::from_str::<serde_json::Value>(args_str) {
                            Ok(parsed_args) => {
                                normalized.insert("arguments".to_string(), parsed_args);
                            }
                            Err(_) => {
                                normalized.insert("arguments".to_string(), args_value.clone());
                            }
                        }
                    } else {
                        normalized.insert("arguments".to_string(), args_value.clone());
                    }
                }
            }
        }

        serde_json::Value::Object(normalized)
    } else {
        tc.clone()
    }
}

/// 执行内置工具
pub async fn execute_builtin_tool(tool_call: &ToolCall) -> crate::error::AppResult<ToolResult> {
    let output = match tool_call.name.as_str() {
        "run_command" => {
            let cmd = tool_call.arguments.get("command")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let working_dir = tool_call.arguments.get("working_dir")
                .and_then(|v| v.as_str());
            let timeout = tool_call.arguments.get("timeout")
                .and_then(|v| v.as_u64())
                .unwrap_or(30);

            execute_shell_command(cmd, working_dir, timeout).await
        }
        "open_url" | "web_agent" | "summarize_page" | "translate" | "export_markdown" => {
            // 这些工具需要 Electron 主进程桥接才能操作浏览器界面
            // 返回特殊标记，让上层知道需要等待 Electron 执行结果
            Ok(format!("__ELECTRON_BRIDGE_REQUIRED__:{}", serde_json::to_string(tool_call)?))
        }
        _ => {
            // 检查是否是 Skill 工具（名称以 skill_ 开头）
            tracing::info!("🔧 execute_builtin_tool: tool_name={}", tool_call.name);
            if tool_call.name.starts_with("skill_") {
                let skill_id = &tool_call.name[6..]; // 去掉 "skill_" 前缀
                tracing::info!("📖 Executing skill: {}", skill_id);
                execute_skill_tool(skill_id, tool_call).await
            } else if tool_call.name.starts_with("mcp_") {
                // 检查是否是 MCP 工具（名称以 mcp_ 开头）
                tracing::info!("🌐 Executing MCP tool: {}", tool_call.name);
                execute_mcp_tool(tool_call).await
            } else {
                Ok(format!("Tool '{}' requires Electron main process bridge for execution", tool_call.name))
            }
        }
    };

    match output {
        Ok(result) => Ok(ToolResult {
            tool_call_id: tool_call.id.clone(),
            output: result,
            success: true,
        }),
        Err(e) => Ok(ToolResult {
            tool_call_id: tool_call.id.clone(),
            output: format!("工具执行失败: {}", e),
            success: false,
        }),
    }
}

/// 执行 shell 命令
async fn execute_shell_command(
    command: &str,
    working_dir: Option<&str>,
    timeout_secs: u64,
) -> crate::error::AppResult<String> {
    use std::process::Stdio;
    use tokio::process::Command;

    let mut cmd = if cfg!(target_os = "windows") {
        let mut c = Command::new("cmd");
        c.args(["/C", command]);
        c
    } else {
        let mut c = Command::new("sh");
        c.args(["-c", command]);
        c
    };

    if let Some(dir) = working_dir {
        cmd.current_dir(dir);
    }

    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let timeout = std::time::Duration::from_secs(timeout_secs.min(120));
    let result = tokio::time::timeout(timeout, cmd.output()).await;

    match result {
        Ok(Ok(output)) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            let mut result = String::new();
            if !stdout.is_empty() {
                result.push_str(&stdout);
            }
            if !stderr.is_empty() {
                if !result.is_empty() {
                    result.push_str("\n--- stderr ---\n");
                }
                result.push_str(&stderr);
            }
            if result.is_empty() {
                result = format!("[exit code: {}]", output.status.code().unwrap_or(-1));
            }
            Ok(result)
        }
        Ok(Err(e)) => Err(crate::error::AppError::Internal(format!("Command execution error: {}", e))),
        Err(_) => Err(crate::error::AppError::Internal(format!("Command timed out after {}s", timeout_secs))),
    }
}

/// 执行 Skill 工具（渐进式加载）
/// 
/// 模型调用 skill_{id} 后，读取完整 SKILL.md 内容作为 tool result 返回
/// Agent Loop 将继续根据 SKILL.md 内容决策下一步操作（MCP tools / 内置工具 / 脚本）
async fn execute_skill_tool(
    skill_id: &str,
    _tool_call: &ToolCall,
) -> crate::error::AppResult<String> {
    use crate::ai::agent;
    
    tracing::info!("📖 Lazy loading Skill: {}", skill_id);
    
    // 从 SkillsManager 懒加载完整 SKILL.md 内容
    match agent::skills_get_content_internal(skill_id) {
        Ok(content) => {
            tracing::info!(skill_id = %skill_id, bytes = content.len(), "✅ Loaded skill content, returning to Agent Loop");
            // 将 SKILL.md 内容作为 tool result 返回，让模型解读并决定下一步操作
            let output = format!(
                "# SKILL LOADED: {}\n\n{}\n\n---\nNow follow the instructions in this SKILL to complete the task. \nUse the available tools (MCP tools, built-in tools, or scripts) as specified in the SKILL.",
                skill_id,
                content
            );
            Ok(output)
        }
        Err(e) => {
            tracing::error!("Failed to load skill {}: {}", skill_id, e);
            Ok(format!("Failed to load skill '{}': {}", skill_id, e))
        }
    }
}

/// 执行 MCP 工具
/// 
/// 模型调用 mcp_{server}_{tool} 后，通过 MCP Client 调用对应的 MCP Server 工具
async fn execute_mcp_tool(
    tool_call: &ToolCall,
) -> crate::error::AppResult<String> {
    use crate::ai::agent;
    
    tracing::info!("🌐 Executing MCP tool: {}, arguments: {}", 
        tool_call.name, serde_json::to_string(&tool_call.arguments).unwrap_or_default());
    
    // 通过 MCP Manager 执行工具
    match agent::execute_mcp_tool(&tool_call.name, &tool_call.arguments).await {
        Ok(result) => {
            tracing::info!("✅ MCP tool executed successfully, result length: {}", 
                serde_json::to_string(&result).unwrap_or_default().len());
            Ok(serde_json::to_string_pretty(&result).unwrap_or_else(|_| result.to_string()))
        }
        Err(e) => {
            tracing::error!("❌ MCP tool execution failed: {}", e);
            Ok(format!("MCP tool execution failed: {}", e))
        }
    }
}
