use rusqlite::params;
use serde::{Deserialize, Serialize};

use crate::db::Database;
use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelConfig {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub model_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    pub temperature: f64,
    pub top_p: f64,
    pub max_tokens: i64,
    pub is_local: bool,
    pub is_active: bool,
}

// ==================== MCP Server 配置 ====================

/// MCP Server 类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum McpServerType {
    /// HTTP/SSE 服务器
    #[serde(alias = "HTTP")]
    Http,
    /// Streamable HTTP（MCP 新标准，POST JSON-RPC 到 url）
    #[serde(alias = "streamable-http", alias = "streamable_http")]
    StreamableHttp,
    /// SSE（Server-Sent Events）
    #[serde(alias = "SSE")]
    Sse,
    /// 本地进程（通过 stdio）
    #[serde(alias = "STDIO")]
    Stdio,
}

impl Default for McpServerType {
    fn default() -> Self {
        McpServerType::Stdio
    }
}

/// 从 JSON 字符串解析 MCP 类型
pub fn parse_mcp_type_str(s: &str) -> McpServerType {
    match s.to_lowercase().as_str() {
        "stdio" => McpServerType::Stdio,
        "http" => McpServerType::Http,
        "streamablehttp" | "streamable-http" | "streamable_http" => McpServerType::StreamableHttp,
        "sse" => McpServerType::Sse,
        _ => McpServerType::Stdio,
    }
}

fn mcp_type_to_str(t: &McpServerType) -> &'static str {
    match t {
        McpServerType::Stdio => "stdio",
        McpServerType::Http => "http",
        McpServerType::StreamableHttp => "streamableHttp",
        McpServerType::Sse => "sse",
    }
}

/// MCP Server 配置（兼容开源标准）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpServerConfig {
    pub id: String,
    pub name: String,
    #[serde(default = "default_mcp_server_type")]
    pub server_type: McpServerType,
    
    // === HTTP 模式字段 ===
    /// HTTP/SSE/StreamableHttp 服务器的 URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// 自定义 HTTP headers（如 X-API-Key）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<serde_json::Map<String, serde_json::Value>>,
    
    // === stdio 模式字段 ===
    /// 启动命令（如 npx, node, python, uvx）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    /// 命令行参数
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<String>>,
    /// 工作目录
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    
    // === 通用字段 ===
    /// 环境变量
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<serde_json::Map<String, serde_json::Value>>,
    /// 是否禁用（注意：JSON 中是 disabled，我们存储为 enabled）
    #[serde(default)]
    pub disabled: bool,
    /// 超时时间（秒）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,
    
    // === 内部字段 ===
    pub enabled: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

fn default_mcp_server_type() -> McpServerType {
    McpServerType::Stdio
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateMcpServerRequest {
    pub name: String,
    #[serde(default)]
    pub server_type: Option<McpServerType>,
    pub url: Option<String>,
    pub headers: Option<serde_json::Map<String, serde_json::Value>>,
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    pub cwd: Option<String>,
    pub env: Option<serde_json::Map<String, serde_json::Value>>,
    #[serde(default)]
    pub disabled: Option<bool>,
    pub timeout: Option<u64>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMcpServerRequest {
    pub name: Option<String>,
    pub server_type: Option<McpServerType>,
    pub url: Option<String>,
    pub headers: Option<serde_json::Map<String, serde_json::Value>>,
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    pub cwd: Option<String>,
    pub env: Option<serde_json::Map<String, serde_json::Value>>,
    pub disabled: Option<bool>,
    pub timeout: Option<u64>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateModelConfigRequest {
    pub name: String,
    pub provider: String,
    pub model_id: String,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
    pub max_tokens: Option<i64>,
    pub is_local: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateModelConfigRequest {
    pub name: Option<String>,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
    pub max_tokens: Option<i64>,
}

impl Database {
    pub fn get_setting(&self, key: &str) -> AppResult<Option<String>> {
        let mut stmt = self.conn().prepare("SELECT value FROM settings WHERE key = ?1")?;
        let result = stmt.query_row(params![key], |row| row.get(0));
        match result {
            Ok(value) => Ok(Some(value)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn set_setting(&self, key: &str, value: &str) -> AppResult<()> {
        self.conn().execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn get_all_settings(&self) -> AppResult<serde_json::Value> {
        let mut stmt = self.conn().prepare("SELECT key, value FROM settings")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;

        let mut map = serde_json::Map::new();
        for row in rows {
            let (key, value) = row?;
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&value) {
                map.insert(key, parsed);
            } else {
                map.insert(key, serde_json::Value::String(value));
            }
        }
        Ok(serde_json::Value::Object(map))
    }

    pub fn list_model_configs(&self) -> AppResult<Vec<ModelConfig>> {
        let mut stmt = self.conn().prepare(
            "SELECT id, name, provider, model_id, api_key, base_url,
                    temperature, top_p, max_tokens, is_local, is_active
             FROM model_configs ORDER BY is_active DESC, name ASC",
        )?;

        let rows = stmt.query_map([], Self::map_model_config)?;

        let mut configs = Vec::new();
        for row in rows {
            configs.push(row?);
        }
        Ok(configs)
    }

    fn map_model_config(row: &rusqlite::Row) -> rusqlite::Result<ModelConfig> {
        Ok(ModelConfig {
            id: row.get(0)?,
            name: row.get(1)?,
            provider: row.get(2)?,
            model_id: row.get(3)?,
            api_key: row.get(4)?,
            base_url: row.get(5)?,
            temperature: row.get(6)?,
            top_p: row.get(7)?,
            max_tokens: row.get(8)?,
            is_local: row.get::<_, i32>(9)? != 0,
            is_active: row.get::<_, i32>(10)? != 0,
        })
    }

    pub fn get_model_config(&self, id: &str) -> AppResult<ModelConfig> {
        let mut stmt = self.conn().prepare(
            "SELECT id, name, provider, model_id, api_key, base_url,
                    temperature, top_p, max_tokens, is_local, is_active
             FROM model_configs WHERE id = ?1",
        )?;

        stmt.query_row(params![id], Self::map_model_config)
            .map_err(|_| AppError::NotFound(format!("Model config {} not found", id)))
    }

    pub fn get_active_model_config(&self) -> AppResult<Option<ModelConfig>> {
        let mut stmt = self.conn().prepare(
            "SELECT id, name, provider, model_id, api_key, base_url,
                    temperature, top_p, max_tokens, is_local, is_active
             FROM model_configs WHERE is_active = 1 LIMIT 1",
        )?;

        let result = stmt.query_row([], Self::map_model_config);
        match result {
            Ok(config) => Ok(Some(config)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn create_model_config(&self, req: &CreateModelConfigRequest) -> AppResult<ModelConfig> {
        let id = uuid::Uuid::new_v4().to_string();

        self.conn().execute(
            "INSERT INTO model_configs (id, name, provider, model_id, api_key, base_url,
             temperature, top_p, max_tokens, is_local, is_active)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 0)",
            params![
                id,
                req.name,
                req.provider,
                req.model_id,
                req.api_key,
                req.base_url,
                req.temperature.unwrap_or(0.7),
                req.top_p.unwrap_or(1.0),
                req.max_tokens.unwrap_or(4096),
                req.is_local.unwrap_or(false) as i32,
            ],
        )?;

        self.get_model_config(&id)
    }

    pub fn update_model_config(&self, id: &str, req: &UpdateModelConfigRequest) -> AppResult<ModelConfig> {
        let existing = self.get_model_config(id)?;

        self.conn().execute(
            "UPDATE model_configs SET name = ?1, api_key = ?2, base_url = ?3,
             temperature = ?4, top_p = ?5, max_tokens = ?6 WHERE id = ?7",
            params![
                req.name.as_deref().unwrap_or(&existing.name),
                req.api_key.as_deref().or(existing.api_key.as_deref()),
                req.base_url.as_deref().or(existing.base_url.as_deref()),
                req.temperature.unwrap_or(existing.temperature),
                req.top_p.unwrap_or(existing.top_p),
                req.max_tokens.unwrap_or(existing.max_tokens),
                id,
            ],
        )?;

        self.get_model_config(id)
    }

    pub fn set_active_model(&self, id: &str) -> AppResult<()> {
        self.conn().execute("UPDATE model_configs SET is_active = 0", [])?;
        let affected = self.conn().execute(
            "UPDATE model_configs SET is_active = 1 WHERE id = ?1",
            params![id],
        )?;
        if affected == 0 {
            return Err(AppError::NotFound(format!("Model config {} not found", id)));
        }
        Ok(())
    }

    pub fn delete_model_config(&self, id: &str) -> AppResult<()> {
        let affected = self.conn().execute("DELETE FROM model_configs WHERE id = ?1", params![id])?;
        if affected == 0 {
            return Err(AppError::NotFound(format!("Model config {} not found", id)));
        }
        Ok(())
    }

    // ==================== Skills 配置 ====================

    /// 获取 Skills 目录路径
    pub fn get_skills_directory(&self) -> AppResult<String> {
        match self.get_setting("skills.directory")? {
            Some(dir) => Ok(dir),
            None => {
                // 默认路径：~/.cosurf/skills
                let default_dir = dirs::home_dir()
                    .unwrap_or_else(|| std::env::temp_dir())
                    .join(".cosurf")
                    .join("skills")
                    .to_string_lossy()
                    .to_string();
                
                // 保存默认值
                self.set_setting("skills.directory", &default_dir)?;
                Ok(default_dir)
            }
        }
    }

    /// 设置 Skills 目录路径
    pub fn set_skills_directory(&self, directory: &str) -> AppResult<()> {
        self.set_setting("skills.directory", directory)
    }

    // ==================== IQS API Key 配置 ====================

    /// 获取阿里云 IQS API Key
    pub fn get_iqs_api_key(&self) -> AppResult<Option<String>> {
        self.get_setting("iqs.api_key")
    }

    /// 设置阿里云 IQS API Key
    pub fn set_iqs_api_key(&self, api_key: &str) -> AppResult<()> {
        self.set_setting("iqs.api_key", api_key)
    }

    // ==================== MCP Server 配置 ====================

    /// 获取所有 MCP Server 配置
    pub fn list_mcp_servers(&self) -> AppResult<Vec<McpServerConfig>> {
        let mut stmt = self.conn().prepare(
            "SELECT id, name, server_type, url, command, args, cwd, env, disabled, timeout, enabled, created_at, updated_at, headers
             FROM mcp_servers ORDER BY created_at DESC",
        )?;

        let rows = stmt.query_map([], Self::map_mcp_server_config)?;

        let mut servers = Vec::new();
        for row in rows {
            servers.push(row?);
        }
        Ok(servers)
    }

    fn map_mcp_server_config(row: &rusqlite::Row) -> rusqlite::Result<McpServerConfig> {
        let server_type_str: String = row.get(2)?;
        let server_type = parse_mcp_type_str(&server_type_str);
        
        // 解析 JSON 字段
        let args: Option<Vec<String>> = row.get::<_, Option<String>>(5)?
            .and_then(|s| serde_json::from_str(&s).ok());
        
        let env: Option<serde_json::Map<String, serde_json::Value>> = row.get::<_, Option<String>>(7)?
            .and_then(|s| serde_json::from_str(&s).ok());
        
        let headers: Option<serde_json::Map<String, serde_json::Value>> = row.get::<_, Option<String>>(13)?
            .and_then(|s| serde_json::from_str(&s).ok());
        
        Ok(McpServerConfig {
            id: row.get(0)?,
            name: row.get(1)?,
            server_type,
            url: row.get(3)?,
            headers,
            command: row.get(4)?,
            args,
            cwd: row.get(6)?,
            env,
            disabled: row.get::<_, i32>(8)? != 0,
            timeout: row.get(9)?,
            enabled: row.get::<_, i32>(10)? != 0,
            created_at: row.get(11)?,
            updated_at: row.get(12)?,
        })
    }

    /// 获取单个 MCP Server 配置
    pub fn get_mcp_server(&self, id: &str) -> AppResult<McpServerConfig> {
        let mut stmt = self.conn().prepare(
            "SELECT id, name, server_type, url, command, args, cwd, env, disabled, timeout, enabled, created_at, updated_at, headers
             FROM mcp_servers WHERE id = ?1",
        )?;

        stmt.query_row(params![id], Self::map_mcp_server_config)
            .map_err(|_| AppError::NotFound(format!("MCP server {} not found", id)))
    }

    /// 创建 MCP Server 配置
    pub fn create_mcp_server(&self, req: &CreateMcpServerRequest) -> AppResult<McpServerConfig> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp();
        
        let server_type = req.server_type.clone().unwrap_or_default();
        let server_type_str = mcp_type_to_str(&server_type);
        
        // 序列化 JSON 字段
        let args_json = req.args.as_ref()
            .and_then(|args| serde_json::to_string(args).ok());
        let env_json = req.env.as_ref()
            .and_then(|env| serde_json::to_string(env).ok());
        let headers_json = req.headers.as_ref()
            .and_then(|h| serde_json::to_string(h).ok());
        
        let disabled = req.disabled.unwrap_or(false);
        let enabled = req.enabled.unwrap_or(!disabled);

        self.conn().execute(
            "INSERT INTO mcp_servers (id, name, server_type, url, command, args, cwd, env, disabled, timeout, enabled, created_at, updated_at, headers)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            params![
                id,
                req.name,
                server_type_str,
                req.url,
                req.command,
                args_json,
                req.cwd,
                env_json,
                disabled as i32,
                req.timeout,
                enabled as i32,
                now,
                now,
                headers_json,
            ],
        )?;

        self.get_mcp_server(&id)
    }

    /// 更新 MCP Server 配置
    pub fn update_mcp_server(&self, id: &str, req: &UpdateMcpServerRequest) -> AppResult<McpServerConfig> {
        let existing = self.get_mcp_server(id)?;
        let now = chrono::Utc::now().timestamp();
        
        let server_type = req.server_type.as_ref().unwrap_or(&existing.server_type);
        let server_type_str = mcp_type_to_str(server_type);
        
        // 序列化 JSON 字段
        let args_json = req.args.as_ref()
            .or(existing.args.as_ref())
            .and_then(|args| serde_json::to_string(args).ok());
        let env_json = req.env.as_ref()
            .or(existing.env.as_ref())
            .and_then(|env| serde_json::to_string(env).ok());
        let headers_json = req.headers.as_ref()
            .or(existing.headers.as_ref())
            .and_then(|h| serde_json::to_string(h).ok());
        
        let disabled = req.disabled.unwrap_or(existing.disabled);
        let enabled = req.enabled.unwrap_or(existing.enabled);

        self.conn().execute(
            "UPDATE mcp_servers SET name = ?1, server_type = ?2, url = ?3, 
             command = ?4, args = ?5, cwd = ?6, env = ?7, disabled = ?8, timeout = ?9, enabled = ?10, updated_at = ?11, headers = ?13
             WHERE id = ?12",
            params![
                req.name.as_deref().unwrap_or(&existing.name),
                server_type_str,
                req.url.as_deref().or(existing.url.as_deref()),
                req.command.as_deref().or(existing.command.as_deref()),
                args_json,
                req.cwd.as_deref().or(existing.cwd.as_deref()),
                env_json,
                disabled as i32,
                req.timeout.or(existing.timeout),
                enabled as i32,
                now,
                id,
                headers_json,
            ],
        )?;

        self.get_mcp_server(id)
    }

    /// 删除 MCP Server 配置
    pub fn delete_mcp_server(&self, id: &str) -> AppResult<()> {
        let affected = self.conn().execute(
            "DELETE FROM mcp_servers WHERE id = ?1",
            params![id],
        )?;
        if affected == 0 {
            return Err(AppError::NotFound(format!("MCP server {} not found", id)));
        }
        Ok(())
    }
}
