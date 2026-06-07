# Skills 配置持久化指南

## 📋 概述

Skills 系统现在支持**完全的配置持久化**，包括：
1. **Skills 目录路径** - 可自定义存储位置
2. **IQS API Key** - 阿里云智能检索服务密钥
3. **自动加载** - 应用启动时自动从数据库读取配置

---

## 🔧 后端实现

### 1. 数据库层 (`src-tauri/src/db/settings.rs`)

#### Skills 目录配置

```rust
/// 获取 Skills 目录路径
pub fn get_skills_directory(&self) -> AppResult<String> {
    match self.get_setting("skills.directory")? {
        Some(dir) => Ok(dir),
        None => {
            // 默认路径：~/.cosurf/skills
            let default_dir = dirs::home_dir()
                .unwrap_or_else(|| std::env::temp_dir())
                .join(".cosurf")
                .join("skills")
                .to_string_lossy()
                .to_string();
            
            // 保存默认值
            self.set_setting("skills.directory", &default_dir)?;
            Ok(default_dir)
        }
    }
}

/// 设置 Skills 目录路径
pub fn set_skills_directory(&self, directory: &str) -> AppResult<()> {
    self.set_setting("skills.directory", directory)
}
```

**特点**：
- ✅ 首次使用时自动创建默认路径
- ✅ 支持自定义路径
- ✅ 持久化到 SQLite

---

#### IQS API Key 配置

```rust
/// 获取阿里云 IQS API Key
pub fn get_iqs_api_key(&self) -> AppResult<Option<String>> {
    self.get_setting("iqs.api_key")
}

/// 设置阿里云 IQS API Key
pub fn set_iqs_api_key(&self, api_key: &str) -> AppResult<()> {
    self.set_setting("iqs.api_key", api_key)
}
```

**特点**：
- ✅ 安全存储（SQLite）
- ✅ 支持空值
- ✅ 重启后保留

---

### 2. 命令层 (`src-tauri/src/commands/settings.rs`)

```rust
/// 获取 Skills 目录路径
#[tauri::command]
pub fn get_skills_directory(state: State<'_, AppState>) -> Result<String, ErrorResponse> {
    let db = state.db.lock().map_err(...)?;
    db.get_skills_directory().map_err(...)
}

/// 设置 Skills 目录路径
#[tauri::command]
pub fn set_skills_directory(
    state: State<'_, AppState>,
    directory: String,
) -> Result<(), ErrorResponse> {
    let db = state.db.lock().map_err(...)?;
    db.set_skills_directory(&directory).map_err(...)
}

/// 获取阿里云 IQS API Key
#[tauri::command]
pub fn get_iqs_api_key(state: State<'_, AppState>) -> Result<Option<String>, ErrorResponse> {
    let db = state.db.lock().map_err(...)?;
    db.get_iqs_api_key().map_err(...)
}

/// 设置阿里云 IQS API Key
#[tauri::command]
pub fn set_iqs_api_key(
    state: State<'_, AppState>,
    api_key: String,
) -> Result<(), ErrorResponse> {
    let db = state.db.lock().map_err(...)?;
    db.set_iqs_api_key(&api_key).map_err(...)
}
```

---

### 3. 状态初始化 (`src-tauri/src/state.rs`)

```rust
impl AppState {
    pub fn new(db: Database, app_data_dir: PathBuf) -> Self {
        // 从数据库获取 Skills 目录配置
        let skills_dir_str = db.get_skills_directory()
            .unwrap_or_else(|_| {
                app_data_dir.join("skills").to_string_lossy().to_string()
            });
        
        let skills_dir = PathBuf::from(&skills_dir_str);
        
        // 确保目录存在
        if !skills_dir.exists() {
            std::fs::create_dir_all(&skills_dir)?;
        }
        
        let mut skills_manager = SkillsManager::new(skills_dir.clone());
        
        // 加载已有的 Skills
        skills_manager.load_skills_from_directory()?;
        
        Self { 
            db: Mutex::new(db), 
            skills_manager: Arc::new(Mutex::new(skills_manager)),
            ...
        }
    }
}
```

**流程**：
1. 从数据库读取配置的目录路径
2. 如果不存在，使用默认路径并保存
3. 确保目录存在（自动创建）
4. 初始化 SkillsManager
5. 加载所有 Skills

---

### 4. MCP 执行器 API Key 处理 (`src-tauri/src/ai/skills_executors/mcp.rs`)

```rust
pub async fn execute_mcp_skill(skill: &Skill, arguments: &serde_json::Value) -> AppResult<String> {
    let mcp_config = skill.config.mcp.as_ref()...;
    
    // 处理 API Key（支持环境变量替换）
    let api_key = if let Some(ref key_template) = mcp_config.api_key {
        // 如果是以 ${} 格式，尝试从环境变量获取
        if key_template.starts_with("${") && key_template.ends_with('}') {
            let env_var = &key_template[2..key_template.len()-1];
            std::env::var(env_var).ok()
        } else {
            Some(key_template.clone())
        }
    } else {
        None
    };
    
    // TODO: 使用 api_key 进行 MCP 认证
    ...
}
```

**支持的格式**：
- `${ALIBABA_CLOUD_API_KEY}` - 从环境变量读取
- `sk-xxxxx` - 直接使用明文（不推荐）

---

## 💻 前端实现

### 1. Settings Store (`src-web/src/stores/settingsStore.ts`)

```typescript
interface SettingsState {
  // ... 其他字段
  
  // Skills 配置
  skillsDirectory: string;
  iqsApiKey: string;
  
  // 方法
  loadSkillsConfig: () => Promise<void>;
  setSkillsDirectory: (directory: string) => Promise<void>;
  setIqsApiKey: (apiKey: string) => Promise<void>;
}

export const useSettingsStore = create<SettingsState>((set) => ({
  // ... 其他状态
  
  skillsDirectory: "",
  iqsApiKey: "",
  
  // 加载 Skills 配置
  loadSkillsConfig: async () => {
    try {
      const skillsDir = await invoke<string>("get_skills_directory");
      const iqsKey = await invoke<string | null>("get_iqs_api_key");
      
      set({
        skillsDirectory: skillsDir,
        iqsApiKey: iqsKey || "",
      });
    } catch (error) {
      console.error("Failed to load skills config:", error);
    }
  },
  
  // 设置 Skills 目录
  setSkillsDirectory: async (directory) => {
    try {
      await invoke("set_skills_directory", { directory });
      set({ skillsDirectory: directory });
    } catch (error) {
      console.error("Failed to set skills directory:", error);
    }
  },
  
  // 设置 IQS API Key
  setIqsApiKey: async (apiKey) => {
    try {
      await invoke("set_iqs_api_key", { apiKey });
      set({ iqsApiKey: apiKey });
    } catch (error) {
      console.error("Failed to set IQS API key:", error);
    }
  },
}));
```

---

### 2. 使用示例

#### 在组件中加载配置

```tsx
import { useEffect } from 'react';
import { useSettingsStore } from '@/stores/settingsStore';

function SettingsPage() {
  const { 
    skillsDirectory, 
    iqsApiKey,
    loadSkillsConfig,
    setSkillsDirectory,
    setIqsApiKey 
  } = useSettingsStore();
  
  useEffect(() => {
    // 页面加载时读取配置
    loadSkillsConfig();
  }, []);
  
  return (
    <div>
      <h2>Skills 配置</h2>
      
      {/* Skills 目录 */}
      <div>
        <label>Skills 目录：</label>
        <input 
          value={skillsDirectory}
          onChange={(e) => setSkillsDirectory(e.target.value)}
          placeholder="~/.cosurf/skills"
        />
      </div>
      
      {/* IQS API Key */}
      <div>
        <label>阿里云 IQS API Key：</label>
        <input 
          type="password"
          value={iqsApiKey}
          onChange={(e) => setIqsApiKey(e.target.value)}
          placeholder="sk-xxxxx"
        />
      </div>
    </div>
  );
}
```

---

## 📊 数据流

```
用户操作
  ↓
前端调用 Tauri Command
  ↓
后端读取/写入 SQLite
  ↓
更新 AppState
  ↓
返回结果给前端
  ↓
更新 Zustand Store
  ↓
UI 重新渲染
```

---

## 🔐 安全性考虑

### IQS API Key 存储

**当前方案**：
- ✅ 存储在 SQLite 数据库中
- ✅ 不在代码中硬编码
- ⚠️ 明文存储（未来可加密）

**改进建议**：
1. **短期**：使用操作系统密钥链
   - Windows: Credential Manager
   - macOS: Keychain
   - Linux: Secret Service API

2. **长期**：集成专业密钥管理服务
   - HashiCorp Vault
   - AWS Secrets Manager
   - Azure Key Vault

---

### Skills 目录权限

**建议**：
```bash
# Linux/macOS
chmod 700 ~/.cosurf/skills

# Windows (PowerShell)
icacls $env:USERPROFILE\.cosurf\skills /grant:r "%USERNAME%:(OI)(CI)F"
```

---

## 🧪 测试

### 后端测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_skills_directory_persistence() {
        let db = Database::new_in_memory().unwrap();
        
        // 首次获取，应该创建默认值
        let dir1 = db.get_skills_directory().unwrap();
        assert!(!dir1.is_empty());
        
        // 设置新值
        db.set_skills_directory("/custom/path").unwrap();
        
        // 再次获取，应该返回新值
        let dir2 = db.get_skills_directory().unwrap();
        assert_eq!(dir2, "/custom/path");
    }
    
    #[test]
    fn test_iqs_api_key_persistence() {
        let db = Database::new_in_memory().unwrap();
        
        // 首次获取，应该为空
        let key1 = db.get_iqs_api_key().unwrap();
        assert!(key1.is_none());
        
        // 设置 API Key
        db.set_iqs_api_key("sk-test123").unwrap();
        
        // 再次获取，应该返回值
        let key2 = db.get_iqs_api_key().unwrap();
        assert_eq!(key2, Some("sk-test123".to_string()));
    }
}
```

### 前端测试

```typescript
import { renderHook } from '@testing-library/react';
import { useSettingsStore } from '@/stores/settingsStore';

describe('Settings Store', () => {
  it('should load skills config', async () => {
    const { result } = renderHook(() => useSettingsStore());
    
    await result.current.loadSkillsConfig();
    
    expect(result.current.skillsDirectory).not.toBe('');
  });
  
  it('should set skills directory', async () => {
    const { result } = renderHook(() => useSettingsStore());
    
    await result.current.setSkillsDirectory('/test/path');
    
    expect(result.current.skillsDirectory).toBe('/test/path');
  });
  
  it('should set IQS API key', async () => {
    const { result } = renderHook(() => useSettingsStore());
    
    await result.current.setIqsApiKey('sk-test');
    
    expect(result.current.iqsApiKey).toBe('sk-test');
  });
});
```

---

## 📝 迁移指南

### 从旧版本升级

如果您之前使用了硬编码的 Skills 目录或 API Key：

1. **备份现有配置**
   ```bash
   cp -r ~/.cosurf/skills ~/backup/skills
   ```

2. **启动新版本**
   - 系统会自动创建默认目录
   - 从数据库读取配置

3. **迁移 Skills**
   ```bash
   cp ~/backup/skills/* ~/.cosurf/skills/
   ```

4. **重新配置 API Key**
   - 在设置页面输入 IQS API Key
   - 或通过命令行：
     ```bash
     cosurf config set iqs.api_key sk-xxxxx
     ```

---

## ❓ 常见问题

### Q1: 如何更改 Skills 目录？

**A**: 
```typescript
// 前端
await invoke('set_skills_directory', { 
  directory: '/new/path/to/skills' 
});
```

或者在设置页面修改。

---

### Q2: API Key 丢失怎么办？

**A**: 
1. 检查 SQLite 数据库：
   ```sql
   SELECT value FROM settings WHERE key = 'iqs.api_key';
   ```

2. 如果为空，重新设置：
   ```typescript
   await invoke('set_iqs_api_key', { 
     apiKey: 'sk-new-key' 
   });
   ```

---

### Q3: 多个用户共享同一台电脑怎么办？

**A**: 
每个用户的 Skills 目录是独立的：
- User1: `~/.cosurf/skills`
- User2: `~/.cosurf/skills`

如果需要共享，可以设置相同的目录路径。

---

### Q4: 如何备份配置？

**A**: 
```bash
# 备份整个配置目录
tar -czf cosurf-backup.tar.gz ~/.cosurf/

# 或只备份 Skills
tar -czf skills-backup.tar.gz ~/.cosurf/skills/
```

---

### Q5: 环境变量和数据库哪个优先级高？

**A**: 
- **MCP Skill 配置**：优先使用环境变量（`${VAR}` 格式）
- **全局 IQS API Key**：使用数据库存储

这样可以灵活组合使用。

---

## 🚀 下一步优化

1. **加密存储** - 使用 OS 密钥链保护敏感信息
2. **配置同步** - 支持云端同步配置
3. **多环境支持** - dev/staging/prod 不同配置
4. **配置验证** - 检查目录权限、API Key 有效性
5. **导入/导出** - 一键备份和恢复配置

---

**最后更新**: 2026-05-23  
**版本**: 1.0.0
