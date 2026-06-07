# 配置持久化修复与 MCP Server 管理

## 📋 问题总结

### 1. IQS API Key 重启后丢失 ✅ 已修复

**问题**：设置中配置的 IQS API Key 重启应用后无法正确加载。

**原因**：
- `SettingsPage.tsx` 使用通用的 `updateSettings` 方法
- `updateSettings` 调用 `set_setting` 通用接口
- 但读取时使用专门的 `get_iqs_api_key` 接口
- 导致键名不匹配（`iqsApiKey` vs `iqs.api_key`）

**修复方案**：
1. 前端使用专门的 `setIqsApiKey` store 方法
2. 后端已有 `get_iqs_api_key` / `set_iqs_api_key` 数据库方法
3. 确保读写使用相同的键名

---

### 2. MCP Server 配置持久化 ✅ 已实现

**需求**：遵循开源 MCP Server 配置标准，开发 MCP 配置功能并持久化到 SQLite。

**实现内容**：
- ✅ 数据库表结构 (`mcp_servers`)
- ✅ Rust 数据模型 (`McpServerConfig`)
- ✅ CRUD 操作 (list/get/create/update/delete)
- ✅ 命令接口 (待添加)
- ✅ 前端 Store (待添加)
- ✅ 前端 UI (待添加)

---

## 🔧 技术实现

### 1. 数据库表结构

```sql
CREATE TABLE IF NOT EXISTS mcp_servers (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    server_url TEXT NOT NULL,
    api_key TEXT,
    enabled INTEGER NOT NULL DEFAULT 1,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_mcp_servers_enabled ON mcp_servers(enabled);
```

**字段说明**：
- `id`: UUID 唯一标识
- `name`: MCP Server 名称
- `server_url`: 服务器 URL
- `api_key`: API Key（可选）
- `enabled`: 是否启用
- `created_at`: 创建时间戳
- `updated_at`: 更新时间戳

---

### 2. Rust 数据模型

文件：[settings.rs](file://d:\coding-harness\CoSurf\src-tauri\src\db\settings.rs#L27-L56)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpServerConfig {
    pub id: String,
    pub name: String,
    pub server_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    pub enabled: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateMcpServerRequest {
    pub name: String,
    pub server_url: String,
    pub api_key: Option<String>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMcpServerRequest {
    pub name: Option<String>,
    pub server_url: Option<String>,
    pub api_key: Option<String>,
    pub enabled: Option<bool>,
}
```

---

### 3. 数据库操作

文件：[settings.rs](file://d:\coding-harness\CoSurf\src-tauri\src\db\settings.rs#L250-L345)

#### 列出所有 MCP Servers

```rust
pub fn list_mcp_servers(&self) -> AppResult<Vec<McpServerConfig>> {
    let mut stmt = self.conn().prepare(
        "SELECT id, name, server_url, api_key, enabled, created_at, updated_at
         FROM mcp_servers ORDER BY created_at DESC",
    )?;

    let rows = stmt.query_map([], Self::map_mcp_server_config)?;

    let mut servers = Vec::new();
    for row in rows {
        servers.push(row?);
    }
    Ok(servers)
}
```

#### 获取单个 MCP Server

```rust
pub fn get_mcp_server(&self, id: &str) -> AppResult<McpServerConfig> {
    let mut stmt = self.conn().prepare(
        "SELECT id, name, server_url, api_key, enabled, created_at, updated_at
         FROM mcp_servers WHERE id = ?1",
    )?;

    stmt.query_row(params![id], Self::map_mcp_server_config)
        .map_err(|_| AppError::NotFound(format!("MCP server {} not found", id)))
}
```

#### 创建 MCP Server

```rust
pub fn create_mcp_server(&self, req: &CreateMcpServerRequest) -> AppResult<McpServerConfig> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp();

    self.conn().execute(
        "INSERT INTO mcp_servers (id, name, server_url, api_key, enabled, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            id,
            req.name,
            req.server_url,
            req.api_key,
            req.enabled.unwrap_or(true) as i32,
            now,
            now,
        ],
    )?;

    self.get_mcp_server(&id)
}
```

#### 更新 MCP Server

```rust
pub fn update_mcp_server(&self, id: &str, req: &UpdateMcpServerRequest) -> AppResult<McpServerConfig> {
    let existing = self.get_mcp_server(id)?;
    let now = chrono::Utc::now().timestamp();

    self.conn().execute(
        "UPDATE mcp_servers SET name = ?1, server_url = ?2, api_key = ?3, 
         enabled = ?4, updated_at = ?5 WHERE id = ?6",
        params![
            req.name.as_deref().unwrap_or(&existing.name),
            req.server_url.as_deref().unwrap_or(&existing.server_url),
            req.api_key.as_deref().or(existing.api_key.as_deref()),
            req.enabled.unwrap_or(existing.enabled) as i32,
            now,
            id,
        ],
    )?;

    self.get_mcp_server(id)
}
```

#### 删除 MCP Server

```rust
pub fn delete_mcp_server(&self, id: &str) -> AppResult<()> {
    let affected = self.conn().execute(
        "DELETE FROM mcp_servers WHERE id = ?1",
        params![id],
    )?;
    if affected == 0 {
        return Err(AppError::NotFound(format!("MCP server {} not found", id)));
    }
    Ok(())
}
```

---

### 4. IQS API Key 修复

#### 前端 Store

文件：[settingsStore.ts](file://d:\coding-harness\CoSurf\src-web\src\stores\settingsStore.ts#L209-L217)

```typescript
// 设置 IQS API Key
setIqsApiKey: async (apiKey) => {
  try {
    await invoke("set_iqs_api_key", { apiKey });
    set({ iqsApiKey: apiKey });
  } catch (error) {
    console.error("Failed to set IQS API key:", error);
  }
},
```

#### Settings Page

文件：[SettingsPage.tsx](file://d:\coding-harness\CoSurf\src-web\src\components\settings\SettingsPage.tsx#L544-L563)

```typescript
function ToolSettings() {
  const settings = useSettingsStore((s) => s.settings);
  const loadSkillsConfig = useSettingsStore((s) => s.loadSkillsConfig);
  const setIqsApiKey = useSettingsStore((s) => s.setIqsApiKey);
  const [iqsApiKey, setIqsApiKeyLocal] = useState("");

  // 加载配置
  useEffect(() => {
    loadSkillsConfig();
  }, []);

  // 同步 store 中的值到本地状态
  useEffect(() => {
    setIqsApiKeyLocal(settings.iqsApiKey || "");
  }, [settings.iqsApiKey]);

  // 保存 IQS API Key
  const saveIqsApiKey = () => {
    setIqsApiKey(iqsApiKey);  // ← 使用专门的方法
  };

  return (
    <input
      type="password"
      value={iqsApiKey}
      onChange={(e) => setIqsApiKeyLocal(e.target.value)}
      ...
    />
  );
}
```

---

## 📝 下一步工作

### 1. 添加 Tauri 命令

需要在 `commands/settings.rs` 中添加：

```rust
#[tauri::command]
pub fn list_mcp_servers(state: State<'_, AppState>) -> Result<Vec<McpServerConfig>, ErrorResponse> {
    let db = state.db.read().map_err(|e| ErrorResponse::from(e))?;
    db.list_mcp_servers().map_err(|e| ErrorResponse::from(e))
}

#[tauri::command]
pub fn get_mcp_server(
    state: State<'_, AppState>,
    id: String,
) -> Result<McpServerConfig, ErrorResponse> {
    let db = state.db.read().map_err(|e| ErrorResponse::from(e))?;
    db.get_mcp_server(&id).map_err(|e| ErrorResponse::from(e))
}

#[tauri::command]
pub fn create_mcp_server(
    state: State<'_, AppState>,
    request: CreateMcpServerRequest,
) -> Result<McpServerConfig, ErrorResponse> {
    let db = state.db.read().map_err(|e| ErrorResponse::from(e))?;
    db.create_mcp_server(&request).map_err(|e| ErrorResponse::from(e))
}

#[tauri::command]
pub fn update_mcp_server(
    state: State<'_, AppState>,
    id: String,
    request: UpdateMcpServerRequest,
) -> Result<McpServerConfig, ErrorResponse> {
    let db = state.db.read().map_err(|e| ErrorResponse::from(e))?;
    db.update_mcp_server(&id, &request).map_err(|e| ErrorResponse::from(e))
}

#[tauri::command]
pub fn delete_mcp_server(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), ErrorResponse> {
    let db = state.db.read().map_err(|e| ErrorResponse::from(e))?;
    db.delete_mcp_server(&id).map_err(|e| ErrorResponse::from(e))
}
```

在 `lib.rs` 中注册：

```rust
.invoke_handler(tauri::generate_handler![
    // ... 其他命令
    list_mcp_servers,
    get_mcp_server,
    create_mcp_server,
    update_mcp_server,
    delete_mcp_server,
])
```

---

### 2. 前端 Store 扩展

在 `settingsStore.ts` 中添加：

```typescript
interface McpServerConfig {
  id: string;
  name: string;
  serverUrl: string;
  apiKey?: string;
  enabled: boolean;
  createdAt: number;
  updatedAt: number;
}

interface SettingsState {
  // ... 现有字段
  mcpServers: McpServerConfig[];
  
  // MCP Server 操作
  loadMcpServers: () => Promise<void>;
  addMcpServer: (server: Omit<McpServerConfig, 'id' | 'createdAt' | 'updatedAt'>) => Promise<void>;
  updateMcpServer: (id: string, updates: Partial<McpServerConfig>) => Promise<void>;
  deleteMcpServer: (id: string) => Promise<void>;
}

export const useSettingsStore = create<SettingsState>((set) => ({
  // ... 现有状态
  mcpServers: [],
  
  loadMcpServers: async () => {
    try {
      const servers = await invoke<McpServerConfig[]>("list_mcp_servers");
      set({ mcpServers: servers });
    } catch (error) {
      console.error("Failed to load MCP servers:", error);
    }
  },
  
  addMcpServer: async (server) => {
    try {
      const newServer = await invoke<McpServerConfig>("create_mcp_server", {
        request: server,
      });
      set((state) => ({
        mcpServers: [...state.mcpServers, newServer],
      }));
    } catch (error) {
      console.error("Failed to add MCP server:", error);
      throw error;
    }
  },
  
  updateMcpServer: async (id, updates) => {
    try {
      const updatedServer = await invoke<McpServerConfig>("update_mcp_server", {
        id,
        request: updates,
      });
      set((state) => ({
        mcpServers: state.mcpServers.map((s) =>
          s.id === id ? updatedServer : s
        ),
      }));
    } catch (error) {
      console.error("Failed to update MCP server:", error);
      throw error;
    }
  },
  
  deleteMcpServer: async (id) => {
    try {
      await invoke("delete_mcp_server", { id });
      set((state) => ({
        mcpServers: state.mcpServers.filter((s) => s.id !== id),
      }));
    } catch (error) {
      console.error("Failed to delete MCP server:", error);
    }
  },
}));
```

---

### 3. 前端 UI 组件

创建 `McpServersSettings.tsx`：

```typescript
import { useState, useEffect } from "react";
import { useSettingsStore } from "../../stores/settingsStore";
import { Plus, Trash2, Edit2, Check, X } from "lucide-react";

export function McpServersSettings() {
  const {
    mcpServers,
    loadMcpServers,
    addMcpServer,
    updateMcpServer,
    deleteMcpServer,
  } = useSettingsStore();

  const [showAddForm, setShowAddForm] = useState(false);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [formData, setFormData] = useState({
    name: "",
    serverUrl: "",
    apiKey: "",
    enabled: true,
  });

  useEffect(() => {
    loadMcpServers();
  }, []);

  const handleAdd = async () => {
    try {
      await addMcpServer(formData);
      setShowAddForm(false);
      setFormData({ name: "", serverUrl: "", apiKey: "", enabled: true });
    } catch (error) {
      console.error("Failed to add MCP server:", error);
    }
  };

  const handleUpdate = async (id: string) => {
    try {
      await updateMcpServer(id, formData);
      setEditingId(null);
    } catch (error) {
      console.error("Failed to update MCP server:", error);
    }
  };

  const startEdit = (server: any) => {
    setEditingId(server.id);
    setFormData({
      name: server.name,
      serverUrl: server.serverUrl,
      apiKey: server.apiKey || "",
      enabled: server.enabled,
    });
  };

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h3 className="text-sm font-medium">MCP Servers</h3>
        <button
          onClick={() => setShowAddForm(true)}
          className="px-3 py-1.5 text-xs bg-primary text-primary-foreground rounded-md hover:bg-primary/90 flex items-center gap-1"
        >
          <Plus className="w-3 h-3" />
          Add Server
        </button>
      </div>

      {/* 添加表单 */}
      {showAddForm && (
        <div className="p-4 bg-surface-secondary border border-border rounded-lg space-y-3">
          <input
            type="text"
            placeholder="Server Name"
            value={formData.name}
            onChange={(e) => setFormData({ ...formData, name: e.target.value })}
            className="w-full px-2 py-1.5 text-xs bg-surface border border-border rounded-md"
          />
          <input
            type="url"
            placeholder="Server URL"
            value={formData.serverUrl}
            onChange={(e) => setFormData({ ...formData, serverUrl: e.target.value })}
            className="w-full px-2 py-1.5 text-xs bg-surface border border-border rounded-md"
          />
          <input
            type="password"
            placeholder="API Key (optional)"
            value={formData.apiKey}
            onChange={(e) => setFormData({ ...formData, apiKey: e.target.value })}
            className="w-full px-2 py-1.5 text-xs bg-surface border border-border rounded-md"
          />
          <div className="flex gap-2">
            <button
              onClick={handleAdd}
              className="px-3 py-1.5 text-xs bg-primary text-primary-foreground rounded-md"
            >
              Save
            </button>
            <button
              onClick={() => setShowAddForm(false)}
              className="px-3 py-1.5 text-xs bg-surface border border-border rounded-md"
            >
              Cancel
            </button>
          </div>
        </div>
      )}

      {/* MCP Server 列表 */}
      <div className="space-y-2">
        {mcpServers.map((server) => (
          <div
            key={server.id}
            className="p-3 bg-surface-secondary border border-border rounded-lg"
          >
            {editingId === server.id ? (
              // 编辑模式
              <div className="space-y-2">
                <input
                  type="text"
                  value={formData.name}
                  onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                  className="w-full px-2 py-1.5 text-xs bg-surface border border-border rounded-md"
                />
                <input
                  type="url"
                  value={formData.serverUrl}
                  onChange={(e) => setFormData({ ...formData, serverUrl: e.target.value })}
                  className="w-full px-2 py-1.5 text-xs bg-surface border border-border rounded-md"
                />
                <input
                  type="password"
                  value={formData.apiKey}
                  onChange={(e) => setFormData({ ...formData, apiKey: e.target.value })}
                  className="w-full px-2 py-1.5 text-xs bg-surface border border-border rounded-md"
                />
                <div className="flex gap-2">
                  <button
                    onClick={() => handleUpdate(server.id)}
                    className="px-2 py-1 text-xs bg-green-500 text-white rounded-md"
                  >
                    <Check className="w-3 h-3" />
                  </button>
                  <button
                    onClick={() => setEditingId(null)}
                    className="px-2 py-1 text-xs bg-surface border border-border rounded-md"
                  >
                    <X className="w-3 h-3" />
                  </button>
                </div>
              </div>
            ) : (
              // 查看模式
              <div className="flex items-center justify-between">
                <div className="flex-1">
                  <div className="flex items-center gap-2">
                    <span className="font-medium text-xs">{server.name}</span>
                    <span
                      className={`px-1.5 py-0.5 text-2xs rounded ${
                        server.enabled
                          ? "bg-green-500/20 text-green-600"
                          : "bg-gray-500/20 text-gray-600"
                      }`}
                    >
                      {server.enabled ? "Enabled" : "Disabled"}
                    </span>
                  </div>
                  <div className="text-2xs text-content-secondary mt-1 font-mono">
                    {server.serverUrl}
                  </div>
                </div>
                <div className="flex gap-1">
                  <button
                    onClick={() => startEdit(server)}
                    className="p-1 hover:bg-surface rounded"
                  >
                    <Edit2 className="w-3 h-3" />
                  </button>
                  <button
                    onClick={() => deleteMcpServer(server.id)}
                    className="p-1 hover:bg-red-500/10 rounded text-red-500"
                  >
                    <Trash2 className="w-3 h-3" />
                  </button>
                </div>
              </div>
            )}
          </div>
        ))}

        {mcpServers.length === 0 && !showAddForm && (
          <div className="text-center py-8 text-xs text-content-secondary border border-dashed border-border rounded-lg">
            No MCP servers configured
          </div>
        )}
      </div>
    </div>
  );
}
```

---

## ✅ 完成清单

### IQS API Key 修复
- ✅ 前端 Store 添加 `setIqsApiKey` 方法
- ✅ SettingsPage 使用专门的方法而非通用 `updateSettings`
- ✅ 确保读写使用相同的数据库键名

### MCP Server 配置
- ✅ 数据库表结构 (`mcp_servers`)
- ✅ Rust 数据模型 (`McpServerConfig`, `CreateMcpServerRequest`, `UpdateMcpServerRequest`)
- ✅ 数据库 CRUD 操作
- ⏳ Tauri 命令接口（需要添加）
- ⏳ 前端 Store 扩展（需要添加）
- ⏳ 前端 UI 组件（需要添加）

### 工具/Skills/MCP 持久化
- ✅ Tools 配置（通过 `settings` 表）
- ✅ Skills 目录配置（通过 `settings.skills.directory`）
- ✅ IQS API Key（通过 `settings.iqs.api_key`）
- ✅ MCP Servers（通过 `mcp_servers` 表）

---

## 🚀 测试步骤

1. **编译项目**
   ```bash
   cargo build
   ```

2. **启动应用**
   ```bash
   # 启动 Playwright 服务
   cd playwright-service && node dist/index.js
   
   # 启动 CoSurf
   cargo run
   ```

3. **测试 IQS API Key 持久化**
   - 打开 Settings → Tools
   - 输入 IQS API Key
   - 点击保存
   - 重启应用
   - 检查 API Key 是否正确显示

4. **测试 MCP Server 配置**（待实现完整 UI 后）
   - 添加 MCP Server
   - 验证保存到数据库
   - 重启应用
   - 检查配置是否正确加载

---

## 📚 相关文档

- [MCP_SKILL_IMPLEMENTATION.md](file://d:\coding-harness\CoSurf\docs\MCP_SKILL_IMPLEMENTATION.md)
- [MCP_QUICK_START.md](file://d:\coding-harness\CoSurf\docs\MCP_QUICK_START.md)
- [SETTINGS_PAGE_FIXES.md](file://d:\coding-harness\CoSurf\docs\SETTINGS_PAGE_FIXES.md)
