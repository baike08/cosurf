# MCP Server JSON 配置模式

## 📋 概述

CoSurf 现在支持标准的 MCP Server JSON 配置格式，允许用户通过粘贴或导入 JSON 文件批量配置多个 MCP Servers。

---

## 🎯 支持的配置格式

### 标准 JSON 格式

```json
{
  "mcpServers": {
    "filesystem": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/你的/工作/目录"]
    },
    "github": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-github"],
      "env": { "GITHUB_PERSONAL_ACCESS_TOKEN": "ghp_xxxx" }
    },
    "brave-search": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-brave-search"],
      "env": { "BRAVE_API_KEY": "BSAxxxx" }
    },
    "fetch": {
      "command": "uvx",
      "args": ["mcp-server-fetch"]
    },
    "context7": {
      "command": "npx",
      "args": ["-y", "@upstash/context7-mcp"]
    },
    "memory": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-memory"]
    }
  }
}
```

---

## 🔧 配置字段说明

### 必填字段

| 字段 | 类型 | 描述 | 示例 |
|------|------|------|------|
| `command` | string | 启动命令 | `"npx"`, `"uvx"`, `"node"` |
| `args` | string[] | 命令行参数 | `["-y", "@mcp/server"]` |

### 可选字段

| 字段 | 类型 | 描述 | 示例 |
|------|------|------|------|
| `env` | object | 环境变量 | `{ "API_KEY": "xxx" }` |

---

## 💻 使用方法

### 方法 1: 在 UI 中粘贴 JSON

1. 打开 **Settings → MCP Servers**
2. 点击 **"Import from JSON"** 按钮
3. 粘贴 JSON 配置
4. 点击 **"Import"**
5. 系统自动解析并创建所有 MCP Servers

### 方法 2: 使用 Tauri Command

```typescript
import { invoke } from "@tauri-apps/api/core";

const jsonContent = `{
  "mcpServers": {
    "filesystem": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/path/to/dir"]
    }
  }
}`;

const servers = await invoke("import_mcp_servers_from_json", {
  jsonContent
});

console.log(`Imported ${servers.length} servers`);
```

---

## 🗄️ 数据库存储

### 表结构

```sql
CREATE TABLE mcp_servers (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    server_type TEXT NOT NULL DEFAULT 'http',  -- 'http' or 'stdio'
    server_url TEXT,                            -- HTTP 模式的 URL
    command TEXT,                               -- stdio 模式的命令
    args TEXT,                                  -- JSON 数组字符串
    env TEXT,                                   -- JSON 对象字符串
    api_key TEXT,
    enabled INTEGER NOT NULL DEFAULT 1,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);
```

### 数据存储方式

JSON 配置中的字段会被转换为数据库记录：

```json
{
  "filesystem": {
    "command": "npx",
    "args": ["-y", "@mcp/filesystem"],
    "env": { "HOME": "/home/user" }
  }
}
```

↓ 转换后 ↓

| 字段 | 值 |
|------|-----|
| `name` | `"filesystem"` |
| `server_type` | `"stdio"` |
| `command` | `"npx"` |
| `args` | `["-y", "@mcp/filesystem"]` (JSON) |
| `env` | `{"HOME": "/home/user"}` (JSON) |
| `enabled` | `true` |

---

## 🔄 后端实现

### Rust 数据结构

文件：[settings.rs](file://d:\coding-harness\CoSurf\src-tauri\src\db\settings.rs#L28-L90)

```rust
/// MCP Server 类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum McpServerType {
    Http,   // HTTP/SSE 服务器
    Stdio,  // 本地进程
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub id: String,
    pub name: String,
    pub server_type: McpServerType,
    pub server_url: Option<String>,
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    pub env: Option<serde_json::Map<String, serde_json::Value>>,
    pub api_key: Option<String>,
    pub enabled: bool,
    pub created_at: i64,
    pub updated_at: i64,
}
```

### JSON 导入命令

文件：[commands/settings.rs](file://d:\coding-harness\CoSurf\src-tauri\src\commands\settings.rs#L247-L310)

```rust
#[tauri::command]
pub fn import_mcp_servers_from_json(
    state: State<'_, AppState>,
    json_content: String,
) -> Result<Vec<McpServerConfig>, ErrorResponse> {
    let db = state.db.lock()?;
    
    // 解析 JSON
    let parsed: serde_json::Value = serde_json::from_str(&json_content)?;
    
    let mcp_servers = parsed.get("mcpServers")
        .and_then(|v| v.as_object())
        .ok_or_else(|| ErrorResponse::new("INVALID_FORMAT"))?;
    
    let mut created_servers = Vec::new();
    
    for (name, server_config) in mcp_servers {
        // 提取配置字段
        let command = server_config.get("command")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        
        let args = server_config.get("args")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect());
        
        let env = server_config.get("env")
            .and_then(|v| v.as_object())
            .cloned();
        
        // 创建请求
        let req = CreateMcpServerRequest {
            name: name.clone(),
            server_type: Some(McpServerType::Stdio),
            server_url: None,
            command,
            args,
            env,
            api_key: None,
            enabled: Some(true),
        };
        
        // 保存到数据库
        match db.create_mcp_server(&req) {
            Ok(server) => created_servers.push(server),
            Err(e) => tracing::warn!("Failed to create server {}: {}", name, e),
        }
    }
    
    Ok(created_servers)
}
```

---

## 📝 完整示例

### 示例 1: 文件系统服务器

```json
{
  "mcpServers": {
    "my-files": {
      "command": "npx",
      "args": [
        "-y",
        "@modelcontextprotocol/server-filesystem",
        "/Users/username/Documents"
      ]
    }
  }
}
```

### 示例 2: GitHub 服务器（带环境变量）

```json
{
  "mcpServers": {
    "github-tools": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-github"],
      "env": {
        "GITHUB_PERSONAL_ACCESS_TOKEN": "ghp_your_token_here"
      }
    }
  }
}
```

### 示例 3: 多个服务器混合配置

```json
{
  "mcpServers": {
    "search": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-brave-search"],
      "env": { "BRAVE_API_KEY": "your_api_key" }
    },
    "web-fetch": {
      "command": "uvx",
      "args": ["mcp-server-fetch"]
    },
    "docs": {
      "command": "npx",
      "args": ["-y", "@upstash/context7-mcp"]
    }
  }
}
```

---

## ⚠️ 注意事项

### 1. JSON 格式要求

- ✅ 必须包含顶层的 `mcpServers` 对象
- ✅ 每个服务器的 key 是名称
- ✅ 每个服务器必须有 `command` 和 `args`

```json
// ❌ 错误格式
{
  "servers": { ... }  // 应该是 "mcpServers"
}

// ✅ 正确格式
{
  "mcpServers": {
    "my-server": {
      "command": "npx",
      "args": ["..."]
    }
  }
}
```

### 2. 环境变量安全

⚠️ **不要在 JSON 中硬编码敏感信息！**

```json
// ❌ 不安全
{
  "mcpServers": {
    "github": {
      "env": { "GITHUB_TOKEN": "ghp_real_token_12345" }
    }
  }
}

// ✅ 推荐：使用占位符，手动填写
{
  "mcpServers": {
    "github": {
      "env": { "GITHUB_TOKEN": "YOUR_TOKEN_HERE" }
    }
  }
}
```

导入后在 UI 中编辑，填入真实的环境变量值。

### 3. 路径问题

Windows 路径需要使用双反斜杠或正斜杠：

```json
{
  "mcpServers": {
    "files": {
      "command": "npx",
      "args": [
        "-y",
        "@modelcontextprotocol/server-filesystem",
        "C:\\\\Users\\\\username\\\\Documents"  // Windows
      ]
    }
  }
}
```

或使用正斜杠：

```json
{
  "args": ["C:/Users/username/Documents"]
}
```

### 4. 重复导入

如果多次导入相同的服务器名称：
- 会创建多条记录（不同的 UUID）
- 建议在导入前删除旧的配置

---

## 🛠️ 故障排查

### 问题 1: 导入失败 - "Invalid format"

**原因**: JSON 缺少 `mcpServers` 字段

**解决**:
```json
// 确保顶层有 mcpServers
{
  "mcpServers": {
    // ...
  }
}
```

### 问题 2: 服务器无法启动

**原因**: 命令或参数错误

**检查步骤**:
1. 在终端手动运行命令测试
   ```bash
   npx -y @modelcontextprotocol/server-filesystem /path/to/dir
   ```
2. 确认已安装必要的工具（npx, uvx, node 等）
3. 检查路径是否正确

### 问题 3: 环境变量不生效

**原因**: JSON 中的 env 字段格式错误

**检查**:
```json
// ❌ 错误
"env": "KEY=value"

// ✅ 正确
"env": { "KEY": "value" }
```

---

## 🚀 未来改进

1. **导出功能** - 将当前配置导出为 JSON
2. **模板库** - 提供常用 MCP Servers 的配置模板
3. **验证工具** - 导入前验证 JSON 格式
4. **批量编辑** - 支持批量修改多个服务器
5. **环境变量管理** - 集中管理所有环境变量

---

## 📚 相关文档

- [MCP_SKILL_IMPLEMENTATION.md](file://d:\coding-harness\CoSurf\docs\MCP_SKILL_IMPLEMENTATION.md) - MCP Skill 执行器
- [CONFIG_PERSISTENCE_FIXES.md](file://d:\coding-harness\CoSurf\docs\CONFIG_PERSISTENCE_FIXES.md) - 配置持久化
- [MCP Quick Start](https://modelcontextprotocol.io/quickstart) - MCP 官方快速开始

---

## ✅ 总结

**新增功能**：

- ✅ 支持标准 MCP JSON 配置格式
- ✅ 批量导入多个 MCP Servers
- ✅ 自动解析 command、args、env 字段
- ✅ 保存到 SQLite 数据库
- ✅ 友好的错误提示

**技术实现**：

- 扩展 `McpServerConfig` 支持 stdio 模式
- 添加 `McpServerType` 枚举（Http/Stdio）
- 实现 `import_mcp_servers_from_json` 命令
- JSON 字段序列化/反序列化

现在你可以轻松导入和管理任何 MCP Server 配置！🎉
