# MCP Server 标准协议兼容配置

## 📋 概述

CoSurf 现已完全兼容开源 MCP (Model Context Protocol) 标准配置格式，支持 Claude Desktop、Cursor 等工具使用的标准 JSON 配置。

---

## 🎯 标准 MCP 配置格式

### 完整示例

```json
{
  "mcpServers": {
    "filesystem": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/Users/username/Documents"],
      "env": {
        "HOME": "/Users/username"
      },
      "cwd": "/Users/username",
      "disabled": false,
      "timeout": 60
    },
    "github": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-github"],
      "env": {
        "GITHUB_PERSONAL_ACCESS_TOKEN": "ghp_xxxx"
      }
    },
    "brave-search": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-brave-search"],
      "env": {
        "BRAVE_API_KEY": "BSAxxxx"
      }
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
    },
    "custom-http": {
      "url": "https://api.example.com/mcp"
    }
  }
}
```

---

## 🔧 配置字段详解

### stdio 模式字段（本地进程）

| 字段 | 类型 | 必填 | 描述 | 示例 |
|------|------|------|------|------|
| `command` | string | ✅ | 启动命令 | `"npx"`, `"node"`, `"python"`, `"uvx"` |
| `args` | string[] | ✅ | 命令行参数 | `["-y", "@mcp/server"]` |
| `cwd` | string | ❌ | 工作目录 | `"/path/to/dir"` |
| `env` | object | ❌ | 环境变量 | `{ "KEY": "value" }` |
| `disabled` | boolean | ❌ | 是否禁用（默认 false） | `false` |
| `timeout` | number | ❌ | 超时时间（秒） | `60` |

### HTTP 模式字段（远程服务器）

| 字段 | 类型 | 必填 | 描述 | 示例 |
|------|------|------|------|------|
| `url` | string | ✅ | HTTP/SSE 服务器 URL | `"https://api.example.com/mcp"` |

---

## 💻 CoSurf 数据结构

### Rust 结构体

文件：[settings.rs](file://d:\coding-harness\CoSurf\src-tauri\src\db\settings.rs#L43-L90)

```rust
pub enum McpServerType {
    Http,   // HTTP/SSE 服务器
    Stdio,  // 本地进程（通过 stdio）
}

pub struct McpServerConfig {
    pub id: String,
    pub name: String,
    pub server_type: McpServerType,
    
    // HTTP 模式字段
    pub url: Option<String>,
    
    // stdio 模式字段
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    pub cwd: Option<String>,
    
    // 通用字段
    pub env: Option<Map<String, Value>>,
    pub disabled: bool,
    pub timeout: Option<u64>,
    
    // 内部字段
    pub enabled: bool,
    pub created_at: i64,
    pub updated_at: i64,
}
```

### 数据库表结构

```sql
CREATE TABLE mcp_servers (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    server_type TEXT NOT NULL DEFAULT 'stdio',
    url TEXT,
    command TEXT,
    args TEXT,              -- JSON 数组字符串
    cwd TEXT,
    env TEXT,               -- JSON 对象字符串
    disabled INTEGER NOT NULL DEFAULT 0,
    timeout INTEGER,
    enabled INTEGER NOT NULL DEFAULT 1,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);
```

---

## 📝 配置示例详解

### 示例 1: 文件系统服务器

```json
{
  "mcpServers": {
    "my-documents": {
      "command": "npx",
      "args": [
        "-y",
        "@modelcontextprotocol/server-filesystem",
        "/Users/username/Documents"
      ],
      "cwd": "/Users/username",
      "env": {
        "HOME": "/Users/username"
      },
      "timeout": 120
    }
  }
}
```

**字段说明**：
- `command`: 使用 `npx` 运行 npm 包
- `args`: 传递包名和目录路径
- `cwd`: 设置工作目录为用户主目录
- `env`: 设置 HOME 环境变量
- `timeout`: 设置 120 秒超时

---

### 示例 2: GitHub 服务器（带认证）

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

**注意**：
- ⚠️ 不要将真实的 Token 提交到版本控制
- ✅ 导入后在 UI 中编辑，填入真实值

---

### 示例 3: Brave Search 服务器

```json
{
  "mcpServers": {
    "web-search": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-brave-search"],
      "env": {
        "BRAVE_API_KEY": "your_api_key_here"
      },
      "disabled": false
    }
  }
}
```

---

### 示例 4: Python 服务器（使用 uvx）

```json
{
  "mcpServers": {
    "python-tools": {
      "command": "uvx",
      "args": ["mcp-server-python"],
      "cwd": "/path/to/python/project"
    }
  }
}
```

---

### 示例 5: HTTP 服务器

```json
{
  "mcpServers": {
    "remote-mcp": {
      "url": "https://api.example.com/mcp"
    }
  }
}
```

---

### 示例 6: 混合配置（多个服务器）

```json
{
  "mcpServers": {
    "files": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/workspace"],
      "cwd": "/workspace"
    },
    "search": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-brave-search"],
      "env": { "BRAVE_API_KEY": "xxx" }
    },
    "git": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-git"],
      "cwd": "/path/to/repo"
    },
    "database": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-postgres"],
      "env": {
        "DATABASE_URL": "postgresql://localhost/mydb"
      }
    }
  }
}
```

---

## 🔄 与旧版本的兼容性

### 字段映射

| 旧字段 | 新字段 | 说明 |
|--------|--------|------|
| `server_url` | `url` | HTTP 服务器 URL |
| `api_key` | （移除） | 改用 `env` 存储认证信息 |
| - | `cwd` | 新增工作目录支持 |
| - | `disabled` | 新增禁用标志 |
| - | `timeout` | 新增超时配置 |

### 迁移指南

如果你之前使用了旧格式的配置，需要手动更新：

**旧格式**：
```json
{
  "mcpServers": {
    "my-server": {
      "server_url": "https://api.example.com",
      "api_key": "sk-xxx"
    }
  }
}
```

**新格式**：
```json
{
  "mcpServers": {
    "my-server": {
      "url": "https://api.example.com",
      "env": {
        "API_KEY": "sk-xxx"
      }
    }
  }
}
```

---

## 🛠️ 使用方法

### 方法 1: 在 UI 中导入 JSON

1. 打开 **Settings → MCP Servers**
2. 点击 **"Import from JSON"** 按钮
3. 粘贴标准格式的 JSON
4. 点击 **"Import"**
5. 系统自动解析并创建所有服务器

### 方法 2: 使用 Tauri Command

```typescript
import { invoke } from "@tauri-apps/api/core";

const jsonContent = `{
  "mcpServers": {
    "filesystem": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/path"]
    }
  }
}`;

const servers = await invoke("import_mcp_servers_from_json", {
  jsonContent
});

console.log(`Imported ${servers.length} servers`);
```

---

## ⚠️ 注意事项

### 1. 必填字段

**stdio 模式**必须有：
- ✅ `command`
- ✅ `args`

**HTTP 模式**必须有：
- ✅ `url`

```json
// ❌ 错误：缺少 command 和 args
{
  "mcpServers": {
    "bad-server": {}
  }
}

// ✅ 正确
{
  "mcpServers": {
    "good-server": {
      "command": "npx",
      "args": ["-y", "@mcp/server"]
    }
  }
}
```

### 2. 路径格式

**Windows 路径**：
```json
{
  "args": ["C:\\\\Users\\\\username\\\\Documents"]
}
```

或使用正斜杠：
```json
{
  "args": ["C:/Users/username/Documents"]
}
```

**Linux/Mac 路径**：
```json
{
  "args": ["/home/username/documents"]
}
```

### 3. 环境变量安全

⚠️ **不要在 JSON 中硬编码敏感信息！**

```json
// ❌ 不安全
{
  "env": { "API_KEY": "real_secret_key_12345" }
}

// ✅ 推荐：使用占位符
{
  "env": { "API_KEY": "YOUR_API_KEY_HERE" }
}
```

导入后在 UI 中编辑，填入真实值。

### 4. 命令可用性

确保系统中已安装所需的命令：

```bash
# 检查 npx
npx --version

# 检查 uvx
uvx --version

# 检查 node
node --version
```

---

## 🔍 故障排查

### 问题 1: 服务器无法启动

**症状**：导入成功，但服务器状态显示错误

**可能原因**：
1. 命令不存在
2. 参数错误
3. 工作目录不存在

**解决步骤**：
```bash
# 1. 手动测试命令
npx -y @modelcontextprotocol/server-filesystem /path/to/dir

# 2. 检查工作目录
ls -la /path/to/dir

# 3. 查看日志
# CoSurf 控制台输出
```

### 问题 2: 环境变量不生效

**症状**：服务器启动失败，提示缺少环境变量

**检查**：
```json
// ❌ 错误格式
"env": "KEY=value"

// ✅ 正确格式
"env": {
  "KEY": "value"
}
```

### 问题 3: 超时错误

**症状**：服务器响应超时

**解决**：增加 timeout 值
```json
{
  "timeout": 120
}
```

---

## 📊 与其他工具的对比

| 特性 | CoSurf | Claude Desktop | Cursor |
|------|--------|----------------|--------|
| stdio 模式 | ✅ | ✅ | ✅ |
| HTTP 模式 | ✅ | ✅ | ✅ |
| 环境变量 | ✅ | ✅ | ✅ |
| 工作目录 | ✅ | ✅ | ❌ |
| 超时配置 | ✅ | ❌ | ❌ |
| 禁用标志 | ✅ | ✅ | ❌ |
| 批量导入 | ✅ | ❌ | ❌ |
| UI 管理 | ✅ | ❌ | ❌ |

---

## 🚀 未来改进

1. **配置验证** - 导入前验证 JSON 格式和字段
2. **模板库** - 提供常用 MCP Servers 的配置模板
3. **导出功能** - 将当前配置导出为标准 JSON
4. **自动检测** - 检测系统中可用的命令
5. **健康检查** - 定期检查服务器状态

---

## 📚 参考资源

- [MCP 官方文档](https://modelcontextprotocol.io/)
- [Claude Desktop MCP 配置](https://claude.ai/download)
- [MCP Servers 仓库](https://github.com/modelcontextprotocol/servers)
- [CoSurf MCP 实现](file://d:\coding-harness\CoSurf\docs\MCP_SKILL_IMPLEMENTATION.md)

---

## ✅ 总结

**完全兼容的标准**：

- ✅ 支持 stdio 和 HTTP 两种模式
- ✅ 完整的字段映射（command, args, env, cwd, disabled, timeout）
- ✅ 批量导入标准 JSON 配置
- ✅ 与 Claude Desktop、Cursor 等工具兼容
- ✅ 额外的增强功能（超时、工作目录、UI 管理）

现在你可以无缝迁移任何标准的 MCP 配置到 CoSurf！🎉
