# Agent 内置工具实现状态

## 📋 工具概览

当前 Agent 定义了 **7 个内置工具**，其中 **3 个已完整实现**，**4 个仅有 Schema 定义**。

---

## ✅ 已完整实现的工具（3/7）

### 1. summarize_page - 页面总结 ⭐⭐⭐

**状态**: ✅ **完全实现**

**功能**: 提取当前页面内容并使用 AI 生成总结

**参数**:
```json
{
  "max_length": {
    "type": "integer",
    "description": "最大摘要长度（字符数），默认 500"
  }
}
```

**实现位置**: `src-tauri/src/ai/stream.rs` (第 317-354 行)

**执行流程**:
1. ✅ 获取当前活跃标签页 ID (`get_active_tab_id`)
2. ✅ 向前端发送事件提取页面内容 (`webview:get-content`)
3. ✅ 等待前端返回内容 (`wait_for_page_content`)
4. ✅ 使用 AI 生成总结 (`summarize_with_ai`)
5. ✅ 返回总结结果

**依赖组件**:
- ✅ `get_active_tab_id()` - 从 AppState 获取活跃标签页
- ✅ `wait_for_page_content()` - 使用 oneshot channel 等待响应
- ✅ `summarize_with_ai()` - 调用非流式 API 生成总结
- ✅ 前端监听器 - WebContentView.tsx 处理 `webview:get-content` 事件

**使用示例**:
```
用户: "总结一下这个页面"
AI: [调用 summarize_page 工具]
    {
      "name": "summarize_page",
      "arguments": { "max_length": 500 }
    }
AI: "该页面主要介绍了..."
```

**注意事项**:
- ⚠️ 跨域网站可能无法提取内容（iframe 限制）
- ⚠️ 超时时间为 5 秒
- ⚠️ 需要模型支持 Function Calling

---

### 2. web_agent - 网页操作 ⭐⭐⭐

**状态**: ✅ **完全实现**

**功能**: 在当前页面执行自动化操作（点击、填写表单等）

**参数**:
```json
{
  "action": {
    "type": "string",
    "enum": ["click", "fill", "select", "scroll", "wait"],
    "description": "要执行的操作类型"
  },
  "selector": {
    "type": "string",
    "description": "CSS 选择器"
  },
  "value": {
    "type": "string",
    "description": "填写的值（仅 fill 操作需要）"
  }
}
```

**实现位置**: `src-tauri/src/ai/stream.rs` (第 356-390 行)

**执行流程**:
1. ✅ 提取 action、selector、value 参数
2. ✅ 获取当前活跃标签页 ID
3. ✅ 调用 `execute_web_action` 命令
4. ✅ 返回执行结果

**支持的操作**:
- ✅ `click` - 点击元素
- ✅ `fill` - 填写表单
- ✅ `select` - 选择下拉框
- ✅ `scroll` - 滚动页面
- ✅ `wait` - 等待

**依赖组件**:
- ✅ `execute_web_action` - `src-tauri/src/commands/page_context.rs`
- ✅ JavaScript 注入 - 在 iframe 中执行脚本

**使用示例**:
```
用户: "点击登录按钮"
AI: [调用 web_agent 工具]
    {
      "name": "web_agent",
      "arguments": {
        "action": "click",
        "selector": "#login-button"
      }
    }
AI: "已点击登录按钮"
```

**注意事项**:
- ⚠️ 跨域网站无法执行操作（iframe 限制）
- ⚠️ CSS 选择器需要准确
- ⚠️ 某些动态内容可能需要等待

---

### 3. open_url - 打开网页 ⭐⭐⭐

**状态**: ✅ **完全实现**

**功能**: 在当前标签页或新标签页打开指定 URL

**参数**:
```json
{
  "url": {
    "type": "string",
    "description": "要打开的网页URL，必须以 http:// 或 https:// 开头"
  }
}
```

**实现位置**: `src-tauri/src/ai/stream.rs` (第 392-419 行)

**执行流程**:
1. ✅ 提取 URL 参数
2. ✅ 验证 URL 格式（必须以 http:// 或 https:// 开头）
3. ✅ 获取当前活跃标签页 ID
4. ✅ 调用 `browser_navigate` 命令
5. ✅ 返回成功消息

**依赖组件**:
- ✅ `browser_navigate` - `src-tauri/src/commands/browser_nav.rs`
- ✅ 前端导航 - WebContentView.tsx 监听 `webview:navigating` 事件

**使用示例**:
```
用户: "帮我打开 GitHub"
AI: [调用 open_url 工具]
    {
      "name": "open_url",
      "arguments": { "url": "https://github.com" }
    }
AI: "已成功打开网页: https://github.com"
```

**注意事项**:
- ✅ URL 格式验证严格
- ✅ 使用正确的 tab_id（从 AppState 获取）
- ✅ 前端会自动更新 iframe src

---

## ❌ 未实现的工具（4/7）

### 4. screenshot - 截图 ❌

**状态**: ❌ **仅有 Schema，未实现**

**预期功能**: 截取当前页面并进行视觉理解

**参数**:
```json
{
  "full_page": {
    "type": "boolean",
    "description": "是否截取整个页面，默认 false"
  }
}
```

**缺失内容**:
- ❌ `execute_tool` 中没有 `screenshot` 分支
- ❌ 没有截图实现代码
- ⚠️ 虽然有 `browser_screenshot` 命令，但只是占位符

**相关文件**:
- `src-tauri/src/commands/browser_nav.rs` (第 269-277 行) - TODO 占位符
- `src-tauri/src/commands/screenshot.rs` - 有截图功能但未集成

**建议实现**:
1. 调用 `capture_full_screen` 或 `capture_region_from_base64`
2. 将截图转换为 base64
3. 如果模型支持视觉理解，发送给 AI
4. 返回分析结果

---

### 5. translate - 翻译 ❌

**状态**: ❌ **仅有 Schema，未实现**

**预期功能**: 翻译当前页面内容为指定语言

**参数**:
```json
{
  "target_language": {
    "type": "string",
    "description": "目标语言，如 'zh', 'en', 'ja'"
  }
}
```

**缺失内容**:
- ❌ `execute_tool` 中没有 `translate` 分支
- ❌ 没有翻译实现代码

**建议实现**:
1. 提取页面内容（类似 summarize_page）
2. 调用 AI 进行翻译
3. 返回翻译结果

---

### 6. export_markdown - 导出 Markdown ❌

**状态**: ❌ **仅有 Schema，未实现**

**预期功能**: 将当前页面内容导出为 Markdown 格式

**参数**: 无

**缺失内容**:
- ❌ `execute_tool` 中没有 `export_markdown` 分支
- ❌ 没有 Markdown 转换逻辑

**建议实现**:
1. 提取页面 HTML 内容
2. 使用库（如 `turndown`）转换为 Markdown
3. 保存为文件或返回内容

---

### 7. web_search - 联网搜索 ❌

**状态**: ❌ **仅有 Schema，未实现**

**预期功能**: 搜索互联网获取最新信息

**参数**:
```json
{
  "query": {
    "type": "string",
    "description": "搜索查询词"
  },
  "max_results": {
    "type": "integer",
    "description": "最大结果数，默认 5"
  }
}
```

**缺失内容**:
- ❌ `execute_tool` 中没有 `web_search` 分支
- ❌ 没有搜索引擎集成

**建议实现**:
1. 调用搜索引擎 API（如 Bing、Google）
2. 解析搜索结果
3. 提取摘要和链接
4. 返回结构化结果

---

## 📊 实现状态总结

| 工具名称 | Schema | 执行逻辑 | 状态 | 优先级 |
|---------|--------|---------|------|--------|
| summarize_page | ✅ | ✅ | ✅ 完全实现 | 🔴 高 |
| web_agent | ✅ | ✅ | ✅ 完全实现 | 🔴 高 |
| open_url | ✅ | ✅ | ✅ 完全实现 | 🔴 高 |
| screenshot | ✅ | ❌ | ❌ 未实现 | 🟡 中 |
| translate | ✅ | ❌ | ❌ 未实现 | 🟢 低 |
| export_markdown | ✅ | ❌ | ❌ 未实现 | 🟢 低 |
| web_search | ✅ | ❌ | ❌ 未实现 | 🟡 中 |

**实现率**: 3/7 = **42.8%**

---

## 🔧 核心基础设施

### ✅ 已实现的基础设施

1. **标签页 ID 管理**
   - ✅ `AppState.active_tab_id` - 存储活跃标签页
   - ✅ `set_active_tab` 命令 - 前端同步
   - ✅ `get_active_tab_id()` - 后端获取

2. **事件通信机制**
   - ✅ `wait_for_page_content()` - oneshot channel
   - ✅ 前端响应 - WebContentView.tsx
   - ✅ requestId 匹配（预留）

3. **工具调用框架**
   - ✅ `execute_tool()` - 统一入口
   - ✅ ToolCall/ToolResult 类型
   - ✅ Schema 自动生成

4. **AI 集成**
   - ✅ `summarize_with_ai()` - 非流式调用
   - ✅ 工具 Schema 注入
   - ✅ auto 模式选择

---

## 🎯 下一步建议

### 高优先级（完善现有功能）

1. **优化 summarize_page**
   - 添加错误处理（跨域情况）
   - 支持自定义提取规则
   - 缓存页面内容避免重复提取

2. **增强 web_agent**
   - 支持更多操作类型
   - 添加元素可见性检查
   - 支持等待条件

3. **改进 open_url**
   - 支持在新标签页打开
   - 添加历史记录追踪
   - 验证 URL 可达性

### 中优先级（实现新工具）

4. **实现 screenshot**
   - 集成现有的截图功能
   - 支持视觉模型（如 GPT-4V）
   - 返回图像描述

5. **实现 web_search**
   - 集成搜索引擎 API
   - 结果去重和排序
   - 提取关键信息

### 低优先级（锦上添花）

6. **实现 translate**
   - 复用 summarize_page 的内容提取
   - 调用翻译 API 或 AI

7. **实现 export_markdown**
   - HTML to Markdown 转换
   - 文件保存功能

---

## 💡 使用建议

### 当前可以使用的功能

✅ **页面总结**: "总结一下这个页面的内容"
✅ **网页操作**: "点击登录按钮"、"填写用户名"
✅ **打开网页**: "帮我打开 GitHub"

### 暂时不可用的功能

❌ **截图分析**: "截图并分析这个页面"
❌ **页面翻译**: "把这个页面翻译成英文"
❌ **导出 Markdown**: "导出为 Markdown"
❌ **联网搜索**: "搜索最新的 AI 新闻"

---

## 📝 技术债务

1. **并发请求管理**: `wait_for_page_content` 不支持多个并发请求
2. **第二轮对话**: 工具执行结果未反馈给 AI 继续对话
3. **跨域限制**: iframe 无法访问跨域网站
4. **错误恢复**: 工具失败后没有重试机制
5. **日志记录**: 缺少工具调用的详细日志

---

## 🚀 快速测试

### 测试 summarize_page
```
用户: "请总结当前页面"
期望: AI 调用 summarize_page 工具并返回总结
```

### 测试 web_agent
```
用户: "点击页面上的第一个链接"
期望: AI 调用 web_agent 工具并执行点击
```

### 测试 open_url
```
用户: "打开 https://www.baidu.com"
期望: AI 调用 open_url 工具并导航到百度
```

---

**最后更新**: 2026-06-06
**版本**: v1.0
