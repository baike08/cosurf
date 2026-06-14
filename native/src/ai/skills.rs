//! Skills 管理系统
//!
//! 从 src-tauri/src/ai/skills.rs 迁移。
//! 目录扫描、SKILL.md 解析、懒加载。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{info, warn};

use crate::error::{AppError, AppResult};

/// Skill 定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(skip)]
    pub dir_path: PathBuf,
    #[serde(skip)]
    pub markdown_content: Option<String>,
}

fn default_enabled() -> bool { true }

/// Skill 元数据（从 SKILL.md frontmatter 解析）
#[derive(Debug, Clone, Deserialize)]
struct SkillMetadata {
    pub name: String,
    pub description: String,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Skill 目录信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDirInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub tags: Vec<String>,
    pub dir_path: String,
    pub file_size: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified: Option<u64>,
}

/// Skills 管理器
pub struct SkillsManager {
    skills: HashMap<String, Skill>,
    skills_dir: PathBuf,
}

impl SkillsManager {
    pub fn new(skills_dir: PathBuf) -> Self {
        info!(path = ?skills_dir, "Initializing SkillsManager");
        Self {
            skills: HashMap::new(),
            skills_dir,
        }
    }

    /// 更新 Skills 目录并重新加载
    pub fn set_skills_dir(&mut self, new_dir: PathBuf) -> AppResult<usize> {
        info!(old = ?self.skills_dir, new = ?new_dir, "Updating Skills directory");
        self.skills_dir = new_dir;
        self.skills.clear();
        self.load_skills()
    }

    /// 从目录加载所有 Skills（仅解析 frontmatter）
    pub fn load_skills(&mut self) -> AppResult<usize> {
        if !self.skills_dir.exists() {
            std::fs::create_dir_all(&self.skills_dir)
                .map_err(|e| AppError::Internal(format!("Failed to create skills directory: {}", e)))?;
            return Ok(0);
        }

        let mut count = 0;
        for entry in std::fs::read_dir(&self.skills_dir)
            .map_err(|e| AppError::Internal(format!("Failed to read skills directory: {}", e)))?
        {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => { warn!("Failed to read entry: {}", e); continue; }
            };
            let path = entry.path();
            if !path.is_dir() { continue; }

            let skill_md_path = path.join("SKILL.md");
            if !skill_md_path.exists() { continue; }

            let dir_name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();

            match self.parse_skill_frontmatter(&skill_md_path) {
                Ok(meta) => {
                    let skill = Skill {
                        id: dir_name.clone(),
                        name: meta.name,
                        description: meta.description,
                        enabled: meta.enabled,
                        tags: meta.tags,
                        dir_path: path.clone(),
                        markdown_content: None,
                    };
                    self.skills.insert(dir_name, skill);
                    count += 1;
                }
                Err(e) => {
                    warn!("Failed to parse skill {:?}: {}", path, e);
                }
            }
        }

        info!(loaded = count, "Skills loaded from directory");
        Ok(count)
    }

    /// 解析 SKILL.md 的 frontmatter
    fn parse_skill_frontmatter(&self, path: &std::path::Path) -> AppResult<SkillMetadata> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| AppError::Internal(format!("Failed to read SKILL.md: {}", e)))?;

        // 提取 YAML frontmatter（--- 分隔）
        let trimmed = content.trim();
        if !trimmed.starts_with("---") {
            return Ok(SkillMetadata {
                name: "Unnamed Skill".to_string(),
                description: "No description".to_string(),
                enabled: true,
                tags: vec![],
            });
        }

        let after_first = &trimmed[3..];
        let end_idx = after_first.find("---").unwrap_or(after_first.len());
        let yaml_str = after_first[..end_idx].trim();

        let meta: SkillMetadata = serde_yaml::from_str(yaml_str)
            .map_err(|e| AppError::Internal(format!("Failed to parse frontmatter YAML: {}", e)))?;

        Ok(meta)
    }

    /// 获取所有启用的 Skills
    pub fn get_enabled_skills(&self) -> Vec<&Skill> {
        self.skills.values().filter(|s| s.enabled).collect()
    }

    /// 获取 Skills 工具 Schema（用于 LLM）
    pub fn get_tool_schemas(&self) -> Vec<serde_json::Value> {
        self.get_enabled_skills()
            .iter()
            .map(|skill| {
                serde_json::json!({
                    "type": "function",
                    "function": {
                        "name": format!("skill_{}", skill.id),
                        "description": skill.description,
                        "parameters": {
                            "type": "object",
                            "properties": {},
                            "additionalProperties": true
                        }
                    }
                })
            })
            .collect()
    }

    /// 懒加载完整 SKILL.md 内容
    pub fn get_skill_content(&mut self, id: &str) -> AppResult<String> {
        let skill = self.skills.get_mut(id)
            .ok_or_else(|| AppError::NotFound(format!("Skill {} not found", id)))?;

        if let Some(ref content) = skill.markdown_content {
            return Ok(content.clone());
        }

        let skill_md_path = skill.dir_path.join("SKILL.md");
        let content = std::fs::read_to_string(&skill_md_path)
            .map_err(|e| AppError::Internal(format!("Failed to read SKILL.md: {}", e)))?;

        skill.markdown_content = Some(content.clone());
        Ok(content)
    }

    /// 列出所有 Skills
    pub fn list_skills(&self) -> Vec<SkillDirInfo> {
        self.skills.values().map(|skill| {
            let skill_md_path = skill.dir_path.join("SKILL.md");
            let (file_size, modified) = if let Ok(meta) = std::fs::metadata(&skill_md_path) {
                let modified = meta.modified().ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs());
                (meta.len(), modified)
            } else {
                (0, None)
            };

            SkillDirInfo {
                id: skill.id.clone(),
                name: skill.name.clone(),
                description: skill.description.clone(),
                enabled: skill.enabled,
                tags: skill.tags.clone(),
                dir_path: skill.dir_path.to_string_lossy().to_string(),
                file_size,
                modified,
            }
        }).collect()
    }

    /// 删除 Skill
    pub fn delete_skill(&mut self, id: &str) -> AppResult<()> {
        let skill = self.skills.remove(id)
            .ok_or_else(|| AppError::NotFound(format!("Skill {} not found", id)))?;

        if skill.dir_path.exists() {
            std::fs::remove_dir_all(&skill.dir_path)
                .map_err(|e| AppError::Internal(format!("Failed to delete skill directory: {}", e)))?;
        }

        Ok(())
    }

    /// 切换 Skill 启用状态
    pub fn toggle_skill(&mut self, id: &str) -> AppResult<bool> {
        let skill = self.skills.get_mut(id)
            .ok_or_else(|| AppError::NotFound(format!("Skill {} not found", id)))?;

        skill.enabled = !skill.enabled;
        Ok(skill.enabled)
    }

    /// 导入 Markdown 文件作为 Skill
    pub fn import_markdown(&mut self, content: &str) -> AppResult<SkillDirInfo> {
        // 解析 frontmatter 获取名称
        let meta = self.parse_skill_frontmatter_from_str(content)?;
        let id = meta.name.to_lowercase().replace(' ', "-");
        let dir_path = self.skills_dir.join(&id);

        std::fs::create_dir_all(&dir_path)
            .map_err(|e| AppError::Internal(format!("Failed to create skill directory: {}", e)))?;

        let skill_md_path = dir_path.join("SKILL.md");
        std::fs::write(&skill_md_path, content)
            .map_err(|e| AppError::Internal(format!("Failed to write SKILL.md: {}", e)))?;

        let skill = Skill {
            id: id.clone(),
            name: meta.name.clone(),
            description: meta.description.clone(),
            enabled: meta.enabled,
            tags: meta.tags.clone(),
            dir_path: dir_path.clone(),
            markdown_content: Some(content.to_string()),
        };
        self.skills.insert(id.clone(), skill);

        Ok(SkillDirInfo {
            id,
            name: meta.name,
            description: meta.description,
            enabled: meta.enabled,
            tags: meta.tags,
            dir_path: dir_path.to_string_lossy().to_string(),
            file_size: content.len() as u64,
            modified: None,
        })
    }

    fn parse_skill_frontmatter_from_str(&self, content: &str) -> AppResult<SkillMetadata> {
        let trimmed = content.trim();
        if !trimmed.starts_with("---") {
            return Ok(SkillMetadata {
                name: "Imported Skill".to_string(),
                description: "No description".to_string(),
                enabled: true,
                tags: vec![],
            });
        }

        let after_first = &trimmed[3..];
        let end_idx = after_first.find("---").unwrap_or(after_first.len());
        let yaml_str = after_first[..end_idx].trim();

        let meta: SkillMetadata = serde_yaml::from_str(yaml_str)
            .map_err(|e| AppError::Internal(format!("Failed to parse frontmatter: {}", e)))?;

        Ok(meta)
    }

    /// 导入整个目录
    pub fn import_directory(&mut self, dir_path: &str) -> AppResult<usize> {
        let source_dir = PathBuf::from(dir_path);
        if !source_dir.exists() {
            return Err(AppError::NotFound(format!("Directory not found: {}", dir_path)));
        }

        let mut count = 0;
        for entry in std::fs::read_dir(&source_dir)
            .map_err(|e| AppError::Internal(format!("Failed to read directory: {}", e)))?
        {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };
            let path = entry.path();
            if !path.is_dir() { continue; }

            let skill_md = path.join("SKILL.md");
            if !skill_md.exists() { continue; }

            let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
            let dest_dir = self.skills_dir.join(&dir_name);

            if std::fs::create_dir_all(&dest_dir).is_err() { continue; }
            if std::fs::copy(&skill_md, dest_dir.join("SKILL.md")).is_ok() {
                count += 1;
            }
        }

        // 重新加载
        self.load_skills()?;
        Ok(count)
    }

    /// 列出 Skill 目录中的文件
    /// 当 id 为空时，列出 skills 根目录下的所有子目录（即已安装的 skills）
    pub fn list_skill_files(&self, id: &str) -> AppResult<Vec<String>> {
        // 空 id = 列出 skills 根目录
        if id.is_empty() {
            let mut dirs = Vec::new();
            if let Ok(entries) = std::fs::read_dir(&self.skills_dir) {
                for entry in entries {
                    if let Ok(e) = entry {
                        if e.path().is_dir() {
                            if let Some(name) = e.file_name().to_str() {
                                dirs.push(name.to_string());
                            }
                        }
                    }
                }
            }
            return Ok(dirs);
        }

        let skill = self.skills.get(id)
            .ok_or_else(|| AppError::NotFound(format!("Skill {} not found", id)))?;

        let mut files = Vec::new();
        for entry in std::fs::read_dir(&skill.dir_path)
            .map_err(|e| AppError::Internal(format!("Failed to read skill directory: {}", e)))?
        {
            if let Ok(e) = entry {
                if let Some(name) = e.file_name().to_str() {
                    files.push(name.to_string());
                }
            }
        }

        Ok(files)
    }

    /// 获取 Skills 目录路径
    pub fn skills_dir(&self) -> &std::path::Path {
        &self.skills_dir
    }
}
