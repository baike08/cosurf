# 浏览器联动工具 - 快速开始

## 🚀 5分钟上手指南

### 第一步：测试基础功能

1. **打开任意网页**（例如 https://example.com）

2. **在开发者控制台测试命令**：

```javascript
// 获取当前标签页ID（假设已存储在全局变量中）
const tabId = "your-tab-id-here";

// 测试智能总结
const summary = await invoke("summarize_page", {
  tabId: tabId,
  maxLength: 500
});
console.log("页面总结:", summary);

// 测试点击操作
await invoke("execute_web_action", {
  tabId: tabId,
  action: "click",
  selector: "button"
});

// 测试填写表单
await invoke("execute_web_action", {
  tabId: tabId,
  action: "fill",
  selector: "input[type='text']",
  value: "Hello World"
});

// 测试关闭弹窗
await invoke("execute_web_action", {
  tabId: tabId,
  action: "close_popup",
  selector: ""
});
```

### 第二步：使用 ToolDemo 组件

在 `src-web/src/App.tsx` 中临时添加测试面板：

```tsx
import { ToolDemo } from "@/components/layout/ToolDemo"

function App() {
  return (
    <div className="h-screen flex flex-col">
      {/* 你的其他组件 */}
      
      {/* 添加工具测试面板 */}
      <div className="fixed right-4 top-20 w-80 bg-surface border border-border rounded-lg shadow-lg z-50">
        <ToolDemo />
      </div>
    </div>
  )
}
```

### 第三步：在代码中使用

#### 方式一：直接调用 Tauri 命令

```typescript
import { invoke } from "@tauri-apps/api/core"

async function summarizePage() {
  const tabId = getActiveTabId() // 从你的状态管理获取
  
  try {
    const result = await invoke("summarize_page", {
      tabId,
      maxLength: 500
    })
    console.log("总结结果:", result)
  } catch (error) {
    console.error("总结失败:", error)
  }
}

async function clickButton() {
  const tabId = getActiveTabId()
  
  try {
    await invoke("execute_web_action", {
      tabId,
      action: "click",
      selector: "#submit-btn"
    })
    console.log("点击成功")
  } catch (error) {
    console.error("点击失败:", error)
  }
}
```

#### 方式二：使用工具执行器

```typescript
import { ToolExecutor } from "@/lib/tools"

// 创建执行器
const executor = new ToolExecutor(activeTabId)

// 总结页面
async function handleSummarize() {
  const summary = await executor.executeTool("summarize_page", {
    max_length: 500
  })
  console.log(summary)
}

// 执行网页操作
async function handleWebAction() {
  const result = await executor.executeTool("web_agent", {
    action: "fill",
    selector: "input[name='email']",
    value: "user@example.com"
  })
  console.log(result)
}
```

## 💡 实际应用场景

### 场景 1：自动填写注册表单

```typescript
async function autoFillRegistration() {
  const tabId = getActiveTabId()
  
  // 填写用户名
  await invoke("execute_web_action", {
    tabId,
    action: "fill",
    selector: "input[name='username']",
    value: "john_doe"
  })
  
  // 填写邮箱
  await invoke("execute_web_action", {
    tabId,
    action: "fill",
    selector: "input[name='email']",
    value: "john@example.com"
  })
  
  // 填写密码
  await invoke("execute_web_action", {
    tabId,
    action: "fill",
    selector: "input[name='password']",
    value: "secure_password_123"
  })
  
  // 点击注册按钮
  await invoke("execute_web_action", {
    tabId,
    action: "click",
    selector: "button[type='submit']"
  })
}
```

### 场景 2：批量提取页面信息

```typescript
async function extractPageInfo() {
  const tabId = getActiveTabId()
  
  // 获取页面内容
  const content = await invoke("summarize_page", {
    tabId,
    maxLength: 2000
  })
  
  // 这里可以进一步处理内容
  // 例如：提取关键信息、生成摘要等
  console.log("页面内容长度:", content.length)
  
  return content
}
```

### 场景 3：自动化测试流程

```typescript
async function runAutomationTest() {
  const tabId = getActiveTabId()
  
  try {
    // 1. 访问登录页面
    await navigateTo("https://example.com/login")
    
    // 2. 填写登录表单
    await invoke("execute_web_action", {
      tabId,
      action: "fill",
      selector: "#username",
      value: "test_user"
    })
    
    await invoke("execute_web_action", {
      tabId,
      action: "fill",
      selector: "#password",
      value: "test_pass"
    })
    
    // 3. 点击登录
    await invoke("execute_web_action", {
      tabId,
      action: "click",
      selector: "#login-btn"
    })
    
    // 4. 等待页面加载（需要实现等待逻辑）
    await sleep(2000)
    
    // 5. 验证登录成功
    const pageContent = await invoke("summarize_page", {
      tabId,
      maxLength: 500
    })
    
    if (pageContent.includes("欢迎")) {
      console.log("✅ 登录成功")
    } else {
      console.log("❌ 登录失败")
    }
    
  } catch (error) {
    console.error("测试失败:", error)
  }
}
```

## 🔧 常见问题

### Q1: 为什么无法提取跨域网站的内容？

**A**: 这是浏览器的同源策略限制。iframe 只能访问与父页面同源的网站内容。

**解决方案**:
- 对于跨域网站，考虑使用 Playwright 服务
- 或在系统提示词中告知 AI 这一限制

### Q2: CSS 选择器找不到元素怎么办？

**A**: 
1. 打开浏览器开发者工具（F12）
2. 使用元素选择器检查目标元素
3. 复制正确的 CSS 选择器
4. 测试选择器：`document.querySelector('your-selector')`

### Q3: 如何等待异步操作完成？

**A**: 目前所有操作都是立即返回的。如果需要等待：

```typescript
// 简单的等待函数
function sleep(ms: number) {
  return new Promise(resolve => setTimeout(resolve, ms))
}

// 使用示例
await invoke("execute_web_action", { /* ... */ })
await sleep(1000) // 等待1秒
// 继续下一步操作
```

### Q4: 如何在 AI 对话中自动调用这些工具？

**A**: 这需要实现 AI 的 function calling 功能（TODO）。

当前可以手动调用：
```typescript
// 用户输入: "帮我总结这个页面"
if (userInput.includes("总结")) {
  const summary = await invoke("summarize_page", { tabId, maxLength: 500 })
  sendMessage(`页面总结:\n${summary}`)
}
```

## 📚 更多资源

- [完整使用指南](./browser-tools-guide.md)
- [实现总结文档](./IMPLEMENTATION_SUMMARY.md)
- [工具定义源码](../src-tauri/src/ai/tools.rs)
- [前端工具库](../src-web/src/lib/tools.ts)

## 🎯 下一步

1. ✅ 测试基础功能
2. ✅ 熟悉 API 调用方式
3. ⏳ 等待 AI function calling 集成
4. ⏳ 体验智能自动化工具

---

**有问题？** 查看 [详细文档](./browser-tools-guide.md) 或提交 Issue。
