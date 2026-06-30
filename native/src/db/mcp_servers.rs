//! MCP 服务器（MCP Servers）模块

use rusqlite::{params, Connection, OptionalExtension};
use uuid::Uuid;

use crate::error::{AppError, AppResult};

/// MCP 服务器结构
#[derive(Debug, Clone)]
pub struct McpServer {
    pub id: String,
    pub name: String,
    pub server_type: String,
    pub url: Option<String>,
    pub command: Option<String>,
    pub args: Option<String>,
    pub cwd: Option<String>,
    pub env: Option<String>,
    pub disabled: bool,
    pub timeout: Option<i64>,
    pub enabled: bool,
    pub created_at: i64,
    pub updated_at: i64,
    pub headers: Option<String>,
}

/// 创建 MCP 服务器表
pub fn create_mcp_servers_table(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS mcp_servers (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            server_type TEXT NOT NULL DEFAULT 'stdio',
            url TEXT,
            command TEXT,
            args TEXT,
            cwd TEXT,
            env TEXT,
            disabled INTEGER NOT NULL DEFAULT 0,
            timeout INTEGER,
            enabled INTEGER NOT NULL DEFAULT 1,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            headers TEXT
        );

        CREATE INDEX IF NOT EXISTS idx_mcp_servers_name ON mcp_servers(name);
        CREATE INDEX IF NOT EXISTS idx_mcp_servers_enabled ON mcp_servers(enabled);
        "#
    )?;
    
    Ok(())
}

/// 列出所有 MCP 服务器
pub fn list_mcp_servers(conn: &Connection) -> AppResult<Vec<McpServer>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, server_type, url, command, args, cwd, env, disabled, timeout, enabled, created_at, updated_at, headers
         FROM mcp_servers ORDER BY name ASC"
    )?;
    
    let rows = stmt.query_map([], |row| {
        Ok(McpServer {
            id: row.get(0)?,
            name: row.get(1)?,
            server_type: row.get(2)?,
            url: row.get(3)?,
            command: row.get(4)?,
            args: row.get(5)?,
            cwd: row.get(6)?,
            env: row.get(7)?,
            disabled: row.get::<_, i32>(8)? != 0,
            timeout: row.get(9)?,
            enabled: row.get::<_, i32>(10)? != 0,
            created_at: row.get(11)?,
            updated_at: row.get(12)?,
            headers: row.get(13)?,
        })
    })?;
    
    let mut servers = Vec::new();
    for server in rows {
        servers.push(server?);
    }
    
    Ok(servers)
}

/// 获取单个 MCP 服务器
pub fn get_mcp_server(conn: &Connection, id: &str) -> AppResult<Option<McpServer>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, server_type, url, command, args, cwd, env, disabled, timeout, enabled, created_at, updated_at, headers
         FROM mcp_servers WHERE id = ?1"
    )?;
    
    let server = stmt.query_row(params![id], |row| {
        Ok(McpServer {
            id: row.get(0)?,
            name: row.get(1)?,
            server_type: row.get(2)?,
            url: row.get(3)?,
            command: row.get(4)?,
            args: row.get(5)?,
            cwd: row.get(6)?,
            env: row.get(7)?,
            disabled: row.get::<_, i32>(8)? != 0,
            timeout: row.get(9)?,
            enabled: row.get::<_, i32>(10)? != 0,
            created_at: row.get(11)?,
            updated_at: row.get(12)?,
            headers: row.get(13)?,
        })
    }).optional()?;
    
    Ok(server)
}

/// 创建 MCP 服务器
pub fn create_mcp_server(
    conn: &Connection,
    name: String,
    server_type: Option<String>,
    url: Option<String>,
    command: Option<String>,
    args: Option<String>,
    cwd: Option<String>,
    env: Option<String>,
    timeout: Option<i64>,
    disabled: Option<bool>,
    enabled: Option<bool>,
    headers: Option<String>,
) -> AppResult<McpServer> {
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp();
    let st = server_type.as_deref().unwrap_or("stdio");
    let dis = disabled.unwrap_or(false);
    let en = enabled.unwrap_or(!dis);
    
    conn.execute(
        "INSERT INTO mcp_servers (id, name, server_type, url, command, args, cwd, env, disabled, timeout, enabled, created_at, updated_at, headers)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
        params![
            id,
            name,
            st,
            url,
            command,
            args,
            cwd,
            env,
            dis as i32,
            timeout,
            en as i32,
            now,
            now,
            headers,
        ],
    )?;
    
    // 读取刚创建的记录
    get_mcp_server(conn, &id)
        .map(|opt| opt.expect("Just created MCP server should exist"))
}

/// 更新 MCP 服务器
pub fn update_mcp_server(
    conn: &Connection,
    id: &str,
    name: Option<String>,
    server_type: Option<String>,
    url: Option<Option<String>>,
    command: Option<Option<String>>,
    args: Option<Option<String>>,
    cwd: Option<Option<String>>,
    env: Option<Option<String>>,
    timeout: Option<Option<i64>>,
    disabled: Option<bool>,
    enabled: Option<bool>,
    headers: Option<Option<String>>,
) -> AppResult<McpServer> {
    // 获取现有配置
    let existing = get_mcp_server(conn, id)?
        .ok_or_else(|| AppError::NotFound(format!("MCP server {} not found", id)))?;
    
    let now = chrono::Utc::now().timestamp();
    let new_name = name.as_deref().unwrap_or(&existing.name);
    let new_st = server_type.as_deref().unwrap_or(&existing.server_type);
    let new_url = url.flatten().or(existing.url);
    let new_cmd = command.flatten().or(existing.command);
    let new_args = args.flatten().or(existing.args);
    let new_cwd = cwd.flatten().or(existing.cwd);
    let new_env = env.flatten().or(existing.env);
    let new_timeout = timeout.flatten().or(existing.timeout);
    let new_dis = disabled.unwrap_or(existing.disabled);
    let new_en = enabled.unwrap_or(existing.enabled);
    let new_headers = headers.flatten().or(existing.headers);
    
    conn.execute(
        "UPDATE mcp_servers SET name = ?1, server_type = ?2, url = ?3,
         command = ?4, args = ?5, cwd = ?6, env = ?7, disabled = ?8, timeout = ?9, enabled = ?10, updated_at = ?11, headers = ?12
         WHERE id = ?13",
        params![
            new_name, new_st, new_url, new_cmd, new_args, new_cwd, new_env,
            new_dis as i32, new_timeout, new_en as i32, now, new_headers, id
        ],
    )?;
    
    // 读取更新后的记录
    get_mcp_server(conn, id)
        .map(|opt| opt.expect("Updated MCP server should exist"))
}

/// 删除 MCP 服务器
pub fn delete_mcp_server(conn: &Connection, id: &str) -> AppResult<()> {
    let affected = conn.execute("DELETE FROM mcp_servers WHERE id = ?1", params![id])?;
    
    if affected == 0 {
        return Err(AppError::NotFound(format!("MCP server {} not found", id)));
    }
    
    Ok(())
}
