use tauri::State;

use crate::db::bookmarks::{Bookmark, BookmarkFolder, CreateBookmarkRequest, CreateFolderRequest};
use crate::error::ErrorResponse;
use crate::state::AppState;

#[tauri::command]
pub fn list_bookmarks(
    state: State<'_, AppState>,
    folder_id: Option<String>,
) -> Result<Vec<Bookmark>, ErrorResponse> {
    let db = state.db.lock().map_err(|e| ErrorResponse {
        code: "LOCK_ERROR".into(),
        message: e.to_string(),
    })?;
    db.list_bookmarks(folder_id.as_deref())
        .map_err(|e| ErrorResponse::from(e))
}

#[tauri::command]
pub fn create_bookmark(
    state: State<'_, AppState>,
    request: CreateBookmarkRequest,
) -> Result<Bookmark, ErrorResponse> {
    let db = state.db.lock().map_err(|e| ErrorResponse {
        code: "LOCK_ERROR".into(),
        message: e.to_string(),
    })?;
    db.create_bookmark(&request).map_err(|e| ErrorResponse::from(e))
}

#[tauri::command]
pub fn delete_bookmark(state: State<'_, AppState>, id: String) -> Result<(), ErrorResponse> {
    let db = state.db.lock().map_err(|e| ErrorResponse {
        code: "LOCK_ERROR".into(),
        message: e.to_string(),
    })?;
    db.delete_bookmark(&id).map_err(|e| ErrorResponse::from(e))
}

#[tauri::command]
pub fn list_bookmark_folders(
    state: State<'_, AppState>,
    parent_id: Option<String>,
) -> Result<Vec<BookmarkFolder>, ErrorResponse> {
    let db = state.db.lock().map_err(|e| ErrorResponse {
        code: "LOCK_ERROR".into(),
        message: e.to_string(),
    })?;
    db.list_bookmark_folders(parent_id.as_deref())
        .map_err(|e| ErrorResponse::from(e))
}

#[tauri::command]
pub fn create_bookmark_folder(
    state: State<'_, AppState>,
    request: CreateFolderRequest,
) -> Result<BookmarkFolder, ErrorResponse> {
    let db = state.db.lock().map_err(|e| ErrorResponse {
        code: "LOCK_ERROR".into(),
        message: e.to_string(),
    })?;
    db.create_bookmark_folder(&request)
        .map_err(|e| ErrorResponse::from(e))
}

#[tauri::command]
pub fn delete_bookmark_folder(state: State<'_, AppState>, id: String) -> Result<(), ErrorResponse> {
    let db = state.db.lock().map_err(|e| ErrorResponse {
        code: "LOCK_ERROR".into(),
        message: e.to_string(),
    })?;
    db.delete_bookmark_folder(&id).map_err(|e| ErrorResponse::from(e))
}
