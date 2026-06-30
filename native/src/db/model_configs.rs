//! 模型配置（Model Configs）模块

use rusqlite::{params, Connection, OptionalExtension};
use uuid::Uuid;

use crate::error::{AppError, AppResult};

/// 模型配置结构
#[derive(Debug, Clone)]
pub struct ModelConfig {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub model_id: String,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub temperature: f64,
    pub top_p: f64,
    pub max_tokens: i64,
    pub is_local: bool,
    pub is_active: bool,
}

/// 创建模型配置表
pub fn create_model_configs_table(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS model_configs (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            provider TEXT NOT NULL,
            model_id TEXT NOT NULL,
            api_key TEXT,
            base_url TEXT,
            temperature REAL NOT NULL DEFAULT 0.7,
            top_p REAL NOT NULL DEFAULT 1.0,
            max_tokens INTEGER NOT NULL DEFAULT 4096,
            is_local INTEGER NOT NULL DEFAULT 0,
            is_active INTEGER NOT NULL DEFAULT 0
        );

        CREATE INDEX IF NOT EXISTS idx_model_configs_active ON model_configs(is_active DESC);
        CREATE INDEX IF NOT EXISTS idx_model_configs_name ON model_configs(name);
        "#
    )?;
    
    Ok(())
}

/// 列出所有模型配置
pub fn list_model_configs(conn: &Connection) -> AppResult<Vec<ModelConfig>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, provider, model_id, api_key, base_url, temperature, top_p, max_tokens, is_local, is_active
         FROM model_configs ORDER BY is_active DESC, name ASC"
    )?;
    
    let rows = stmt.query_map([], |row| {
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
    })?;
    
    let mut configs = Vec::new();
    for config in rows {
        configs.push(config?);
    }
    
    Ok(configs)
}

/// 获取活跃的模型配置
pub fn get_active_model(conn: &Connection) -> AppResult<Option<ModelConfig>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, provider, model_id, api_key, base_url, temperature, top_p, max_tokens, is_local, is_active
         FROM model_configs WHERE is_active = 1 LIMIT 1"
    )?;
    
    let config = stmt.query_row([], |row| {
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
    }).optional()?;
    
    Ok(config)
}

/// 获取单个模型配置
pub fn get_model_config(conn: &Connection, id: &str) -> AppResult<Option<ModelConfig>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, provider, model_id, api_key, base_url, temperature, top_p, max_tokens, is_local, is_active
         FROM model_configs WHERE id = ?1"
    )?;
    
    let config = stmt.query_row(params![id], |row| {
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
    }).optional()?;
    
    Ok(config)
}

/// 创建模型配置
pub fn create_model_config(
    conn: &Connection,
    name: String,
    provider: String,
    model_id: String,
    api_key: Option<String>,
    base_url: Option<String>,
    temperature: Option<f64>,
    top_p: Option<f64>,
    max_tokens: Option<i64>,
) -> AppResult<ModelConfig> {
    let id = Uuid::new_v4().to_string();
    
    conn.execute(
        "INSERT INTO model_configs (id, name, provider, model_id, api_key, base_url,
         temperature, top_p, max_tokens, is_local, is_active)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 0, 0)",
        params![
            id,
            name,
            provider,
            model_id,
            api_key,
            base_url,
            temperature.unwrap_or(0.7),
            top_p.unwrap_or(1.0),
            max_tokens.unwrap_or(4096),
        ],
    )?;
    
    // 读取刚创建的记录
    get_model_config(conn, &id)
        .map(|opt| opt.expect("Just created model config should exist"))
}

/// 更新模型配置
pub fn update_model_config(
    conn: &Connection,
    id: &str,
    name: Option<String>,
    api_key: Option<Option<String>>,
    base_url: Option<Option<String>>,
    temperature: Option<f64>,
    top_p: Option<f64>,
    max_tokens: Option<i64>,
) -> AppResult<ModelConfig> {
    // 获取现有配置
    let existing = get_model_config(conn, id)?
        .ok_or_else(|| AppError::NotFound(format!("Model config {} not found", id)))?;
    
    let new_name = name.as_deref().unwrap_or(&existing.name);
    let new_api_key = api_key.flatten().or(existing.api_key);
    let new_base_url = base_url.flatten().or(existing.base_url);
    let new_temp = temperature.unwrap_or(existing.temperature);
    let new_top_p = top_p.unwrap_or(existing.top_p);
    let new_max = max_tokens.unwrap_or(existing.max_tokens);
    
    conn.execute(
        "UPDATE model_configs SET name = ?1, api_key = ?2, base_url = ?3,
         temperature = ?4, top_p = ?5, max_tokens = ?6 WHERE id = ?7",
        params![new_name, new_api_key, new_base_url, new_temp, new_top_p, new_max, id],
    )?;
    
    // 读取更新后的记录
    get_model_config(conn, id)
        .map(|opt| opt.expect("Updated model config should exist"))
}

/// 设置活跃模型
pub fn set_active_model(conn: &Connection, id: &str) -> AppResult<()> {
    // 先取消所有模型的活跃状态
    conn.execute("UPDATE model_configs SET is_active = 0", [])?;
    
    // 设置指定模型为活跃
    let affected = conn.execute(
        "UPDATE model_configs SET is_active = 1 WHERE id = ?1",
        params![id],
    )?;
    
    if affected == 0 {
        return Err(AppError::NotFound(format!("Model config {} not found", id)));
    }
    
    Ok(())
}

/// 删除模型配置
pub fn delete_model_config(conn: &Connection, id: &str) -> AppResult<()> {
    let affected = conn.execute("DELETE FROM model_configs WHERE id = ?1", params![id])?;
    
    if affected == 0 {
        return Err(AppError::NotFound(format!("Model config {} not found", id)));
    }
    
    Ok(())
}
