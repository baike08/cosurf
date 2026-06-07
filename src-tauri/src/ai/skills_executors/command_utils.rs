/// 命令执行工具函数
/// 提供跨平台的命令解析和 PATH 增强

/// 构建增强的 PATH 环境变量，包含常见的运行时安装位置
pub fn build_enhanced_path() -> String {
    let mut paths: Vec<String> = Vec::new();
    
    // 当前系统 PATH
    if let Ok(path) = std::env::var("PATH") {
        paths.push(path);
    }
    
    // Windows 常见安装位置
    #[cfg(target_os = "windows")]
    {
        let home = std::env::var("USERPROFILE").unwrap_or_default();
        let appdata = std::env::var("APPDATA").unwrap_or_default();
        let local_appdata = std::env::var("LOCALAPPDATA").unwrap_or_default();
        let program_files = std::env::var("ProgramFiles").unwrap_or_else(|_| "C:\\Program Files".to_string());
        
        let candidates = [
            // 项目本地 Node.js
            format!("{}\\.tools\\node-v20.18.0-win-x64", env!("CARGO_MANIFEST_DIR")),
            // nvm-windows
            format!("{}\\nvm\\current", appdata),
            format!("{}\\AppData\\Roaming\\nvm\\current", home),
            // 全局安装
            format!("{}\\nodejs", program_files),
            // npm global
            format!("{}\\npm", appdata),
            // fnm (Fast Node Manager)
            format!("{}\\fnm_multishells", local_appdata),
            // volta
            format!("{}\\volta\\bin", local_appdata),
            // nvm for Windows
            format!("{}\\nvm", appdata),
            // pnpm
            format!("{}\\pnpm", local_appdata),
            // Python 常见位置
            format!("{}\\Python", local_appdata),
            format!("{}\\Python311", local_appdata),
            format!("{}\\Python312", local_appdata),
        ];
        
        for p in &candidates {
            if std::path::Path::new(p).exists() {
                paths.push(p.clone());
            }
        }
    }
    
    // macOS/Linux 常见位置
    #[cfg(not(target_os = "windows"))]
    {
        let home = std::env::var("HOME").unwrap_or_default();
        let candidates = [
            format!("{}/.nvm/versions/node", home),
            format!("{}/.volta/bin", home),
            format!("{}/.fnm/node-versions", home),
            "/usr/local/bin".to_string(),
            "/opt/homebrew/bin".to_string(),
        ];
        
        for p in &candidates {
            if std::path::Path::new(p).exists() {
                paths.push(p.clone());
            }
        }
    }
    
    #[cfg(target_os = "windows")]
    { paths.join(";") }
    #[cfg(not(target_os = "windows"))]
    { paths.join(":") }
}

/// 解析命令，Windows 上 .cmd 文件和 shell 内建命令需要通过 cmd /c 执行
pub fn resolve_command(cmd: &str, args: &[String]) -> (String, Vec<String>) {
    #[cfg(target_os = "windows")]
    {
        // Windows 上的 shell 内建命令和 .cmd 包装器
        let cmd_wrappers = ["echo", "type", "dir", "copy", "move", "del", "ren", "set", 
                           "npx", "npm", "pnpm", "yarn", "node", "uvx", "pipx"];
        let cmd_lower = cmd.to_lowercase();
        
        if cmd_wrappers.contains(&cmd_lower.as_str()) {
            let mut new_args = vec!["/c".to_string(), cmd.to_string()];
            new_args.extend(args.iter().cloned());
            return ("cmd".to_string(), new_args);
        }
    }
    
    (cmd.to_string(), args.to_vec())
}
