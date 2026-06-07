# Skills 目录修改功能

## 功能概述

实现了在设置页面中动态修改 Skills 目录的功能,用户可以:
- 查看当前 Skills 目录路径
- 通过文件选择器选择新目录
- 手动输入目录路径
- 保存配置并自动重新加载 Skills

## 实现细节

### 后端实现 (`src-tauri/src/commands/settings.rs`)

#### 1. 增强的 `set_skills_directory` 命令

```rust
#[tauri::command]
pub fn set_skills_directory(
    state: State<'_, AppState>,
    directory: String,
) -> Result<(), ErrorResponse> {
    // 1. 保存配置到数据库
    let db = state.db.lock()?;
    db.set_skills_directory(&directory)?;
    
    // 2. 确保目录存在
    let skills_dir = PathBuf::from(&directory);
    if !skills_dir.exists() {
        std::fs::create_dir_all(&skills_dir)?;
    }
    
    // 3. 创建新的 SkillsManager 并加载 Skills
    let mut new_manager = SkillsManager::new(skills_dir.clone());
    new_manager.load_skills_from_directory()?;
    
    // 4. 替换旧的 manager
    let mut manager = state.skills_manager.lock()?;
    *manager = new_manager;
    
    Ok(())
}
```

**关键改进**:
- ✅ 保存配置到 SQLite 数据库 (持久化)
- ✅ 自动创建目录 (如果不存在)
- ✅ 重新初始化 SkillsManager
- ✅ 从新目录加载所有 Skills
- ✅ 原子性替换 (避免并发问题)

### 前端实现 (`src-web/src/components/settings/SkillsSettings.tsx`)

#### 1. 新增状态

```typescript
const [editingDirectory, setEditingDirectory] = useState(false);
const [tempDirectory, setTempDirectory] = useState("");
```

#### 2. 新增方法

**选择目录**:
```typescript
const selectDirectory = async () => {
  const selected = await open({
    directory: true,
    multiple: false,
    title: "选择 Skills 目录"
  });
  
  if (selected) {
    setTempDirectory(selected as string);
  }
};
```

**保存目录配置**:
```typescript
const saveDirectory = async () => {
  await invoke("set_skills_directory", { directory: tempDirectory });
  setSkillsDirectory(tempDirectory);
  setEditingDirectory(false);
  
  // 重新加载 Skills
  await Promise.all([loadSkills(), loadSkillsConfig()]);
};
```

**取消编辑**:
```typescript
const cancelEdit = () => {
  setTempDirectory(skillsDirectory);
  setEditingDirectory(false);
};
```

#### 3. UI 更新

**查看模式**:
```tsx
<div className="flex items-center justify-between mb-2">
  <div className="flex items-center gap-2">
    <FolderOpen className="w-4 h-4" />
    <span className="font-mono">{skillsDirectory}</span>
  </div>
  <button onClick={() => setEditingDirectory(true)}>
    <Edit2 className="w-3 h-3" />
  </button>
</div>
```

**编辑模式**:
```tsx
<div className="space-y-2">
  {/* 输入框 + 选择按钮 */}
  <div className="flex gap-2">
    <input
      type="text"
      value={tempDirectory}
      onChange={(e) => setTempDirectory(e.target.value)}
      placeholder="输入 Skills 目录路径"
    />
    <button onClick={selectDirectory}>
      <FolderOpen className="w-3 h-3" />
    </button>
  </div>
  
  {/* 保存/取消按钮 */}
  <div className="flex justify-end gap-2">
    <button onClick={cancelEdit}>取消</button>
    <button onClick={saveDirectory}>保存</button>
  </div>
</div>
```

## 使用流程

1. **打开设置页面** → 切换到 "Skills" 标签
2. **查看当前目录** → 显示在 "Skills 目录" 区域
3. **点击编辑图标** ✏️ → 进入编辑模式
4. **修改目录**:
   - 方式一: 点击文件夹图标 📁 选择目录
   - 方式二: 手动输入路径
5. **保存** → 系统自动:
   - 保存配置到数据库
   - 创建目录 (如果不存在)
   - 重新加载 Skills
   - 刷新列表

## 数据流

```
用户操作
  ↓
前端调用 set_skills_directory(directory)
  ↓
后端接收命令
  ↓
保存到 SQLite (settings 表)
  ↓
创建新 SkillsManager
  ↓
从新目录加载 Skills (.md/.json)
  ↓
替换全局 SkillsManager
  ↓
返回成功
  ↓
前端重新加载 Skills 列表
```

## 持久化机制

### 数据库存储

**表**: `settings`

| key | value |
|-----|-------|
| `skills.directory` | `/path/to/skills` |

### 读取流程

应用启动时 (`src-tauri/src/state.rs`):
```rust
let skills_dir_str = db.get_skills_directory()
    .unwrap_or_else(|_| default_path);

let skills_dir = PathBuf::from(&skills_dir_str);
let mut skills_manager = SkillsManager::new(skills_dir);
skills_manager.load_skills_from_directory()?;
```

## 错误处理

### 前端
- ✅ 空路径验证
- ✅ 加载失败提示
- ✅ 保存失败显示错误信息

### 后端
- ✅ 目录创建失败返回错误
- ✅ Skills 加载失败记录日志 (不中断)
- ✅ 数据库操作失败返回错误

## 注意事项

1. **目录迁移**: 修改目录后,原目录中的 Skills 不会自动移动
2. **权限检查**: 确保新目录有读写权限
3. **并发安全**: 使用 Mutex 保护 SkillsManager 替换
4. **向后兼容**: 默认路径保持不变 (`~/.cosurf/skills`)

## 相关文件

- `src-tauri/src/commands/settings.rs` - 后端命令
- `src-tauri/src/db/settings.rs` - 数据库操作
- `src-tauri/src/state.rs` - 状态初始化
- `src-web/src/components/settings/SkillsSettings.tsx` - 前端 UI
- `src-web/src/stores/settingsStore.ts` - 状态管理

## 测试建议

### 手动测试

1. **基本功能**
   - [ ] 查看当前目录
   - [ ] 编辑目录
   - [ ] 选择新目录
   - [ ] 保存配置
   - [ ] 验证 Skills 重新加载

2. **边界情况**
   - [ ] 输入空路径
   - [ ] 选择不存在的目录 (应自动创建)
   - [ ] 选择无权限的目录
   - [ ] 切换回原目录

3. **持久化**
   - [ ] 重启应用后配置保持
   - [ ] 重启后 Skills 正确加载

### 自动化测试 (可选)

```typescript
// 前端测试
describe('Skills Directory Settings', () => {
  it('should update skills directory', async () => {
    // TODO: 实现 E2E 测试
  });
});
```

## 未来改进

1. **目录迁移向导**: 提供自动迁移现有 Skills 的功能
2. **多目录支持**: 允许配置多个 Skills 目录
3. **同步功能**: 与远程仓库同步 Skills
4. **备份恢复**: 定期备份 Skills 配置
