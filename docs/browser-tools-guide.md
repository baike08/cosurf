# 浏览器联动工具使用说明

## 概述

CoSurf 实现了 AI 对话与浏览器的深度联动，支持以下两个核心工具：

1. **智能总结** - 对当前浏览器标签页打开的网页内容进行智能总结
2. **网页操作** - 通过指令对浏览器标签页进行自动化操作（点击、填写表单、关闭弹窗等）

## 架构设计

### 后端 (Rust)

#### 1. 页面内容提取 (`src-tauri/src/commands/page_context.rs`)

```rust
// 智能总结当前页面
#[tauri::command]
pub async fn summarize_page(
    app: AppHandle,
    state: State<'_, AppState>,
    tab_id: String,
    max_length: Option<usize>,
) -> AppResult<String>

// 执行网页操作
#[tauri::command]
pub async fn execute_web_action(
    app: AppHandle,
    tab_id: String,
    action: String,      // "click" | "fill" | "close_popup"
    selector: String,    // CSS 选择器
    value: Option<String>, // 填写的值（仅 fill 操作需要）
) -> AppResult<String>
```

#### 2. 工具定义 (`src-tauri/src/ai/tools.rs`)

已定义的工具 schema：
- `summarize_page` - 总结页面内容
- `web_agent` - 执行网页自动化操作
- `screenshot` - 截图并理解页面
- `translate` - 翻译页面内容
- `export_markdown` - 导出为 Markdown
- `web_search` - 联网搜索

### 前端 (TypeScript/React)

#### 1. 工具调用库 (`src-web/src/lib/tools.ts`)

```typescript
// 智能总结
await summarizeCurrentPage(tabId, maxLength)

// 网页操作
await executeWebAction(tabId, action, selector, value)

// 工具执行器
const executor = new ToolExecutor(activeTabId)
await executor.executeTool("summarize_page", { max_length: 500 })
```

#### 2. 事件监听 (`src-web/src/components/layout/WebContentView.tsx`)

监听后端发来的页面内容提取请求：
```typescript
listen('webview:get-content', async (event) => {
  const { tabId, script } = event.payload
  // 在 iframe 中执行脚本获取内容
  const result = iframeDoc.defaultView?.eval(script)
  // 返回结果
  window.dispatchEvent(new CustomEvent('cosurf:page-content', {
    detail: { tabId, content: result }
  }))
})
```

## 使用方法

### 方法一：直接调用命令

在前端代码中直接调用 Tauri 命令：

```typescript
import { invoke } from "@tauri-apps/api/core"

// 总结页面
const summary = await invoke("summarize_page", {
  tabId: activeTabId,
  maxLength: 500
})

// 点击元素
await invoke("execute_web_action", {
  tabId: activeTabId,
  action: "click",
  selector: "#submit-button"
})

// 填写表单
await invoke("execute_web_action", {
  tabId: activeTabId,
  action: "fill",
  selector: "input[name='username']",
  value: "testuser"
})

// 关闭弹窗
await invoke("execute_web_action", {
  tabId: activeTabId,
  action: "close_popup",
  selector: ""
})
```

### 方法二：使用工具执行器

```typescript
import { ToolExecutor } from "@/lib/tools"

const executor = new ToolExecutor(activeTabId)

// 总结页面
const summary = await executor.executeTool("summarize_page", {
  max_length: 500
})

// 网页操作
const result = await executor.executeTool("web_agent", {
  action: "click",
  selector: ".login-button"
})
```

### 方法三：在 AI 对话中使用（TODO）

未来可以在 AI 对话中自动识别用户意图并调用工具：

```
用户: "帮我总结一下这个页面"
AI: [检测到总结意图] → 调用 summarize_page 工具 → 返回总结

用户: "点击登录按钮"
AI: [检测到点击意图] → 调用 web_agent 工具 → 执行点击

用户: "在用户名输入框填入 test"
AI: [检测到填写意图] → 调用 web_agent 工具 → 填写表单
```

## 支持的网页操作

### 1. 点击元素 (click)

```typescript
await executeWebAction(tabId, "click", "#button-id")
await executeWebAction(tabId, "click", ".submit-btn")
await executeWebAction(tabId, "click", "a[href='/logout']")
```

### 2. 填写表单 (fill)

```typescript
await executeWebAction(tabId, "fill", "input[name='email']", "user@example.com")
await executeWebAction(tabId, "fill", "#password", "secret123")
await executeWebAction(tabId, "fill", "textarea", "这是一段文本")
```

### 3. 关闭弹窗 (close_popup)

```typescript
await executeWebAction(tabId, "close_popup", "")
```

自动尝试以下方式关闭弹窗：
- 移除 `.modal`, `.popup`, `[role="dialog"]` 元素
- 点击 `.close`, `[aria-label="Close"]`, `button.close` 按钮
- 发送 ESC 键盘事件

## 测试工具

提供了 `ToolDemo` 组件用于测试功能：

```tsx
import { ToolDemo } from "@/components/layout/ToolDemo"

// 在应用中添加测试面板
<ToolDemo />
```

## 注意事项

### 1. 跨域限制

由于浏览器安全策略，iframe 只能访问同源网站的内容。对于跨域网站：
- 页面内容提取可能失败
- 建议在系统提示词中告知 AI 这一限制

### 2. 选择器准确性

CSS 选择器需要准确匹配目标元素：
- 使用开发者工具检查元素的 selector
- 优先使用 ID 选择器 (`#id`)
- 其次使用类选择器 (`.class`)
- 最后使用属性选择器 (`[name='xxx']`)

### 3. 异步操作

所有工具调用都是异步的，需要正确处理：
```typescript
try {
  const result = await executeWebAction(...)
  console.log("成功:", result)
} catch (error) {
  console.error("失败:", error)
}
```

## 未来扩展

可以添加更多工具：

1. **滚动页面**
   ```rust
   browser_scroll(tab_id, direction, amount)
   ```

2. **等待元素出现**
   ```rust
   browser_wait_for_element(tab_id, selector, timeout)
   ```

3. **获取元素信息**
   ```rust
   browser_get_element_info(tab_id, selector)
   ```

4. **截图并分析**
   ```rust
   browser_screenshot_and_analyze(tab_id, prompt)
   ```

5. **多步操作序列**
   ```rust
   browser_execute_sequence(tab_id, actions: Vec<Action>)
   ```

## 相关文件

- 后端命令: `src-tauri/src/commands/page_context.rs`
- 工具定义: `src-tauri/src/ai/tools.rs`
- 前端工具库: `src-web/src/lib/tools.ts`
- 事件监听: `src-web/src/components/layout/WebContentView.tsx`
- 测试组件: `src-web/src/components/layout/ToolDemo.tsx`
