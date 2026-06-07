# CoSurf Skills 系统 - Markdown 格式指南

## 📋 概述

CoSurf Skills 系统现在支持 **Markdown 格式**的 Skill 定义，相比之前的 JSON 格式，具有以下优势：

- ✅ **人类可读**：Markdown 格式更易阅读和编辑
- ✅ **结构化**：Frontmatter + YAML + Markdown 表格的组合
- ✅ **文档化**：可以在同一个文件中包含详细说明、示例和注意事项
- ✅ **版本控制友好**：Git diff 更清晰

---

## 📝 Skill 文件格式

### 基本结构

```markdown
---
id: skill-id
name: Skill 名称
description: 简短描述
type: cli | script | mcp | built_in
enabled: true
tags:
  - tag1
  - tag2
---

# Skill 标题

详细说明...

## 配置

```yaml
# YAML 配置块
command: echo
args_template:
  - "Hello {{message}}!"
timeout: 5
```

## 参数

| 参数名 | 类型 | 必填 | 默认值 | 描述 |
|--------|------|------|--------|------|
| message | string | 否 | World | 消息内容 |

## 使用示例

```bash
使用 skill-id，message="CoSurf"
```

## 注意事项

- 注意事项 1
- 注意事项 2
```

---

## 🔧 Skill 类型

### 1. CLI Skills

执行命令行工具。

**示例**：[echo-skill.md](../examples/echo-skill.md)

```yaml
command: echo
args_template:
  - "Hello from {{message}}!"
timeout: 5
require_confirmation: false
```

**参数替换**：
- 使用 `{{param_name}}` 语法
- 支持字符串、数字、布尔值

---

### 2. Script Skills

执行 Python、JavaScript、Bash 或 PowerShell 脚本。

**示例**：[python-calculator-skill.md](../examples/python-calculator-skill.md)

```yaml
language: python
source: |
  import sys
  import json
  
  args_file = sys.argv[1]
  with open(args_file, 'r') as f:
      params = json.load(f)
  
  # 你的代码...
  print(json.dumps(result))
is_file: false
timeout: 10
```

**支持的語言**：
- `python` / `python3`
- `javascript` / `js`
- `bash` / `sh`
- `powershell` / `ps1`

---

### 3. MCP Skills

连接到 Model Context Protocol (MCP) 服务器。

**示例**：[alibabacloud-iqs-search-skill.md](../examples/alibabacloud-iqs-search-skill.md)

```yaml
server_url: https://iqs.cn-hangzhou.aliyuncs.com
tool_name: search_documents
api_key: ${ALIBABA_CLOUD_API_KEY}
timeout: 30
```

**环境变量**：
- 使用 `${VAR_NAME}` 语法引用环境变量
- API Key 等敏感信息应通过环境变量传递

---

### 4. Built-in Skills

内置技能，由 CoSurf 核心模块处理。

```yaml
# 通常不需要额外配置
```

---

## 📤 导入 Skills

### 方法 1：从文件导入（推荐）

```typescript
import { invoke } from '@tauri-apps/api/core';

// 从 Markdown 文件导入
const skill = await invoke('import_skill_from_markdown_file', {
  filePath: '/path/to/skill.md'
});

console.log('Imported skill:', skill);
```

### 方法 2：从文本导入

```typescript
const markdownContent = `
---
id: my-skill
name: My Skill
description: A custom skill
type: cli
enabled: true
tags:
  - demo
---

# My Skill

## 配置

\`\`\`yaml
command: echo
args_template:
  - "Hello!"
\`\`\`

## 参数

| 参数名 | 类型 | 必填 | 默认值 | 描述 |
|--------|------|------|--------|------|
`;

const skill = await invoke('import_skill_from_markdown', {
  markdownContent
});
```

### 方法 3：批量导入

```typescript
const skills = await invoke('import_skills_batch', {
  request: {
    skillsJson: [
      // JSON 格式的 Skills（向后兼容）
    ]
  }
});
```

---

## ▶️ 执行 Skills

```typescript
const result = await invoke('execute_skill', {
  request: {
    skillId: 'echo-skill',
    arguments: {
      message: 'CoSurf'
    }
  }
});

console.log('Result:', result);
// Output: Hello from CoSurf!
```

---

## 🗂️ 管理 Skills

### 列出所有 Skills

```typescript
const skills = await invoke('list_skills');
console.log('Available skills:', skills);
```

### 启用/禁用 Skill

```typescript
await invoke('toggle_skill', {
  request: {
    skillId: 'echo-skill',
    enabled: false
  }
});
```

### 删除 Skill

```typescript
await invoke('delete_skill', {
  skillId: 'echo-skill'
});
```

---

## 📂 存储位置

Skills 存储在以下目录：

```
~/.cosurf/skills/
├── echo-skill.json        # JSON 格式（内部使用）
├── echo-skill.md          # Markdown 格式（用户编辑）
├── python-calculator.json
├── python-calculator.md
└── ...
```

- `.json` 文件：系统内部使用，包含解析后的配置
- `.md` 文件：用户可编辑的原始 Markdown 文件

---

## 🎯 最佳实践

### 1. 清晰的命名

```markdown
✅ 好：id: alibabacloud-iqs-search
❌ 差：id: skill1
```

### 2. 详细的描述

```markdown
✅ 好：description: 使用阿里云智能检索服务进行语义搜索
❌ 差：description: 搜索
```

### 3. 完整的示例

```markdown
## 使用示例

```bash
# 基本用法
使用 skill-id，param1="value1"

# 高级用法
使用 skill-id，param1="value1", param2=10
```
```

### 4. 错误处理说明

```markdown
## 错误处理

常见错误：
- `INVALID_INPUT`: 输入参数无效
- `TIMEOUT`: 执行超时
- `PERMISSION_DENIED`: 权限不足
```

### 5. 安全注意事项

```markdown
## 安全说明

- ⚠️ 该技能会执行外部命令
- ✅ 已设置超时限制
- ❌ 不支持文件写入操作
```

---

## 🔍 调试技巧

### 查看日志

```bash
# 后端日志
tail -f ~/.cosurf/logs/cosurf.log

# 查找 Skill 相关日志
grep "skill" ~/.cosurf/logs/cosurf.log
```

### 测试 YAML 配置

```bash
# 验证 YAML 语法
python3 -c "import yaml; yaml.safe_load(open('skill.md'))"
```

### 检查参数解析

在 Skill 中添加调试输出：

```python
import json
print("Received arguments:", json.dumps(params, indent=2))
```

---

## 📚 示例 Skills

查看 `examples/` 目录中的完整示例：

1. **[echo-skill.md](../examples/echo-skill.md)** - 简单的 CLI 回显技能
2. **[python-calculator-skill.md](../examples/python-calculator-skill.md)** - Python 数学计算器
3. **[alibabacloud-iqs-search-skill.md](../examples/alibabacloud-iqs-search-skill.md)** - 阿里云 IQS 搜索

---

## ❓ 常见问题

### Q1: Markdown 格式和 JSON 格式有什么区别？

**A**: 
- Markdown 更适合人类阅读和编辑
- JSON 更适合程序解析
- 系统会自动将 Markdown 转换为 JSON 存储
- 两种格式都支持，可以混合使用

### Q2: 如何迁移现有的 JSON Skills？

**A**: 
1. 手动将 JSON 转换为 Markdown 格式
2. 或者继续使用 JSON 格式（向后兼容）
3. 建议新 Skills 使用 Markdown 格式

### Q3: YAML 配置块必须是有效的 YAML 吗？

**A**: 是的，YAML 必须有效，否则解析会失败。可以使用在线 YAML 验证器检查。

### Q4: 参数表格的格式要求是什么？

**A**: 
- 必须是标准的 Markdown 表格
- 表头：`| 参数名 | 类型 | 必填 | 默认值 | 描述 |`
- 分隔线：`|--------|------|------|--------|------|`
- 数据行：`| param | type | yes/no | default | desc |`

### Q5: 如何在 Skill 中使用环境变量？

**A**: 
```yaml
api_key: ${MY_API_KEY}
```

系统会在执行时自动替换为环境变量的值。

---

## 🚀 下一步

1. **创建你的第一个 Skill**：复制 `examples/echo-skill.md` 并修改
2. **导入 Skill**：使用 `import_skill_from_markdown_file` 命令
3. **测试执行**：使用 `execute_skill` 命令
4. **分享 Skill**：将 `.md` 文件分享给其他人

---

**最后更新**: 2026-05-23  
**版本**: 1.0.0 (Markdown 格式)
