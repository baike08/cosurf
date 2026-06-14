//! Native 模块错误类型
//!
//! 从 Tauri 版 error.rs 迁移，移除 Tauri 特有错误

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("HTTP request error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("AI provider error: {0}")]
    AiProvider(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// 将 AppError 转换为 napi::Error
impl From<AppError> for napi::Error {
    fn from(err: AppError) -> Self {
        napi::Error::from_reason(err.to_string())
    }
}

pub type AppResult<T> = std::result::Result<T, AppError>;
