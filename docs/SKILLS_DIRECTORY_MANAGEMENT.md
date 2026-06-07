# Skills 目录管理优化指南

## 📋 概述

本次优化实现了 **统一的 Skills 目录管理**，所有 Skill Markdown 文件都保存到同一个可配置的目录中，并提供了完整的管理接口。

---

## ✨ 核心改进

### 1. **统一目录存储**

所有 Skill 文件（`.md` 和 `.json`）都存储在配置的目录中：

```
~/.cosurf/skills/           # 默认目录（可配置）
├── echo-skill.md          # Markdown 格式（推荐）
├── echo-skill.json        # JSON 格式（自动生成）
├── python-calculator.md
├── alibabacloud-iqs-search.md
└── ...
```

**优势**：
- ✅ 集中管理，易于备份
- ✅ 支持版本控制（Git）
- ✅ 便于分享和迁移

---

### 2. **优先加载 Markdown**

应用启动时自动从目录加载 Skills：

```rust
pub fn load_skills_from_directory(&mut self) -> AppResult<usize> {
    // 1. 优先加载 .md 文件
    if path.extension() == "md" {
        self.import_skill_from_markdown_file(...)?;
    }
    // 2. 如果没有对应的 .md，才加载 .json
    else if path.extension() == "json" && !md_exists {
        self.import_skill_from_file(...)?;
    }
}
```

**逻辑**：
- Markdown 是**权威来源**
- JSON 是**缓存/兼容**格式
- 避免重复加载

---

### 3. **导入即保存**

从任意位置导入 Markdown Skill 时，会自动复制到配置目录：

```typescript
// 前端调用
await invoke('import_skill_from_markdown_file', {
  filePath: '/tmp/my-skill.md'  // 任意位置
});

// 后端处理
// 1. 读取文件内容
// 2. 解析并验证
// 3. 保存到 ~/.cosurf/skills/my-skill.md  ← 自动复制
// 4. 同时生成 my-skill.json（兼容）
```

---

### 4. **完整的文件管理 API**

#### 列出所有 Skill 文件

```typescript
const files = await invoke<SkillFileInfo[]>('list_skill_files');

// 返回示例
[
  {
    path: "/home/user/.cosurf/skills/echo-skill.md",
    filename: "echo-skill.md",
    size: 1024,
    modified: 1716451200,  // Unix 时间戳
    isMarkdown: true
  },
  {
    path: "/home/user/.cosurf/skills/calculator.json",
    filename: "calculator.json",
    size: 2048,
    modified: 1716450000,
    isMarkdown: false
  }
]
```

#### 获取/设置目录

```typescript
// 获取当前目录
const dir = await invoke<string>('get_skills_directory');
console.log(dir);  // "~/.cosurf/skills"

// 设置新目录
await invoke('set_skills_directory', {
  directory: '/custom/skills/path'
});
```

---

## 📁 文件结构

### 重构后的目录布局

```
~/.cosurf/skills/
├── echo-skill.md              # Markdown 源文件
├── echo-skill.json            # JSON 缓存（自动生成）
│
├── python-calculator.md       # Python 脚本 Skill
├── python-calculator.json
│
├── alibabacloud-iqs-search.md # MCP Skill
├── alibabacloud-iqs-search.json
│
└── custom-script.md           # 用户自定义 Skill
    └── custom-script.json
```

**规则**：
- `.md` 文件：**手动编辑**，权威来源
- `.json` 文件：**自动生成**，用于快速加载

---

## 🔧 使用示例

### 1. 导入 Markdown Skill

#### 从文件导入

```typescript
// 选择文件对话框
const filePath = await openDialog();

// 导入（自动保存到配置目录）
const skill = await invoke('import_skill_from_markdown_file', {
  filePath
});

console.log(`✅ Imported: ${skill.name}`);
```

#### 从文本导入

```typescript
const markdownContent = `
---
id: my-skill
name: My Skill
description: A custom skill
type: cli
enabled: true
tags:
  - custom
---

# My Skill

## 配置

\`\`\`yaml
command: echo
args_template:
  - "Hello {{name}}!"
timeout: 5
\`\`\`

## 参数

| 参数名 | 类型 | 必填 | 描述 |
|--------|------|------|------|
| name | string | 否 | 名称 |
`;

const skill = await invoke('import_skill_from_markdown', {
  markdownContent
});
```

---

### 2. 列出所有 Skill 文件

```typescript
const files = await invoke('list_skill_files');

// 按类型分组
const mdFiles = files.filter(f => f.isMarkdown);
const jsonFiles = files.filter(f => !f.isMarkdown);

console.log(`Markdown: ${mdFiles.length}, JSON: ${jsonFiles.length}`);

// 显示最新修改的文件
files.forEach(file => {
  const date = new Date(file.modified! * 1000);
  console.log(`${file.filename} - ${date.toLocaleDateString()}`);
});
```

---

### 3. 更改 Skills 目录

```typescript
// 获取当前目录
const currentDir = await invoke('get_skills_directory');
console.log('Current:', currentDir);

// 设置新目录
const newDir = '/mnt/shared/skills';
await invoke('set_skills_directory', {
  directory: newDir
});

// 重启应用后生效
// 或者手动重新加载
await invoke('reload_skills');  // TODO: 实现此命令
```

---

### 4. 删除 Skill

```typescript
// 删除 Skill（同时删除 .md 和 .json）
await invoke('delete_skill', {
  skillId: 'echo-skill'
});

// 文件系统中的文件也会被删除
// ~/.cosurf/skills/echo-skill.md      ← 删除
// ~/.cosurf/skills/echo-skill.json    ← 删除
```

---

## 🎯 工作流程

### 导入流程

```
用户选择 Markdown 文件
         ↓
读取文件内容
         ↓
解析 YAML frontmatter
         ↓
提取配置块（YAML）
         ↓
提取参数定义（表格）
         ↓
验证 Skill 定义
         ↓
保存到内存（HashMap）
         ↓
保存到磁盘
  ├─ ~/.cosurf/skills/{id}.md   ← 原始 Markdown
  └─ ~/.cosurf/skills/{id}.json ← 解析后的 JSON
         ↓
返回 Skill 信息给前端
```

---

### 加载流程（应用启动）

```
AppState::new()
         ↓
从数据库读取配置的目录路径
  ├─ 有配置 → 使用配置的值
  └─ 无配置 → 使用默认值 (~/.cosurf/skills)
              ↓
          保存到数据库
         ↓
确保目录存在（自动创建）
         ↓
SkillsManager::load_skills_from_directory()
         ↓
遍历目录中的所有文件
         ↓
优先加载 .md 文件
  ├─ 解析 Markdown
  ├─ 提取配置
  └─ 保存到内存
         ↓
如果没有对应的 .md，加载 .json
         ↓
记录加载统计
  ┌──────────────────────┐
  │ Loaded: 5 skills     │
  │ Markdown: 4          │
  │ JSON: 1 (fallback)   │
  └──────────────────────┘
         ↓
应用正常运行
```

---

## 📊 数据流

### 文件同步策略

```
Markdown (.md)          JSON (.json)
     │                       │
     │  手动编辑              │  自动生成
     │                       │
     └───────┬───────────────┘
             │
             ↓
      哪个是权威来源？
             │
     ┌───────┴───────┐
     │               │
   .md 存在        .md 不存在
     │               │
     ↓               ↓
  使用 .md       使用 .json
  （忽略.json）   （向后兼容）
```

**原则**：
- Markdown 始终是**单一事实来源**
- JSON 仅作为**性能优化**和**向后兼容**

---

## 🔍 与之前版本的对比

### 之前

```
问题：
❌ Skill 文件散落在各处
❌ 没有统一的目录管理
❌ 导入后不知道文件保存在哪
❌ 无法列出所有 Skill 文件
❌ 重启后可能丢失配置
```

### 现在

```
改进：
✅ 所有文件统一到 ~/.cosurf/skills/
✅ 目录路径可配置且持久化
✅ 导入时自动复制到配置目录
✅ 提供 list_skill_files API
✅ 应用启动自动加载
✅ Markdown 作为权威来源
```

---

## 🛠️ 技术实现

### 1. SkillFileInfo 结构

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillFileInfo {
    /// 文件完整路径
    pub path: String,
    
    /// 文件名
    pub filename: String,
    
    /// 文件大小（字节）
    pub size: u64,
    
    /// 修改时间（Unix 时间戳）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified: Option<u64>,
    
    /// 是否为 Markdown 格式
    pub is_markdown: bool,
}
```

---

### 2. 列出文件实现

```rust
pub fn list_skill_files(&self) -> AppResult<Vec<SkillFileInfo>> {
    if !self.skills_dir.exists() {
        return Ok(Vec::new());
    }
    
    let mut files = Vec::new();
    
    for entry in std::fs::read_dir(&self.skills_dir)? {
        let path = entry.path();
        
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if ext == "md" || ext == "json" {
                let metadata = std::fs::metadata(&path)?;
                
                files.push(SkillFileInfo {
                    path: path.to_string_lossy().to_string(),
                    filename: path.file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string(),
                    size: metadata.len(),
                    modified: metadata.modified()
                        .ok()
                        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                        .map(|d| d.as_secs()),
                    is_markdown: ext == "md",
                });
            }
        }
    }
    
    // 按修改时间排序（最新的在前）
    files.sort_by(|a, b| b.modified.cmp(&a.modified));
    
    Ok(files)
}
```

---

### 3. 删除文件实现

```rust
pub fn delete_skill(&mut self, id: &str) -> AppResult<()> {
    if self.skills.remove(id).is_some() {
        // 删除 JSON 文件
        let json_path = self.skills_dir.join(format!("{}.json", id));
        if json_path.exists() {
            std::fs::remove_file(&json_path)?;
        }
        
        // 删除 Markdown 文件
        let md_path = self.skills_dir.join(format!("{}.md", id));
        if md_path.exists() {
            std::fs::remove_file(&md_path)?;
        }
        
        Ok(())
    } else {
        Err(AppError::NotFound(...))
    }
}
```

---

## 🧪 测试场景

### 场景 1: 首次启动

```bash
# 1. 启动应用
$ cosurf

# 2. 检查目录
$ ls ~/.cosurf/skills
# 应该自动创建空目录

# 3. 检查数据库
$ sqlite3 ~/.cosurf/app.db
sqlite> SELECT value FROM settings WHERE key = 'skills.directory';
# 应该返回默认路径
```

---

### 场景 2: 导入 Skill

```typescript
// 1. 准备测试文件
const testSkill = `
---
id: test-skill
name: Test
description: Test skill
type: cli
enabled: true
---

# Test

\`\`\`yaml
command: echo
args_template:
  - "Test"
timeout: 5
\`\`\`
`;

// 2. 导入
const skill = await invoke('import_skill_from_markdown', {
  markdownContent: testSkill
});

// 3. 验证文件存在
$ ls ~/.cosurf/skills/test-skill.*
# 应该看到 test-skill.md 和 test-skill.json

// 4. 列出文件
const files = await invoke('list_skill_files');
console.log(files.length);  // 应该 >= 1
```

---

### 场景 3: 更改目录

```typescript
// 1. 设置新目录
await invoke('set_skills_directory', {
  directory: '/tmp/custom-skills'
});

// 2. 重启应用
// 3. 验证新目录被使用
const dir = await invoke('get_skills_directory');
console.log(dir);  // "/tmp/custom-skills"

// 4. 导入 Skill
await invoke('import_skill_from_markdown', {
  markdownContent: testSkill
});

// 5. 验证文件在新目录
$ ls /tmp/custom-skills/
# 应该看到 test-skill.md
```

---

### 场景 4: 混合加载

```bash
# 1. 在目录中放置文件
$ cp echo-skill.md ~/.cosurf/skills/
$ cp old-skill.json ~/.cosurf/skills/

# 2. 启动应用
$ cosurf

# 3. 检查日志
INFO Loaded skills from directory { loaded=2, markdown=1 }
# markdown=1 表示只加载了 .md 文件
# .json 文件被跳过（因为没有对应的 .md）
```

---

## 💡 最佳实践

### 1. 使用 Git 管理 Skills

```bash
# 初始化 Git 仓库
cd ~/.cosurf/skills
git init

# 添加 .gitignore
echo "*.json" >> .gitignore  # 不提交自动生成的 JSON

# 提交 Skills
git add *.md
git commit -m "Add custom skills"

# 推送到远程
git remote add origin git@github.com:user/cosurf-skills.git
git push -u origin main
```

---

### 2. 定期备份

```bash
# 备份整个 Skills 目录
tar -czf skills-backup-$(date +%Y%m%d).tar.gz ~/.cosurf/skills/

# 或使用 rsync 增量备份
rsync -av ~/.cosurf/skills/ /backup/skills/
```

---

### 3. 团队协作

```bash
# 共享 Skills 目录
# 方案 1: Git 仓库
git clone git@github.com:team/cosurf-skills.git ~/.cosurf/skills

# 方案 2: NFS/SMB 共享挂载
mount -t nfs server:/shared/skills ~/.cosurf/skills

# 方案 3: 云存储同步
# Dropbox/OneDrive/iCloud 同步 ~/.cosurf/skills
```

---

### 4. 版本控制建议

```markdown
# 推荐的 .gitignore
*.json          # 自动生成的缓存
.DS_Store       # macOS
Thumbs.db       # Windows
node_modules/   # 如果有脚本依赖
```

---

## 🚀 未来优化方向

### 1. 热重载

```rust
// 监听目录变化
use notify::{Watcher, RecursiveMode};

let mut watcher = notify::recommended_watcher(|event| {
    match event {
        Ok(event) => {
            if event.kind.is_create() || event.kind.is_modify() {
                // 重新加载 Skill
                reload_skill(event.path);
            }
        }
        Err(e) => error!("Watch error: {}", e),
    }
})?;

watcher.watch(&skills_dir, RecursiveMode::NonRecursive)?;
```

---

### 2. 技能市场

```typescript
// 从在线市场安装 Skill
await invoke('install_skill_from_market', {
  skillId: 'web-search',
  version: '1.0.0'
});

// 更新 Skill
await invoke('update_skill', {
  skillId: 'web-search'
});
```

---

### 3. 分类管理

```rust
pub struct SkillCategory {
    pub name: String,
    pub description: String,
    pub skills: Vec<String>,  // Skill IDs
}

// 目录结构
~/.cosurf/skills/
├── web/
│   ├── search.md
│   └── screenshot.md
├── system/
│   ├── file-manager.md
│   └── process-monitor.md
└── ai/
    └── text-summarizer.md
```

---

### 4. 权限控制

```rust
pub struct SkillPermission {
    pub allow_network: bool,
    pub allow_filesystem: bool,
    pub allow_shell: bool,
    pub max_memory_mb: u64,
    pub timeout_secs: u64,
}

// 在执行前检查权限
if !skill.permission.allow_shell {
    return Err("Shell access not permitted");
}
```

---

## 📝 总结

### 核心特性

1. ✅ **统一目录** - 所有 Skill 文件集中管理
2. ✅ **可配置路径** - 支持自定义存储位置
3. ✅ **持久化** - 目录配置保存到数据库
4. ✅ **自动加载** - 应用启动时自动扫描
5. ✅ **优先 Markdown** - .md 作为权威来源
6. ✅ **完整 API** - 列出、导入、删除、配置

### 关键改进

- 📂 文件管理更清晰
- 🔄 导入流程更简单
- 💾 配置持久化可靠
- 📋 文件列表可视化
- 🔒 单一事实来源

---

**最后更新**: 2026-05-23  
**版本**: 1.1.0  
**作者**: CoSurf Team
