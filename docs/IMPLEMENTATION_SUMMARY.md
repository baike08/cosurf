# 浏览器联动工具实现总结

## 实现的功能

### ✅ 1. 智能总结工具 (summarize_page)

**功能描述**: 对当前浏览器标签页打开的网页内容进行智能总结

**实现位置**:
- 后端: `src-tauri/src/commands/page_context.rs::summarize_page`
- 前端: `src-web/src/lib/tools.ts::summarizeCurrentPage`

**工作流程**:
1. AI 检测到用户想要总结页面
2. 调用 `summarize_page` 命令
3. 后端通过 Tauri 事件通知前端提取页面内容
4. 前端在 iframe 中执行 JavaScript 获取页面文本
5. 返回内容给后端，由 AI 进行总结（TODO: 集成 LLM）

**使用示例**:
```typescript
// 直接调用
const summary = await invoke("summarize_page", {
  tabId: activeTabId,
  maxLength: 500
})

// 使用工具执行器
const executor = new ToolExecutor(activeTabId)
const summary = await executor.executeTool("summarize_page", { max_length: 500 })
```

### ✅ 2. 网页操作工具 (web_agent / execute_web_action)

**功能描述**: 通过指令对浏览器标签页进行自动化操作

**支持的操作**:
- **click** - 点击元素
- **fill** - 填写表单
- **close_popup** - 关闭弹窗

**实现位置**:
- 后端: `src-tauri/src/commands/page_context.rs::execute_web_action`
- 前端: `src-web/src/lib/tools.ts::executeWebAction`

**工作流程**:
1. AI 解析用户意图，确定要执行的操作
2. 调用 `execute_web_action` 命令
3. 后端生成对应的 JavaScript 代码
4. 通过 Tauri 事件发送到前端
5. 前端在 iframe 中执行脚本完成操作

**使用示例**:
```typescript
// 点击按钮
await invoke("execute_web_action", {
  tabId: activeTabId,
  action: "click",
  selector: "#submit-button"
})

// 填写表单
await invoke("execute_web_action", {
  tabId: activeTabId,
  action: "fill",
  selector: "input[name='email']",
  value: "user@example.com"
})

// 关闭弹窗
await invoke("execute_web_action", {
  tabId: activeTabId,
  action: "close_popup",
  selector: ""
})
```

## 技术架构

### 后端 (Rust)

#### 命令注册 (`src-tauri/src/lib.rs`)
```rust
commands::page_context::summarize_page,
commands::page_context::execute_web_action,
```

#### 工具 Schema (`src-tauri/src/ai/tools.rs`)
定义了 OpenAI function calling 格式的工具描述：
```rust
BuiltInTool::SummarizePage.to_openai_schema()
BuiltInTool::WebAgent.to_openai_schema()
```

### 前端 (TypeScript/React)

#### 事件监听 (`src-web/src/components/layout/WebContentView.tsx`)
```typescript
listen('webview:get-content', async (event) => {
  // 在 iframe 中执行脚本
  const result = iframeDoc.defaultView?.eval(script)
  // 返回结果
  window.dispatchEvent(new CustomEvent('cosurf:page-content', {
    detail: { tabId, content: result }
  }))
})
```

#### 工具库 (`src-web/src/lib/tools.ts`)
提供统一的工具调用接口：
```typescript
export class ToolExecutor {
  async executeTool(toolName: string, args: any): Promise<any>
}
```

## 文件清单

### 新增文件
1. `src-web/src/lib/tools.ts` - 工具调用库
2. `src-web/src/components/layout/ToolDemo.tsx` - 测试组件
3. `docs/browser-tools-guide.md` - 使用文档

### 修改文件
1. `src-tauri/src/commands/page_context.rs` - 添加 summarize_page 和 execute_web_action
2. `src-tauri/src/lib.rs` - 注册新命令
3. `src-web/src/components/layout/WebContentView.tsx` - 添加页面内容提取事件监听

## 测试方法

### 1. 使用 ToolDemo 组件

在应用中临时添加测试面板：
```tsx
import { ToolDemo } from "@/components/layout/ToolDemo"

function App() {
  return (
    <>
      {/* 其他组件 */}
      <ToolDemo />
    </>
  )
}
```

### 2. 手动测试

打开开发者控制台，执行：
```javascript
// 获取当前活跃标签页 ID
const tabId = window.__TAURI_INTERNALS__?.metadata?.currentTabId

// 测试总结
await invoke("summarize_page", { tabId, maxLength: 500 })

// 测试点击
await invoke("execute_web_action", { 
  tabId, 
  action: "click", 
  selector: "button:first-of-type" 
})
```

## 已知限制

### 1. 跨域限制 ⚠️

由于浏览器同源策略，iframe 只能访问同源网站的内容。

**影响**:
- 对于跨域网站，无法提取页面内容
- 无法执行 JavaScript 操作

**解决方案**:
- 在系统提示词中告知 AI 这一限制
- 未来可以考虑使用 Playwright 服务处理跨域场景

### 2. 异步通信 ⚠️

当前实现中，后端发送事件后无法直接等待前端返回结果。

**现状**:
- `summarize_page` 目前返回"正在提取页面内容..."
- 需要实现完整的事件响应机制

**TODO**:
- 实现基于 Promise 的事件响应
- 或使用 channel 机制进行双向通信

### 3. AI 工具调用集成 ⚠️

工具定义已完成，但尚未在 AI 对话流中自动调用。

**现状**:
- 工具 schema 已定义在 `tools.rs`
- 需要修改 `send_chat_message` 以支持 function calling

**TODO**:
- 在 AI 请求中添加 `tools` 参数
- 解析 AI 返回的工具调用
- 执行工具并将结果反馈给 AI

## 下一步计划

### Phase 1: 完善基础功能
- [ ] 实现完整的事件响应机制（后端等待前端返回）
- [ ] 添加错误处理和重试逻辑
- [ ] 优化页面内容提取算法

### Phase 2: AI 集成
- [ ] 在 `send_chat_message` 中启用 function calling
- [ ] 解析并执行 AI 的工具调用
- [ ] 将工具执行结果反馈给 AI 继续对话

### Phase 3: 增强功能
- [ ] 添加更多网页操作（滚动、等待、截图等）
- [ ] 支持多步操作序列
- [ ] 实现视觉理解（截图 + VLM）

### Phase 4: 用户体验
- [ ] 在 UI 中显示工具执行状态
- [ ] 添加操作历史记录
- [ ] 提供快捷操作按钮

## 相关文档

- [详细使用指南](./browser-tools-guide.md)
- [工具定义](../src-tauri/src/ai/tools.rs)
- [测试组件](../src-web/src/components/layout/ToolDemo.tsx)
