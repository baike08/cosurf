use tauri::State;

use crate::db::conversations::{Conversation, CreateConversationRequest, UpdateConversationRequest};
use crate::db::messages::Message;
use crate::error::ErrorResponse;
use crate::state::AppState;

#[tauri::command]
pub fn list_conversations(state: State<'_, AppState>) -> Result<Vec<Conversation>, ErrorResponse> {
    let db = state.db.lock().map_err(|e| ErrorResponse {
        code: "LOCK_ERROR".into(),
        message: e.to_string(),
    })?;
    db.list_conversations().map_err(|e| ErrorResponse::from(e))
}

#[tauri::command]
pub fn get_conversation(state: State<'_, AppState>, id: String) -> Result<Conversation, ErrorResponse> {
    let db = state.db.lock().map_err(|e| ErrorResponse {
        code: "LOCK_ERROR".into(),
        message: e.to_string(),
    })?;
    db.get_conversation(&id).map_err(|e| ErrorResponse::from(e))
}

#[tauri::command]
pub fn create_conversation(
    state: State<'_, AppState>,
    request: CreateConversationRequest,
) -> Result<Conversation, ErrorResponse> {
    let db = state.db.lock().map_err(|e| ErrorResponse {
        code: "LOCK_ERROR".into(),
        message: e.to_string(),
    })?;
    db.create_conversation(&request).map_err(|e| ErrorResponse::from(e))
}

#[tauri::command]
pub fn update_conversation(
    state: State<'_, AppState>,
    id: String,
    request: UpdateConversationRequest,
) -> Result<Conversation, ErrorResponse> {
    let db = state.db.lock().map_err(|e| ErrorResponse {
        code: "LOCK_ERROR".into(),
        message: e.to_string(),
    })?;
    db.update_conversation(&id, &request).map_err(|e| ErrorResponse::from(e))
}

#[tauri::command]
pub fn delete_conversation(state: State<'_, AppState>, id: String) -> Result<(), ErrorResponse> {
    let db = state.db.lock().map_err(|e| ErrorResponse {
        code: "LOCK_ERROR".into(),
        message: e.to_string(),
    })?;
    db.delete_conversation(&id).map_err(|e| ErrorResponse::from(e))
}

#[tauri::command]
pub fn get_conversation_with_messages(
    state: State<'_, AppState>,
    id: String,
) -> Result<(Conversation, Vec<Message>), ErrorResponse> {
    let db = state.db.lock().map_err(|e| ErrorResponse {
        code: "LOCK_ERROR".into(),
        message: e.to_string(),
    })?;
    let conversation = db.get_conversation(&id).map_err(|e| ErrorResponse::from(e))?;
    let messages = db.list_messages(&id).map_err(|e| ErrorResponse::from(e))?;
    Ok((conversation, messages))
}
