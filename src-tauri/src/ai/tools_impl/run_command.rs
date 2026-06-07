/// run_command 工具实现
///
/// 在系统终端中执行 shell 命令，捕获 stdout/stderr 返回给 Agent Loop。
/// 安全机制：
/// - 超时限制（默认 30 秒）
/// - 输出截断（最大 8000 字符）
/// - 禁止危险命令（rm -rf /、格式化磁盘等）

use tauri::AppHandle;
use tracing::{info, error, warn};
use std::time::Duration;

use crate::error::{AppError, AppResult};
use crate::ai::tools::{ToolCall, ToolResult};

/// 最大输出长度（字符）
const MAX_OUTPUT_LEN: usize = 8000;

/// 命令超时时间（秒）
const COMMAND_TIMEOUT_SECS: u64 = 30;

/// 危险命令黑名单（不区分大小写匹配前缀）
const BLOCKED_COMMANDS: &[&str] = &[
    "rm -rf /",
    "rm -rf /*",
    "del /s /q c:\\",
    "format c:",
    "format d:",
    "mkfs",
    "dd if=",
    ":(){ :|:& };:",  // fork bomb
];

/// 执行 run_command 工具
pub async fn execute(_app: &AppHandle, tool_call: &ToolCall) -> AppResult<ToolResult> {
    info!("🖥️  Executing run_command tool");

    let args = &tool_call.arguments;

    // 提取参数
    let command = args.get("command")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Internal("Missing 'command' parameter".into()))?;

    let working_dir = args.get("working_dir")
        .and_then(|v| v.as_str());

    let timeout_secs = args.get("timeout")
        .and_then(|v| v.as_u64())
        .unwrap_or(COMMAND_TIMEOUT_SECS);

    info!("  command: {}", command);
    info!("  working_dir: {:?}", working_dir);
    info!("  timeout: {}s", timeout_secs);

    // 安全检查：拦截危险命令
    let lower_cmd = command.to_lowercase();
    for blocked in BLOCKED_COMMANDS {
        if lower_cmd.contains(blocked) {
            warn!("🚫 Blocked dangerous command: {}", command);
            return Ok(ToolResult {
                tool_call_id: tool_call.id.clone(),
                output: format!("⛔ 安全拦截：命令包含危险操作 '{}'\n被拦截的命令: {}", blocked, command),
                success: false,
            });
        }
    }

    // 使用 tokio::process::Command 执行命令
    let mut cmd = if cfg!(target_os = "windows") {
        let mut c = tokio::process::Command::new("cmd");
        c.args(["/C", command]);
        c
    } else {
        let mut c = tokio::process::Command::new("sh");
        c.args(["-c", command]);
        c
    };

    // 设置工作目录
    if let Some(dir) = working_dir {
        cmd.current_dir(dir);
    }

    // 隐藏窗口（Windows 上不弹出 cmd 窗口）
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    info!("⏳ Running command with {}s timeout...", timeout_secs);

    // 执行命令并设置超时
    let output_result = tokio::time::timeout(
        Duration::from_secs(timeout_secs),
        cmd.output()
    ).await;

    match output_result {
        Ok(Ok(output)) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let exit_code = output.status.code().unwrap_or(-1);

            info!("✅ Command finished with exit code: {}", exit_code);
            info!("  stdout_len: {}, stderr_len: {}", stdout.len(), stderr.len());

            // 构建输出
            let mut result_parts = Vec::new();

            if !stdout.is_empty() {
                let truncated_stdout = truncate_output(&stdout, MAX_OUTPUT_LEN);
                result_parts.push(format!("## stdout\n{}", truncated_stdout));
            }

            if !stderr.is_empty() {
                let truncated_stderr = truncate_output(&stderr, MAX_OUTPUT_LEN / 2);
                result_parts.push(format!("## stderr\n{}", truncated_stderr));
            }

            result_parts.push(format!("## exit_code\n{}", exit_code));

            let combined_output = result_parts.join("\n\n");

            Ok(ToolResult {
                tool_call_id: tool_call.id.clone(),
                output: combined_output,
                success: exit_code == 0,
            })
        }
        Ok(Err(e)) => {
            error!("❌ Failed to execute command: {}", e);
            Ok(ToolResult {
                tool_call_id: tool_call.id.clone(),
                output: format!("命令执行失败: {}\n命令: {}", e, command),
                success: false,
            })
        }
        Err(_) => {
            error!("⏰ Command timed out after {}s", timeout_secs);
            Ok(ToolResult {
                tool_call_id: tool_call.id.clone(),
                output: format!("命令执行超时（{}秒），已强制终止。\n命令: {}", timeout_secs, command),
                success: false,
            })
        }
    }
}

/// 截断输出内容，避免超长文本
fn truncate_output(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        text.to_string()
    } else {
        let truncated: String = text.chars().take(max_len).collect();
        format!("{}... (truncated, total {} chars)", truncated, text.len())
    }
}
