# CoSurf Skills 系统使用指南

## 概述

Skills 是 CoSurf 的可扩展能力系统，允许用户导入自定义技能来增强 AI 的能力。通过 Skills，AI 可以：

- 执行命令行工具（CLI）
- 运行脚本（Python、JavaScript、Bash、PowerShell）
- 调用 MCP (Model Context Protocol) 服务器
- 集成第三方服务和 API

## 快速开始

### 1. 导入第一个 Skill

**方法一：通过设置界面**

1. 打开 CoSurf
2. 点击左下角设置图标 ⚙️
3. 选择 "Skills" 标签页
4. 点击 "导入 Skill" 按钮
5. 粘贴以下 JSON：

```json
{
  "id": "echo-skill",
  "name": "Echo 消息",
  "description": "简单的回显技能，用于测试 Skills 系统",
  "type": "cli",
  "enabled": true,
  "tags": ["test", "demo"],
  "config": {
    "cli": {
      "command": "echo",
      "args_template": ["Hello from {{message}}!"],
      "timeout": 5,
      "require_confirmation": false
    },
    "parameters": {
      "type": "object",
      "properties": {
        "message": {
          "type": "string",
          "description": "要回显的消息",
          "default": "CoSurf"
        }
      }
    }
  }
}
```

6. 点击 "导入"

**方法二：从文件导入**

1. 在设置页面点击 "从文件导入"
2. 选择 `examples/echo-skill.json` 文件

**方法三：直接放置文件**

1. 将 JSON 文件复制到 `%APPDATA%\CoSurf\skills\` 目录
2. 重启 CoSurf

### 2. 测试 Skill

1. 在 Skills 列表中找到刚导入的 Skill
2. 点击右侧的测试按钮（代码图标）
3. 应该看到输出："Hello from CoSurf!"

## Skill 类型详解

### 1. CLI Skills

CLI Skills 允许 AI 执行命令行工具。

#### 示例：天气查询

```json
{
  "id": "weather-query",
  "name": "天气查询",
  "description": "查询指定城市的天气",
  "type": "cli",
  "enabled": true,
  "tags": ["weather", "api"],
  "config": {
    "cli": {
      "command": "curl",
      "args_template": [
        "-s",
        "https://api.openweathermap.org/data/2.5/weather?q={{city}}&appid={{api_key}}"
      ],
      "timeout": 10,
      "require_confirmation": false
    },
    "parameters": {
      "type": "object",
      "properties": {
        "city": {
          "type": "string",
          "description": "城市名称"
        },
        "api_key": {
          "type": "string",
          "description": "OpenWeatherMap API Key"
        }
      },
      "required": ["city", "api_key"]
    }
  }
}
```

#### 参数说明

- `command`: 要执行的命令
- `args_template`: 参数模板数组，支持 `{{param_name}}` 占位符
- `working_dir`: 工作目录（可选）
- `timeout`: 超时时间（秒），默认 30
- `require_confirmation`: 是否需要用户确认（安全特性）

### 2. Script Skills

Script Skills 允许 AI 执行脚本代码。

#### 支持的脚本语言

- **Python** (`python`)
- **JavaScript** (`javascript`) - 需要 Node.js
- **Bash** (`bash`) - Linux/macOS
- **PowerShell** (`powershell`) - Windows

#### 示例：数据处理

```json
{
  "id": "data-processor",
  "name": "数据处理器",
  "description": "处理 CSV 数据并生成统计信息",
  "type": "script",
  "enabled": true,
  "tags": ["python", "data", "csv"],
  "config": {
    "script": {
      "language": "python",
      "source": "import sys\nimport json\nimport csv\nfrom io import StringIO\n\nwith open(sys.argv[1], 'r') as f:\n    args = json.load(f)\n\ndata = args.get('csv_data', '')\nreader = csv.reader(StringIO(data))\nrows = list(reader)\n\nprint(f'Total rows: {len(rows)}')\nprint(f'Columns: {len(rows[0]) if rows else 0}')",
      "is_file": false
    },
    "parameters": {
      "type": "object",
      "properties": {
        "csv_data": {
          "type": "string",
          "description": "CSV 格式的数据"
        }
      },
      "required": ["csv_data"]
    }
  }
}
```

#### 脚本执行流程

1. 参数被写入临时 JSON 文件
2. 脚本作为第一个参数接收该文件路径
3. 脚本读取并解析参数
4. 脚本的输出被捕获并返回给 AI

### 3. MCP Skills

MCP (Model Context Protocol) Skills 允许连接到外部 MCP 服务器。

#### 示例：GitHub MCP

```json
{
  "id": "github-mcp",
  "name": "GitHub 工具",
  "description": "通过 MCP 访问 GitHub API",
  "type": "mcp",
  "enabled": true,
  "tags": ["github", "mcp", "api"],
  "config": {
    "mcp": {
      "server_url": "http://localhost:8080",
      "tool_name": "search_repositories",
      "api_key": null
    },
    "parameters": {
      "type": "object",
      "properties": {
        "query": {
          "type": "string",
          "description": "搜索查询"
        }
      },
      "required": ["query"]
    }
  }
}
```

## 高级用法

### 1. 参数插值

CLI Skills 支持参数模板中的变量替换：

```json
{
  "args_template": ["--url", "{{url}}", "--method", "{{method}}"]
}
```

当 AI 调用时传入 `{ "url": "https://example.com", "method": "GET" }`，最终执行的命令为：

```bash
command --url https://example.com --method GET
```

### 2. 安全确认

对于可能危险的命令，可以启用确认机制：

```json
{
  "cli": {
    "command": "rm",
    "args_template": ["-rf", "{{path}}"],
    "require_confirmation": true
  }
}
```

当 AI 尝试执行时，会提示用户确认。

### 3. 错误处理

所有 Skill 执行都会捕获 stdout 和 stderr：

- **成功**：返回 stdout 内容
- **失败**：返回 stderr 内容作为错误信息

### 4. 超时控制

防止长时间运行的任务阻塞系统：

```json
{
  "cli": {
    "command": "long_running_task",
    "timeout": 60
  }
}
```

超过 60 秒后会自动终止并返回超时错误。

## 最佳实践

### 1. Skill 设计原则

- **单一职责**：每个 Skill 只做一件事
- **清晰的描述**：详细描述 Skill 的功能和用途
- **合理的参数**：提供必要的参数，设置默认值
- **错误处理**：确保脚本能优雅地处理错误

### 2. 安全性考虑

- **验证输入**：在脚本中验证所有输入参数
- **限制权限**：避免使用需要管理员权限的命令
- **沙箱执行**：对于不受信任的脚本，在沙箱环境中运行
- **审计日志**：记录所有 Skill 执行情况

### 3. 性能优化

- **缓存结果**：对于重复查询，考虑缓存结果
- **异步执行**：对于耗时操作，使用异步模式
- **资源限制**：设置合理的超时和资源限制

## 故障排除

### 常见问题

#### 1. Skill 导入失败

**症状**：导入时显示错误信息

**解决**：
- 检查 JSON 格式是否正确
- 确保所有必需字段都存在
- 查看控制台日志获取详细错误

#### 2. CLI Skill 不执行

**症状**：执行时没有任何输出

**解决**：
- 确认命令在系统中可用（如 `python`, `curl`）
- 检查命令路径是否正确
- 查看后端日志

#### 3. 脚本执行超时

**症状**：显示 "timed out" 错误

**解决**：
- 增加 `timeout` 值
- 优化脚本性能
- 检查是否有死循环

#### 4. 参数未正确传递

**症状**：脚本收到的参数为空或错误

**解决**：
- 检查 `args_template` 中的占位符名称
- 确保参数名与 JSON Schema 匹配
- 添加日志调试参数传递

## 示例库

项目提供了多个示例 Skills（在 `examples/` 目录）：

1. **echo-skill.json** - 基础 CLI 测试
2. **python-calculator-skill.json** - Python 脚本示例
3. **更多示例即将添加...**

## 贡献 Skills

欢迎分享你创建的 Skills！

1. 创建 Skill JSON 文件
2. 添加详细的描述和使用说明
3. 提交到项目的 `examples/` 目录
4. 或者发布到社区论坛

## 未来计划

- [ ] Skill 市场（在线下载和分享）
- [ ] Skill 组合（将多个 Skills 串联）
- [ ] 可视化编辑器
- [ ] Skill 依赖管理
- [ ] 自动更新机制

## 相关资源

- [MCP 协议规范](https://modelcontextprotocol.io/)
- [Tauri 文档](https://tauri.app/)
- [Rust 异步编程](https://tokio.rs/)

---

如有问题或建议，请提交 Issue 或 Pull Request！
