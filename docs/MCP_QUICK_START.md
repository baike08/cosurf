# MCP Skill 快速开始指南

## 🚀 5 分钟上手 MCP Skill

本指南将帮助你在 5 分钟内创建并运行第一个 MCP Skill。

---

## 步骤 1: 了解 MCP

**MCP (Model Context Protocol)** 是一个开源协议，允许 AI Agent 通过标准接口调用外部服务。

**核心概念**：
- **MCP Client** - CoSurf 中的执行器（已实现）
- **MCP Server** - 提供工具的外部服务
- **JSON-RPC 2.0** - 通信协议
- **Tools** - 可调用的功能

---

## 步骤 2: 创建你的第一个 MCP Skill

### 2.1 准备 Skill Markdown 文件

创建 `hello-mcp-skill.md`：

```markdown
---
id: hello-mcp
name: Hello MCP
description: 测试 MCP 连接
type: mcp
enabled: true
tags:
  - test
  - mcp
---

# Hello MCP

这是一个测试用的 MCP Skill，用于验证 MCP 连接是否正常。

## 配置

```yaml
server_url: https://your-mcp-server.com
tool_name: hello
api_key: ${TEST_API_KEY}
```

## 参数

| 参数名 | 类型 | 必填 | 描述 |
|--------|------|------|------|
| name | string | 否 | 你的名字 |

## 使用示例

```typescript
使用 hello-mcp，name="World"
```
```

### 2.2 导入 Skill

1. 打开 CoSurf
2. 进入 **Settings → Skills**
3. 点击 **Import Skill**
4. 粘贴上面的 Markdown 内容
5. 点击 **Save**

---

## 步骤 3: 设置环境变量

### Windows PowerShell

```powershell
$env:TEST_API_KEY="your-test-api-key"
```

### Linux/Mac

```bash
export TEST_API_KEY="your-test-api-key"
```

### 永久设置（推荐）

创建 `.env` 文件在项目根目录：

```env
TEST_API_KEY=your-test-api-key
```

---

## 步骤 4: 测试 MCP Server

在导入 Skill 之前，先测试你的 MCP Server 是否正常工作。

### 使用 curl 测试

```bash
curl -X POST https://your-mcp-server.com/message \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "initialize",
    "params": {
      "protocol_version": "2024-11-05",
      "capabilities": {},
      "client_info": {
        "name": "Test",
        "version": "1.0.0"
      }
    }
  }'
```

**期望响应**：

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "protocol_version": "2024-11-05",
    "capabilities": {
      "tools": {}
    },
    "server_info": {
      "name": "Your Server",
      "version": "1.0.0"
    }
  }
}
```

---

## 步骤 5: 在 Agent 中使用

打开 AI Panel，输入：

```
使用 hello-mcp，name="CoSurf"
```

**期望输出**：

```
Hello, CoSurf! Welcome to MCP.
```

---

## 🔍 调试技巧

### 1. 查看日志

启动应用后，在控制台查看日志：

```
INFO Executing MCP skill server_url=https://... tool_name=hello
INFO Initializing MCP connection server_url=https://...
INFO MCP connection initialized successfully
INFO Calling MCP tool tool_name=hello
INFO MCP tool call succeeded tool_name=hello
```

### 2. 常见问题

#### 问题 1: 认证失败

**错误**：`MCP error -32000: Unauthorized`

**解决**：
```bash
# 检查环境变量
echo $TEST_API_KEY  # Linux/Mac
echo $env:TEST_API_KEY  # Windows

# 确认格式正确
# ✅ 正确: api_key: ${TEST_API_KEY}
# ❌ 错误: api_key: TEST_API_KEY
```

#### 问题 2: 连接超时

**错误**：`Failed to send MCP request: operation timed out`

**解决**：
- 检查网络连接
- 确认服务器 URL 正确
- 测试服务器是否在线

#### 问题 3: 工具不存在

**错误**：`MCP error -32601: Method not found`

**解决**：
- 检查 `tool_name` 拼写
- 确认服务器支持该工具
- 查看服务器文档

---

## 📝 真实示例：阿里云 IQS 搜索

### 1. 创建 Skill

```markdown
---
id: alibabacloud-iqs-search
name: 阿里云 IQS 智能搜索
description: 使用阿里云智能查询服务进行实时网页搜索
type: mcp
enabled: true
tags:
  - search
  - web
---

# 阿里云 IQS 智能搜索

## 配置

```yaml
server_url: https://dashscope.aliyuncs.com/api/v1/services/search/unified
tool_name: web_search
api_key: ${ALIYUN_IQS_API_KEY}
```

## 参数

| 参数名 | 类型 | 必填 | 描述 |
|--------|------|------|------|
| query | string | 是 | 搜索查询词 |
| numResults | integer | 否 | 结果数量 (1-10) |
```

### 2. 设置 API Key

从 [阿里云控制台](https://dashscope.console.aliyun.com/) 获取 API Key：

```bash
export ALIYUN_IQS_API_KEY="sk-your-api-key-here"
```

### 3. 使用

```
使用 alibabacloud-iqs-search，query="AI 最新进展", numResults=5
```

---

## 🎯 最佳实践

### 1. Skill 命名规范

- ✅ `alibabacloud-iqs-search`
- ✅ `weather-realtime`
- ❌ `MySkill`
- ❌ `test_123`

### 2. 参数设计

```yaml
# ✅ 好的设计
query: string        # 清晰的参数名
numResults: integer  # 驼峰命名
freshness: string    # 有意义的名称

# ❌ 不好的设计
q: string            # 太简短
n: integer           # 不明确
x: string            # 无意义
```

### 3. 错误处理

在 Skill 描述中说明可能的错误：

```markdown
## 错误处理

- `401 Unauthorized`: API Key 无效
- `429 Too Many Requests`: 请求频率超限
- `Timeout`: 请求超时（30秒）
```

### 4. 文档完整性

每个 Skill 应包含：
- ✅ 清晰的描述
- ✅ 配置示例
- ✅ 参数表格
- ✅ 使用示例
- ✅ 错误说明

---

## 🔄 下一步

1. **探索更多示例**
   - [alibabacloud-iqs-search-skill.md](file://d:\coding-harness\CoSurf\examples\alibabacloud-iqs-search-skill.md)
   - [weather-search-skill.md](file://d:\coding-harness\CoSurf\examples\weather-search-skill.md)

2. **阅读完整文档**
   - [MCP_SKILL_IMPLEMENTATION.md](file://d:\coding-harness\CoSurf\docs\MCP_SKILL_IMPLEMENTATION.md)
   - [MCP_SKILL_SUMMARY.md](file://d:\coding-harness\CoSurf\docs\MCP_SKILL_SUMMARY.md)

3. **创建自己的 MCP Server**
   - [MCP Server SDK](https://github.com/modelcontextprotocol/typescript-sdk)
   - [Python SDK](https://github.com/modelcontextprotocol/python-sdk)

4. **集成现有服务**
   - Slack
   - GitHub
   - Notion
   - Google Calendar

---

## 💡 提示

- **从简单开始** - 先测试基本的 hello world
- **逐步复杂化** - 添加更多参数和功能
- **充分测试** - 确保每个参数都经过测试
- **写好文档** - 方便自己和他人使用

---

**祝你使用愉快！** 🎉
