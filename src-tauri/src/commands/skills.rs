/// Skills 管理命令

use tauri::{AppHandle, State};
use tracing::info;

use crate::error::{AppError, AppResult};
use crate::state::AppState;
use serde::{Deserialize, Serialize};

/// Skill 信息（用于前端显示）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub tags: Vec<String>,
    /// Skill 目录路径
    pub dir_path: String,
}

impl From<&crate::ai::skills::Skill> for SkillInfo {
    fn from(skill: &crate::ai::skills::Skill) -> Self {
        Self {
            id: skill.id.clone(),
            name: skill.name.clone(),
            description: skill.description.clone(),
            enabled: skill.enabled,
            tags: skill.tags.clone(),
            dir_path: skill.dir_path.to_string_lossy().to_string(),
        }
    }
}

/// Toggle Skill 请求
#[derive(Debug, Clone, Deserialize)]
pub struct ToggleSkillRequest {
    pub skill_id: String,
    pub enabled: bool,
}

/// 获取所有 Skills
#[tauri::command]
pub async fn list_skills(
    state: State<'_, AppState>,
) -> AppResult<Vec<SkillInfo>> {
    info!("Listing all skills");

    let manager = state.skills_manager.lock()
        .map_err(|e| AppError::Internal(format!("Failed to lock skills manager: {}", e)))?;

    let skills = manager.get_all_skills()
        .iter()
        .map(|s| SkillInfo::from(*s))
        .collect();

    Ok(skills)
}

/// 删除 Skill
#[tauri::command]
pub async fn delete_skill(
    state: State<'_, AppState>,
    skill_id: String,
) -> AppResult<()> {
    info!(skill_id = %skill_id, "Deleting skill");

    let mut manager = state.skills_manager.lock()
        .map_err(|e| AppError::Internal(format!("Failed to lock skills manager: {}", e)))?;

    manager.delete_skill(&skill_id)?;

    Ok(())
}

/// 启用/禁用 Skill
#[tauri::command]
pub async fn toggle_skill(
    state: State<'_, AppState>,
    request: ToggleSkillRequest,
) -> AppResult<()> {
    info!(skill_id = %request.skill_id, enabled = request.enabled, "Toggling skill");

    let mut manager = state.skills_manager.lock()
        .map_err(|e| AppError::Internal(format!("Failed to lock skills manager: {}", e)))?;

    manager.toggle_skill(&request.skill_id, request.enabled)?;

    Ok(())
}

/// 从 Markdown 文本导入 Skill（创建目录结构）
#[tauri::command]
pub async fn import_skill_from_markdown(
    _app: AppHandle,
    state: State<'_, AppState>,
    markdown_content: String,
) -> AppResult<SkillInfo> {
    info!("Importing skill from Markdown content");

    let mut manager = state.skills_manager.lock()
        .map_err(|e| AppError::Internal(format!("Failed to lock skills manager: {}", e)))?;

    let skill = manager.import_skill_from_markdown(&markdown_content)?;

    Ok(SkillInfo::from(&skill))
}

/// 从文件夹导入 Skill（复制目录到 skills 目录）
#[tauri::command]
pub async fn import_skill_from_directory(
    _app: AppHandle,
    state: State<'_, AppState>,
    source_dir: String,
) -> AppResult<SkillInfo> {
    info!(source_dir = %source_dir, "Importing skill from directory");

    let mut manager = state.skills_manager.lock()
        .map_err(|e| AppError::Internal(format!("Failed to lock skills manager: {}", e)))?;

    let skill = manager.import_skill_from_directory(&source_dir)?;

    Ok(SkillInfo::from(&skill))
}

/// 列出所有 Skill 目录
#[tauri::command]
pub async fn list_skill_files(
    state: State<'_, AppState>,
) -> AppResult<Vec<crate::ai::SkillDirInfo>> {
    info!("Listing skill directories");

    let manager = state.skills_manager.lock()
        .map_err(|e| AppError::Internal(format!("Failed to lock skills manager: {}", e)))?;

    manager.list_skill_dirs()
}

/// 读取指定 Skill 的 SKILL.md 完整内容（用于前端预览）
#[tauri::command]
pub async fn get_skill_content(
    state: State<'_, AppState>,
    skill_id: String,
) -> AppResult<String> {
    info!(skill_id = %skill_id, "Getting skill content");

    let manager = state.skills_manager.lock()
        .map_err(|e| AppError::Internal(format!("Failed to lock skills manager: {}", e)))?;

    manager.load_skill_content(&skill_id)
}
