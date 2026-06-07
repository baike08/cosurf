use serde::Serialize;
use std::fmt;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("HTTP request error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Tauri error: {0}")]
    Tauri(#[from] tauri::Error),

    #[error("AI provider error: {0}")]
    AiProvider(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

// 为 Tauri IPC 实现 Serialize
impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
}

impl From<AppError> for ErrorResponse {
    fn from(err: AppError) -> Self {
        let (code, message) = match &err {
            AppError::Database(e) => ("DATABASE_ERROR".into(), e.to_string()),
            AppError::Http(e) => ("HTTP_ERROR".into(), e.to_string()),
            AppError::Json(e) => ("JSON_ERROR".into(), e.to_string()),
            AppError::Tauri(e) => ("TAURI_ERROR".into(), e.to_string()),
            AppError::AiProvider(msg) => ("AI_PROVIDER_ERROR".into(), msg.clone()),
            AppError::Config(msg) => ("CONFIG_ERROR".into(), msg.clone()),
            AppError::NotFound(msg) => ("NOT_FOUND".into(), msg.clone()),
            AppError::Internal(msg) => ("INTERNAL_ERROR".into(), msg.clone()),
        };
        ErrorResponse { code, message }
    }
}

pub type AppResult<T> = Result<T, AppError>;
