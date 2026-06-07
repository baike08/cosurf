use tauri::State;

use crate::db::history::{AddHistoryRequest, HistoryEntry};
use crate::error::ErrorResponse;
use crate::state::AppState;

#[tauri::command]
pub fn list_history(
    state: State<'_, AppState>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<HistoryEntry>, ErrorResponse> {
    let db = state.db.lock().map_err(|e| ErrorResponse {
        code: "LOCK_ERROR".into(),
        message: e.to_string(),
    })?;
    db.list_history(limit.unwrap_or(50), offset.unwrap_or(0))
        .map_err(|e| ErrorResponse::from(e))
}

#[tauri::command]
pub fn search_history(
    state: State<'_, AppState>,
    query: String,
    limit: Option<i64>,
) -> Result<Vec<HistoryEntry>, ErrorResponse> {
    let db = state.db.lock().map_err(|e| ErrorResponse {
        code: "LOCK_ERROR".into(),
        message: e.to_string(),
    })?;
    db.search_history(&query, limit.unwrap_or(50))
        .map_err(|e| ErrorResponse::from(e))
}

#[tauri::command]
pub fn add_history(
    state: State<'_, AppState>,
    request: AddHistoryRequest,
) -> Result<HistoryEntry, ErrorResponse> {
    let db = state.db.lock().map_err(|e| ErrorResponse {
        code: "LOCK_ERROR".into(),
        message: e.to_string(),
    })?;
    db.add_history(&request).map_err(|e| ErrorResponse::from(e))
}

#[tauri::command]
pub fn clear_history(state: State<'_, AppState>) -> Result<(), ErrorResponse> {
    let db = state.db.lock().map_err(|e| ErrorResponse {
        code: "LOCK_ERROR".into(),
        message: e.to_string(),
    })?;
    db.clear_history().map_err(|e| ErrorResponse::from(e))
}

#[tauri::command]
pub fn delete_history_entry(state: State<'_, AppState>, id: String) -> Result<(), ErrorResponse> {
    let db = state.db.lock().map_err(|e| ErrorResponse {
        code: "LOCK_ERROR".into(),
        message: e.to_string(),
    })?;
    db.delete_history_entry(&id).map_err(|e| ErrorResponse::from(e))
}
