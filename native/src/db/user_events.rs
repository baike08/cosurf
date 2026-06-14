//! 用户行为事件模块
//! 
//! 用于追踪和存储用户在浏览器中的行为事件
//! 数据保留策略：最多保留最近 3 天

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{AppError, AppResult};

/// 用户行为事件类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    /// 打开标签页
    TabOpen,
    /// 关闭标签页
    TabClose,
    /// 切换标签页
    TabSwitch,
    /// 页面点击
    PageClick,
    /// 页面滚动
    PageScroll,
    /// 页面停留
    PageStay,
    /// URL 变化
    UrlChange,
    /// 表单输入
    FormInput,
    /// 窗口调整
    WindowResize,
    /// 页面加载
    PageLoad,
}

impl std::fmt::Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventType::TabOpen => write!(f, "tab_open"),
            EventType::TabClose => write!(f, "tab_close"),
            EventType::TabSwitch => write!(f, "tab_switch"),
            EventType::PageClick => write!(f, "page_click"),
            EventType::PageScroll => write!(f, "page_scroll"),
            EventType::PageStay => write!(f, "page_stay"),
            EventType::UrlChange => write!(f, "url_change"),
            EventType::FormInput => write!(f, "form_input"),
            EventType::WindowResize => write!(f, "window_resize"),
            EventType::PageLoad => write!(f, "page_load"),
        }
    }
}

impl std::str::FromStr for EventType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "tab_open" => Ok(EventType::TabOpen),
            "tab_close" => Ok(EventType::TabClose),
            "tab_switch" => Ok(EventType::TabSwitch),
            "page_click" => Ok(EventType::PageClick),
            "page_scroll" => Ok(EventType::PageScroll),
            "page_stay" => Ok(EventType::PageStay),
            "url_change" => Ok(EventType::UrlChange),
            "form_input" => Ok(EventType::FormInput),
            "window_resize" => Ok(EventType::WindowResize),
            "page_load" => Ok(EventType::PageLoad),
            _ => Err(format!("Unknown event type: {}", s)),
        }
    }
}

/// 事件数据（使用 JSON 对象存储灵活字段）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub focused: Option<bool>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag_name: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub class_name: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub element_id: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub element_text: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub click_x: Option<f64>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub click_y: Option<f64>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scroll_y: Option<f64>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scroll_height: Option<f64>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub viewport_height: Option<f64>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scroll_percent: Option<f64>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<i64>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub navigation_type: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_type: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_name: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<i64>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<i64>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub load_time: Option<i64>,
}

/// 用户行为事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserEvent {
    pub id: String,
    #[serde(rename = "type")]
    pub event_type: EventType,
    pub timestamp: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tab_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window_id: Option<i64>,
    pub data: EventData,
    pub created_at: i64,
}

/// 创建用户行为事件表
pub fn create_user_events_table(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS user_events (
            id TEXT PRIMARY KEY,
            type TEXT NOT NULL,
            timestamp INTEGER NOT NULL,
            url TEXT,
            tab_id TEXT,
            window_id INTEGER,
            data TEXT NOT NULL,
            created_at INTEGER DEFAULT (strftime('%s', 'now') * 1000)
        );
        
        CREATE INDEX IF NOT EXISTS idx_user_events_type ON user_events(type);
        CREATE INDEX IF NOT EXISTS idx_user_events_timestamp ON user_events(timestamp DESC);
        CREATE INDEX IF NOT EXISTS idx_user_events_url ON user_events(url);
        CREATE INDEX IF NOT EXISTS idx_user_events_tab_id ON user_events(tab_id);
        "#
    )?;
    
    Ok(())
}

/// 插入单个事件
pub fn insert_user_event(conn: &Connection, event: &UserEvent) -> AppResult<()> {
    let data_json = serde_json::to_string(&event.data)
        .map_err(|e| AppError::Internal(format!("Failed to serialize event data: {}", e)))?;
    
    conn.execute(
        r#"
        INSERT INTO user_events (id, type, timestamp, url, tab_id, window_id, data, created_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#,
        params![
            event.id,
            event.event_type.to_string(),
            event.timestamp,
            event.url,
            event.tab_id,
            event.window_id,
            data_json,
            event.created_at
        ],
    )?;
    
    Ok(())
}

/// 批量插入事件（性能优化）
pub fn batch_insert_user_events(conn: &Connection, events: &[UserEvent]) -> AppResult<usize> {
    if events.is_empty() {
        return Ok(0);
    }
    
    let tx = conn.unchecked_transaction()?;
    
    let mut count = 0;
    for event in events {
        insert_user_event(&tx, event)?;
        count += 1;
    }
    
    tx.commit()?;
    
    Ok(count)
}

/// 清理超过指定天数的旧数据
pub fn cleanup_old_user_events(conn: &Connection, retention_days: i64) -> AppResult<usize> {
    let cutoff_timestamp = chrono::Utc::now().timestamp_millis() - (retention_days * 24 * 60 * 60 * 1000);
    
    let changes = conn.execute(
        "DELETE FROM user_events WHERE timestamp < ?",
        params![cutoff_timestamp],
    )?;
    
    Ok(changes)
}

/// 获取指定时间范围内的事件
pub fn get_user_events(
    conn: &Connection,
    start_time: i64,
    end_time: i64,
    limit: i64,
) -> AppResult<Vec<UserEvent>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, type, timestamp, url, tab_id, window_id, data, created_at
        FROM user_events
        WHERE timestamp >= ? AND timestamp <= ?
        ORDER BY timestamp DESC
        LIMIT ?
        "#
    )?;
    
    let rows = stmt.query_map(
        params![start_time, end_time, limit],
        |row| {
            let data_json: String = row.get(6)?;
            let data: EventData = serde_json::from_str(&data_json)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                    6,
                    rusqlite::types::Type::Text,
                    Box::new(e)
                ))?;
            
            Ok(UserEvent {
                id: row.get(0)?,
                event_type: row.get::<_, String>(1)?.parse()
                    .map_err(|e: String| rusqlite::Error::FromSqlConversionFailure(
                        1,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e))
                    ))?,
                timestamp: row.get(2)?,
                url: row.get(3)?,
                tab_id: row.get(4)?,
                window_id: row.get(5)?,
                data,
                created_at: row.get(7)?,
            })
        }
    )?;
    
    let mut events = Vec::new();
    for event in rows {
        events.push(event?);
    }
    
    Ok(events)
}

/// 获取事件统计
pub fn get_event_stats(
    conn: &Connection,
    event_type: &EventType,
    days: i64,
) -> AppResult<EventStats> {
    let cutoff_time = chrono::Utc::now().timestamp_millis() - (days * 24 * 60 * 60 * 1000);
    
    let mut stmt = conn.prepare(
        r#"
        SELECT 
            COUNT(*) as count,
            MIN(timestamp) as first_occurrence,
            MAX(timestamp) as last_occurrence
        FROM user_events
        WHERE type = ? AND timestamp > ?
        "#
    )?;
    
    let stats = stmt.query_row(
        params![event_type.to_string(), cutoff_time],
        |row| {
            Ok(EventStats {
                count: row.get(0)?,
                first_occurrence: row.get(1).unwrap_or(0),
                last_occurrence: row.get(2).unwrap_or(0),
            })
        }
    )?;
    
    Ok(stats)
}

/// 获取页面停留统计
pub fn get_page_stay_stats(
    conn: &Connection,
    url: &str,
    days: i64,
) -> AppResult<PageStayStats> {
    let cutoff_time = chrono::Utc::now().timestamp_millis() - (days * 24 * 60 * 60 * 1000);
    
    let mut stmt = conn.prepare(
        r#"
        SELECT 
            COUNT(*) as visit_count,
            SUM(json_extract(data, '$.duration')) as total_duration,
            AVG(json_extract(data, '$.duration')) as avg_duration,
            MAX(json_extract(data, '$.duration')) as max_duration
        FROM user_events
        WHERE type = 'page_stay' AND url = ? AND timestamp > ?
        "#
    )?;
    
    let stats = stmt.query_row(
        params![url, cutoff_time],
        |row| {
            Ok(PageStayStats {
                visit_count: row.get(0).unwrap_or(0),
                total_duration: row.get(1).unwrap_or(0),
                avg_duration: row.get(2).unwrap_or(0),
                max_duration: row.get(3).unwrap_or(0),
            })
        }
    ).unwrap_or(PageStayStats::default());
    
    Ok(stats)
}

/// 获取最活跃的标签页
pub fn get_most_active_tabs(
    conn: &Connection,
    limit: i64,
) -> AppResult<Vec<ActiveTab>> {
    let cutoff_time = chrono::Utc::now().timestamp_millis() - (3 * 24 * 60 * 60 * 1000);
    
    let mut stmt = conn.prepare(
        r#"
        SELECT 
            tab_id,
            url,
            COUNT(*) as event_count,
            MAX(timestamp) as last_activity
        FROM user_events
        WHERE tab_id IS NOT NULL AND timestamp > ?
        GROUP BY tab_id, url
        ORDER BY event_count DESC
        LIMIT ?
        "#
    )?;
    
    let rows = stmt.query_map(params![cutoff_time, limit], |row| {
        Ok(ActiveTab {
            tab_id: row.get(0)?,
            url: row.get(1)?,
            event_count: row.get(2)?,
            last_activity: row.get(3)?,
        })
    })?;
    
    let mut tabs = Vec::new();
    for tab in rows {
        tabs.push(tab?);
    }
    
    Ok(tabs)
}

/// 生成 UUID
fn generate_uuid() -> String {
    Uuid::new_v4().to_string()
}

/// 创建新事件（辅助函数）
pub fn create_event(
    event_type: EventType,
    url: Option<String>,
    tab_id: Option<String>,
    window_id: Option<i64>,
    data: EventData,
) -> UserEvent {
    let now = chrono::Utc::now().timestamp_millis();
    
    UserEvent {
        id: generate_uuid(),
        event_type,
        timestamp: now,
        url,
        tab_id,
        window_id,
        data,
        created_at: now,
    }
}

/// 事件统计结果
#[derive(Debug, Serialize, Deserialize)]
pub struct EventStats {
    pub count: i64,
    pub first_occurrence: i64,
    pub last_occurrence: i64,
}

/// 页面停留统计结果
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct PageStayStats {
    pub visit_count: i64,
    pub total_duration: i64,
    pub avg_duration: i64,
    pub max_duration: i64,
}

/// 活跃标签页信息
#[derive(Debug, Serialize, Deserialize)]
pub struct ActiveTab {
    pub tab_id: String,
    pub url: String,
    pub event_count: i64,
    pub last_activity: i64,
}
