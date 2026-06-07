use tauri::State;

use crate::db::messages::{CreateMessageRequest, Message, UpdateMessageRequest};
use crate::error::ErrorResponse;
use crate::state::AppState;

#[tauri::command]
pub fn list_messages(state: State<'_, AppState>, conversation_id: String) -> Result<Vec<Message>, ErrorResponse> {
    let db = state.db.lock().map_err(|e| ErrorResponse {
        code: "LOCK_ERROR".into(),
        message: e.to_string(),
    })?;
    db.list_messages(&conversation_id).map_err(|e| ErrorResponse::from(e))
}

#[tauri::command]
pub fn get_message(state: State<'_, AppState>, id: String) -> Result<Message, ErrorResponse> {
    let db = state.db.lock().map_err(|e| ErrorResponse {
        code: "LOCK_ERROR".into(),
        message: e.to_string(),
    })?;
    db.get_message(&id).map_err(|e| ErrorResponse::from(e))
}

#[tauri::command]
pub fn create_message(
    state: State<'_, AppState>,
    request: CreateMessageRequest,
) -> Result<Message, ErrorResponse> {
    let db = state.db.lock().map_err(|e| ErrorResponse {
        code: "LOCK_ERROR".into(),
        message: e.to_string(),
    })?;
    db.create_message(&request).map_err(|e| ErrorResponse::from(e))
}

#[tauri::command]
pub fn update_message(
    state: State<'_, AppState>,
    id: String,
    request: UpdateMessageRequest,
) -> Result<Message, ErrorResponse> {
    let db = state.db.lock().map_err(|e| ErrorResponse {
        code: "LOCK_ERROR".into(),
        message: e.to_string(),
    })?;
    db.update_message(&id, &request).map_err(|e| ErrorResponse::from(e))
}

#[tauri::command]
pub fn delete_message(state: State<'_, AppState>, id: String) -> Result<(), ErrorResponse> {
    let db = state.db.lock().map_err(|e| ErrorResponse {
        code: "LOCK_ERROR".into(),
        message: e.to_string(),
    })?;
    db.delete_message(&id).map_err(|e| ErrorResponse::from(e))
}

/// 追加消息内容（用于流式响应）
#[tauri::command]
pub fn append_message_content(
    state: State<'_, AppState>,
    id: String,
    content: String,
) -> Result<(), ErrorResponse> {
    let db = state.db.lock().map_err(|e| ErrorResponse {
        code: "LOCK_ERROR".into(),
        message: e.to_string(),
    })?;
    db.append_message_content(&id, &content, false).map_err(|e| ErrorResponse::from(e))
}

/// 完成流式消息
#[tauri::command]
pub fn complete_message(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), ErrorResponse> {
    let db = state.db.lock().map_err(|e| ErrorResponse {
        code: "LOCK_ERROR".into(),
        message: e.to_string(),
    })?;
    db.complete_message(&id).map_err(|e| ErrorResponse::from(e))
}

/// 设置消息反馈（like / dislike / 取消）
#[tauri::command]
pub fn set_message_feedback(
    state: State<'_, AppState>,
    id: String,
    feedback: String,
) -> Result<Message, ErrorResponse> {
    let db = state.db.lock().map_err(|e| ErrorResponse {
        code: "LOCK_ERROR".into(),
        message: e.to_string(),
    })?;
    db.set_message_feedback(&id, &feedback).map_err(|e| ErrorResponse::from(e))
}
