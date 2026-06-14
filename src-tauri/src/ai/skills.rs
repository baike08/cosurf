/// Skills 管理系统（渐进式加载架构）
///
/// 架构设计：
/// - 初始加载时仅解析 SKILL.md 的 frontmatter（name/description/tags）
/// - 模型选择 skill 后，懒加载完整 SKILL.md 内容作为 tool result 返回
/// - Agent Loop 根据 SKILL.md 内容继续决策调用 MCP tools/内置工具/脚本
///
/// 目录结构：
/// ```
/// skills/
/// ├── my-skill/
/// │   └── SKILL.md
/// └── another-skill/
///     └── SKILL.md
/// ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{info, warn, error};

use crate::error::{AppError, AppResult};

/// Skill 定义（简化版）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    /// Skill 唯一标识（目录名）
    pub id: String,
    /// Skill 名称
    pub name: String,
    /// Skill 描述
    pub description: String,
    /// 是否启用
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    /// 标签
    #[serde(default)]
    pub tags: Vec<String>,
    /// Skill 目录路径
    #[serde(skip)]
    pub dir_path: PathBuf,
    /// Markdown 原始内容（懒加载，不在初始加载时填充）
    #[serde(skip)]
    pub markdown_content: Option<String>,
}

fn default_enabled() -> bool {
    true
}

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

/// Skill 目录信息（用于前端显示）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDirInfo {
    /// Skill ID（目录名）
    pub id: String,
    /// Skill 名称
    pub name: String,
    /// Skill 描述
    pub description: String,
    /// 是否启用
    pub enabled: bool,
    /// 标签
    pub tags: Vec<String>,
    /// 目录完整路径
    pub dir_path: String,
    /// SKILL.md 文件大小（字节）
    pub file_size: u64,
    /// 修改时间（Unix 时间戳）
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

    /// 将项目示例 Skills 同步到 Skills 目录（目录结构格式）
    ///
    /// 期望 examples/skills/ 下是目录结构：
    /// examples/skills/{skill-name}/SKILL.md
    pub fn seed_example_skills(&mut self) -> AppResult<usize> {
        let examples_skills_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .map(|p| p.join("examples").join("skills"))
            .unwrap_or_else(|| PathBuf::from("examples/skills"));

        if !examples_skills_dir.exists() {
            info!("Examples skills directory not found: {:?}, skipping seed", examples_skills_dir);
            return Ok(0);
        }

        // 确保 skills 目录存在
        if !self.skills_dir.exists() {
            std::fs::create_dir_all(&self.skills_dir)
                .map_err(|e| AppError::Internal(format!("Failed to create skills directory: {}", e)))?;
        }

        let mut count = 0;
        for entry in std::fs::read_dir(&examples_skills_dir)
            .map_err(|e| AppError::Internal(format!("Failed to read examples skills dir: {}", e)))?
        {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    warn!("Failed to read entry: {}", e);
                    continue;
                }
            };
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let skill_md = path.join("SKILL.md");
            if !skill_md.exists() {
                warn!("No SKILL.md in {:?}, skipping", path);
                continue;
            }

            let dir_name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();

            let dest_dir = self.skills_dir.join(&dir_name);

            // 创建目标目录
            if let Err(e) = std::fs::create_dir_all(&dest_dir) {
                error!(dir = ?dest_dir, error = %e, "Failed to create skill directory");
                continue;
            }

            // 复制 SKILL.md（始终覆盖以确保是最新版本）
            let dest_md = dest_dir.join("SKILL.md");
            match std::fs::copy(&skill_md, &dest_md) {
                Ok(_) => {
                    info!(skill = %dir_name, "Seeded example skill");
                    count += 1;
                }
                Err(e) => {
                    error!(skill = %dir_name, error = %e, "Failed to seed example skill");
                }
            }
        }

        info!(seeded = count, "Seeded example skills");
        Ok(count)
    }

    /// 从 Skills 目录加载所有 Skills（仅解析 frontmatter，懒加载 body）
    pub fn load_skills_from_directory(&mut self) -> AppResult<usize> {
        info!(path = ?self.skills_dir, "Starting to load skills from directory");
        
        if !self.skills_dir.exists() {
            info!(path = ?self.skills_dir, "Skills directory does not exist, creating it");
            std::fs::create_dir_all(&self.skills_dir)
                .map_err(|e| AppError::Internal(format!("Failed to create skills directory: {}", e)))?;
            return Ok(0);
        }

        let mut count = 0;
        let mut total_dirs = 0;

        for entry in std::fs::read_dir(&self.skills_dir)
            .map_err(|e| AppError::Internal(format!("Failed to read skills directory: {}", e)))?
        {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    warn!("Failed to read directory entry: {}", e);
                    continue;
                }
            };
            let path = entry.path();

            // 只处理目录
            if !path.is_dir() {
                continue;
            }
            
            total_dirs += 1;
            info!(dir = ?path, "Found skill directory");

            let skill_md_path = path.join("SKILL.md");
            if !skill_md_path.exists() {
                warn!(dir = ?path, "No SKILL.md found in directory, skipping");
                continue;
            }
            
            info!(dir = ?path, "Found SKILL.md file");

            let dir_name = match path.file_name().and_then(|n| n.to_str()) {
                Some(name) => name.to_string(),
                None => continue,
            };

            match self.load_skill_from_dir(&dir_name, &path, &skill_md_path) {
                Ok(_) => count += 1,
                Err(e) => {
                    error!(skill = %dir_name, error = %e, "Failed to load skill from directory");
                }
            }
        }

        info!(loaded = count, total_dirs = total_dirs, path = ?self.skills_dir, "Finished loading skills from directory");
        Ok(count)
    }

    /// 从单个目录加载 Skill（仅解析 frontmatter）
    fn load_skill_from_dir(
        &mut self,
        id: &str,
        dir_path: &std::path::Path,
        skill_md_path: &std::path::Path,
    ) -> AppResult<()> {
        let content = std::fs::read_to_string(skill_md_path)
            .map_err(|e| AppError::Internal(format!("Failed to read SKILL.md: {}", e)))?;

        // 解析 frontmatter 获取元数据
        let (frontmatter, _body) = parse_markdown_sections(&content)?;
        let metadata: SkillMetadata = serde_yaml::from_str(&frontmatter)
            .map_err(|e| AppError::Internal(format!("Failed to parse SKILL.md frontmatter: {}", e)))?;

        let skill = Skill {
            id: id.to_string(),
            name: metadata.name,
            description: metadata.description,
            enabled: metadata.enabled,
            tags: metadata.tags,
            dir_path: dir_path.to_path_buf(),
            markdown_content: None, // 懒加载：初始不加载 body
        };

        info!(skill_id = %skill.id, skill_name = %skill.name, "Loaded skill metadata");
        self.skills.insert(id.to_string(), skill);
        Ok(())
    }

    /// 懒加载：读取指定 Skill 的完整 SKILL.md 内容
    pub fn load_skill_content(&self, skill_id: &str) -> AppResult<String> {
        let skill = self.skills.get(skill_id)
            .ok_or_else(|| AppError::Internal(format!("Skill not found: {}", skill_id)))?;

        let skill_md_path = skill.dir_path.join("SKILL.md");
        let content = std::fs::read_to_string(&skill_md_path)
            .map_err(|e| AppError::Internal(format!("Failed to read SKILL.md: {}", e)))?;

        info!(skill_id = %skill_id, bytes = content.len(), "Lazy loaded skill content");
        Ok(content)
    }

    /// 获取所有已启用的 Skills
    pub fn get_enabled_skills(&self) -> Vec<&Skill> {
        self.skills.values()
            .filter(|s| s.enabled)
            .collect()
    }

    /// 获取所有 Skills
    pub fn get_all_skills(&self) -> Vec<&Skill> {
        self.skills.values().collect()
    }

    /// 获取单个 Skill
    pub fn get_skill(&self, id: &str) -> Option<&Skill> {
        self.skills.get(id)
    }

    /// 获取 Skills 目录
    pub fn get_skills_dir(&self) -> &PathBuf {
        &self.skills_dir
    }

    /// 列出 Skills 目录信息（用于前端显示）
    pub fn list_skill_dirs(&self) -> AppResult<Vec<SkillDirInfo>> {
        if !self.skills_dir.exists() {
            return Ok(Vec::new());
        }

        let mut dirs = Vec::new();

        for entry in std::fs::read_dir(&self.skills_dir)
            .map_err(|e| AppError::Internal(format!("Failed to read skills directory: {}", e)))?
        {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let skill_md_path = path.join("SKILL.md");
            if !skill_md_path.exists() {
                continue;
            }

            let dir_name = match path.file_name().and_then(|n| n.to_str()) {
                Some(name) => name.to_string(),
                None => continue,
            };

            // 读取文件元信息
            let metadata = match std::fs::metadata(&skill_md_path) {
                Ok(m) => m,
                Err(_) => continue,
            };

            // 尝试从已加载的 skills 中获取信息
            let (name, description, enabled, tags) = if let Some(skill) = self.skills.get(&dir_name) {
                (skill.name.clone(), skill.description.clone(), skill.enabled, skill.tags.clone())
            } else {
                (dir_name.clone(), String::new(), true, Vec::new())
            };

            dirs.push(SkillDirInfo {
                id: dir_name,
                name,
                description,
                enabled,
                tags,
                dir_path: path.to_string_lossy().to_string(),
                file_size: metadata.len(),
                modified: metadata.modified()
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs()),
            });
        }

        // 按修改时间排序（最新的在前）
        dirs.sort_by(|a, b| b.modified.cmp(&a.modified));
        Ok(dirs)
    }

    /// 从文件夹导入 Skill（复制整个目录到 skills 目录）
    pub fn import_skill_from_directory(&mut self, source_dir: &str) -> AppResult<Skill> {
        let source_path = std::path::Path::new(source_dir);
        if !source_path.is_dir() {
            return Err(AppError::Internal(format!("Source is not a directory: {}", source_dir)));
        }

        let skill_md_path = source_path.join("SKILL.md");
        if !skill_md_path.exists() {
            return Err(AppError::Internal(
                format!("No SKILL.md found in directory: {}", source_dir)
            ));
        }

        // 读取 SKILL.md 内容
        let content = std::fs::read_to_string(&skill_md_path)
            .map_err(|e| AppError::Internal(format!("Failed to read SKILL.md: {}", e)))?;

        // 解析 frontmatter
        let (frontmatter, _body) = parse_markdown_sections(&content)?;
        let metadata: SkillMetadata = serde_yaml::from_str(&frontmatter)
            .map_err(|e| AppError::Internal(format!("Failed to parse frontmatter: {}", e)))?;

        // 源目录名作为 skill ID
        let dir_name = source_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(&metadata.name);
        let skill_id = to_kebab_case(dir_name);

        let dest_dir = self.skills_dir.join(&skill_id);

        info!(source = %source_dir, dest = ?dest_dir, skill_id = %skill_id, "Importing skill from directory");

        // 如果目标已存在，先删除
        if dest_dir.exists() {
            std::fs::remove_dir_all(&dest_dir)
                .map_err(|e| AppError::Internal(format!("Failed to remove existing skill directory: {}", e)))?;
        }

        // 复制整个目录
        copy_dir_recursive(source_path, &dest_dir)?;

        // 从目标目录重新加载
        self.load_skill_from_dir(&skill_id, &dest_dir, &dest_dir.join("SKILL.md"))?;

        let skill = self.skills.get(&skill_id)
            .ok_or_else(|| AppError::Internal(format!("Failed to load imported skill: {}", skill_id)))?
            .clone();

        info!(skill_id = %skill.id, skill_name = %skill.name, "Skill imported from directory");
        Ok(skill)
    }

    /// 从 Markdown 文本导入 Skill（创建目录结构）
    pub fn import_skill_from_markdown(&mut self, markdown_content: &str) -> AppResult<Skill> {
        info!("Importing skill from Markdown content");

        // 解析 frontmatter
        let (frontmatter, _body) = parse_markdown_sections(markdown_content)?;
        let metadata: SkillMetadata = serde_yaml::from_str(&frontmatter)
            .map_err(|e| AppError::Internal(format!("Failed to parse frontmatter: {}", e)))?;

        // 用 name 生成目录名（kebab-case）
        let skill_id = to_kebab_case(&metadata.name);
        let skill_dir = self.skills_dir.join(&skill_id);

        // 创建目录
        std::fs::create_dir_all(&skill_dir)
            .map_err(|e| AppError::Internal(format!("Failed to create skill directory: {}", e)))?;

        // 写入 SKILL.md
        let skill_md_path = skill_dir.join("SKILL.md");
        std::fs::write(&skill_md_path, markdown_content)
            .map_err(|e| AppError::Internal(format!("Failed to write SKILL.md: {}", e)))?;

        let skill = Skill {
            id: skill_id,
            name: metadata.name,
            description: metadata.description,
            enabled: metadata.enabled,
            tags: metadata.tags,
            dir_path: skill_dir,
            markdown_content: Some(markdown_content.to_string()),
        };

        info!(skill_id = %skill.id, skill_name = %skill.name, "Imported skill from Markdown");
        self.skills.insert(skill.id.clone(), skill.clone());
        Ok(skill)
    }

    /// 启用/禁用 Skill
    pub fn toggle_skill(&mut self, id: &str, enabled: bool) -> AppResult<()> {
        let skill = self.skills.get_mut(id)
            .ok_or_else(|| AppError::Internal(format!("Skill not found: {}", id)))?;

        info!(skill_id = %id, enabled, "Toggling skill");
        skill.enabled = enabled;

        // 更新 SKILL.md frontmatter 中的 enabled 字段
        self.update_skill_enabled_in_file(id)?;
        Ok(())
    }

    /// 删除 Skill（删除整个目录）
    pub fn delete_skill(&mut self, id: &str) -> AppResult<()> {
        let skill = self.skills.remove(id)
            .ok_or_else(|| AppError::Internal(format!("Skill not found: {}", id)))?;

        info!(skill_id = %id, "Deleting skill directory");

        // 删除整个目录
        if skill.dir_path.exists() {
            std::fs::remove_dir_all(&skill.dir_path)
                .map_err(|e| AppError::Internal(format!("Failed to delete skill directory: {}", e)))?;
        }

        Ok(())
    }

    /// 更新 SKILL.md 文件中的 enabled 字段
    fn update_skill_enabled_in_file(&self, id: &str) -> AppResult<()> {
        let skill = self.skills.get(id)
            .ok_or_else(|| AppError::Internal(format!("Skill not found: {}", id)))?;

        let skill_md_path = skill.dir_path.join("SKILL.md");
        let content = std::fs::read_to_string(&skill_md_path)
            .map_err(|e| AppError::Internal(format!("Failed to read SKILL.md: {}", e)))?;

        let (frontmatter, body) = parse_markdown_sections(&content)?;

        // 更新 frontmatter 中的 enabled 字段
        let mut updated_fm = String::new();
        let mut found_enabled = false;
        for line in frontmatter.lines() {
            if line.starts_with("enabled:") {
                updated_fm.push_str(&format!("enabled: {}\n", skill.enabled));
                found_enabled = true;
            } else {
                updated_fm.push_str(line);
                updated_fm.push('\n');
            }
        }
        if !found_enabled {
            updated_fm.push_str(&format!("enabled: {}\n", skill.enabled));
        }

        let new_content = format!("---\n{}---\n\n{}", updated_fm.trim(), body);
        std::fs::write(&skill_md_path, new_content)
            .map_err(|e| AppError::Internal(format!("Failed to update SKILL.md: {}", e)))?;

        Ok(())
    }

    /// 验证 Skill ID 是否合法（kebab-case）
    fn is_valid_skill_id(id: &str) -> bool {
        !id.is_empty()
            && id.chars().all(|c| c.is_alphanumeric() || c == '-')
    }
}

// ---- 工具函数 ----

/// 解析 Markdown 为 frontmatter 和 body
fn parse_markdown_sections(markdown: &str) -> AppResult<(String, String)> {
    let markdown = markdown.trim();

    if !markdown.starts_with("---") {
        return Err(AppError::Internal(
            "SKILL.md must start with --- frontmatter delimiter".to_string()
        ));
    }

    let rest = &markdown[3..];
    let end_pos = rest.find("---")
        .ok_or_else(|| AppError::Internal("Missing closing --- for frontmatter".to_string()))?;

    let frontmatter = rest[..end_pos].trim();
    let body = rest[end_pos + 3..].trim();

    Ok((frontmatter.to_string(), body.to_string()))
}

/// 将名称转换为 kebab-case（用作目录名）
fn to_kebab_case(name: &str) -> String {
    name.to_lowercase()
        .replace(|c: char| !c.is_alphanumeric() && c != '-', "-")
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// 递归复制目录
fn copy_dir_recursive(src: &std::path::Path, dst: &std::path::Path) -> AppResult<()> {
    std::fs::create_dir_all(dst)
        .map_err(|e| AppError::Internal(format!("Failed to create directory {:?}: {}", dst, e)))?;

    for entry in std::fs::read_dir(src)
        .map_err(|e| AppError::Internal(format!("Failed to read directory {:?}: {}", src, e)))?
    {
        let entry = entry
            .map_err(|e| AppError::Internal(format!("Failed to read directory entry: {}", e)))?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)
                .map_err(|e| AppError::Internal(
                    format!("Failed to copy {:?} to {:?}: {}", src_path, dst_path, e)
                ))?;
        }
    }

    Ok(())
}
