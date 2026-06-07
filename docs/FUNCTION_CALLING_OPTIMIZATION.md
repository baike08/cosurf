# Qwen Function Calling 实现与优化

## 🎯 问题诊断

### 原始问题
用户说："帮我打开 baidu"
AI 回复："我只能给你链接，你需要手动点击或在地址栏输入..."

### 根本原因
1. ✅ **工具代码已实现** - `open_url`, `summarize_page`, `web_agent` 都已完整实现
2. ❌ **模型未启用工具调用** - `supports_tool_calling()` 函数中没有包含 `qwen`/`aliyun`/`dashscope`
3. ❌ **工具结果未反馈** - 即使执行了工具，结果也没有返回给 AI 进行第二轮对话

---

## ✅ 已完成的修复

### 1. 启用 Qwen 模型的工具调用支持

**文件**: `src-tauri/src/ai/stream.rs`

```rust
fn supports_tool_calling(provider: &str) -> bool {
    matches!(
        provider,
        "openai" | "anthropic" | "google" | "deepseek" | "moonshot" | "zhipu" 
        | "qwen" | "aliyun" | "dashscope"  // ✅ 新增
    )
}
```

**效果**: 
- Qwen 模型现在会收到工具 schema
- AI 可以决定何时调用工具

---

### 2. 添加工具调用事件通知

**后端事件**:
- `ai:tool-call-start` - 工具开始执行
- `ai:tool-call-result` - 工具执行完成

**前端监听**:
```typescript
// 监听工具调用开始
listen("ai:tool-call-start", (event) => {
  console.log('🔧 正在执行:', payload.tool_name);
  appendStreamDelta(`\n\n🔧 正在执行: ${payload.tool_name}...`);
});

// 监听工具调用结果
listen("ai:tool-call-result", (event) => {
  if (payload.success) {
    appendStreamDelta(`\n\n✅ ${payload.tool_name} 执行成功: ${payload.output}`);
  } else {
    appendStreamDelta(`\n\n❌ ${payload.tool_name} 执行失败: ${payload.output}`);
  }
});
```

**用户体验**:
```
用户: "帮我打开 baidu"

AI: [思考中...]
    [检测到需要打开网页]
    [调用 open_url 工具]

UI 显示:
  🔧 正在执行: open_url...
  ✅ open_url 执行成功: 成功打开网页: https://www.baidu.com
  
AI: "已成功为您打开百度网站！"
```

---

### 3. 添加调试日志

**文件**: `src-tauri/src/commands/ai.rs`

```rust
info!("=== Model Config Debug ===");
info!("Model ID: {}", model_config.model_id);
info!("Provider: {}", model_config.provider);
info!("Base URL: {:?}", model_config.base_url);
info!("=========================");
```

**文件**: `src-tauri/src/ai/stream.rs`

```rust
if supports_tool_calling(&config.provider) {
    info!("Model {} (provider: {}) supports tool calling, injecting {} tools", 
          config.model_id, config.provider, get_available_tools_schemas().len());
} else {
    info!("Model {} (provider: {}) does NOT support tool calling", 
          config.model_id, config.provider);
}
```

---

## 🔧 Function Calling 工作流程

### 完整流程图

```
┌─────────────┐
│   用户输入   │ "帮我打开 baidu"
└──────┬──────┘
       │
       ▼
┌─────────────────────────────────┐
│  AI 大脑 (Qwen Model)           │
│                                  │
│  1. 接收消息 + 工具 Schema      │
│  2. 分析意图                    │
│  3. 决定调用 open_url 工具      │
│  4. 返回 tool_calls             │
└──────┬──────────────────────────┘
       │
       │ SSE Stream
       │ { tool_calls: [...] }
       ▼
┌─────────────────────────────────┐
│  Backend (stream.rs)            │
│                                  │
│  1. 解析 tool_calls             │
│  2. emit("ai:tool-call-start")  │ ← 前端显示 "🔧 正在执行..."
│  3. execute_tool()              │
│  4. emit("ai:tool-call-result") │ ← 前端显示 "✅ 执行成功"
└──────┬──────────────────────────┘
       │
       │ browser_navigate()
       ▼
┌─────────────────────────────────┐
│  浏览器外壳 (Frontend)          │
│                                  │
│  1. 监听 webview:navigating     │
│  2. 更新 iframe.src             │
│  3. 加载 https://www.baidu.com  │
└──────┬──────────────────────────┘
       │
       │ 页面加载完成
       ▼
┌─────────────┐
│   用户看到   │ 百度首页已打开！
└─────────────┘
```

---

## 📊 当前实现状态

### ✅ 已实现的功能

| 功能 | 状态 | 说明 |
|------|------|------|
| 工具 Schema 注入 | ✅ | Qwen 模型现在能收到 7 个工具的 schema |
| 工具调用检测 | ✅ | 解析 AI 返回的 tool_calls |
| 工具执行 | ✅ | execute_tool() 执行 3 个已实现的工具 |
| 事件通知 | ✅ | 前端实时显示工具调用进度 |
| 浏览器操作 | ✅ | open_url 真正打开网页 |
| 调试日志 | ✅ | 完整的日志输出便于排查问题 |

### ⚠️ 待优化的功能

| 功能 | 状态 | 说明 |
|------|------|------|
| 第二轮对话 | ❌ | 工具结果未反馈给 AI |
| 多轮工具调用 | ❌ | 不支持连续调用多个工具 |
| 工具选择优化 | ⚠️ | AI 可能不总是选择正确的工具 |

---

## 🎯 下一步优化方向

### 高优先级：实现第二轮对话

**问题**: 
当前流程是：
1. AI 调用工具
2. 执行工具
3. **结束** ← AI 不知道工具执行的结果

**解决方案**:
```rust
// 伪代码
if has_tool_calls {
    // 1. 执行所有工具
    let tool_results = execute_all_tools(tool_calls).await;
    
    // 2. 构建新的消息，包含工具结果
    let follow_up_messages = vec![
        // 原始对话历史
        ...original_messages,
        
        // AI 的工具调用
        ChatMessage {
            role: "assistant",
            content: "",
            tool_calls: original_tool_calls,
        },
        
        // 工具执行结果
        ChatMessage {
            role: "tool",
            name: "open_url",
            content: "成功打开网页: https://www.baidu.com",
        },
    ];
    
    // 3. 再次调用 API，让 AI 基于工具结果生成最终回复
    let final_response = call_api_again(follow_up_messages).await;
    
    // 4. 发送最终回复给用户
    emit_final_response(final_response);
}
```

**预期效果**:
```
用户: "帮我打开 baidu 并总结一下首页内容"

AI: [第一轮]
  → 调用 open_url 打开百度
  → 调用 summarize_page 总结页面
  
[第二轮]
  → 基于总结结果生成回复
  
AI 最终回复: 
  "已为您打开百度。首页主要包含搜索框、热点新闻、常用服务入口等..."
```

---

### 中优先级：优化工具选择

**问题**: AI 可能不理解何时该使用工具

**解决方案**:
1. **增强 System Prompt**
   ```
   你是一个智能助手，可以使用以下工具帮助用户：
   
   - open_url: 当用户要求打开某个网站时使用
   - summarize_page: 当用户要求总结当前页面时使用
   - web_agent: 当用户要求操作页面元素时使用
   
   如果用户的请求可以通过工具完成，请优先调用工具，而不是只提供文字建议。
   ```

2. **提供示例**
   ```
   示例 1:
   用户: "打开 GitHub"
   正确: 调用 open_url({ url: "https://github.com" })
   错误: "你可以访问 https://github.com"
   
   示例 2:
   用户: "总结这个页面"
   正确: 调用 summarize_page({ max_length: 500 })
   错误: "请告诉我页面的 URL"
   ```

---

### 低优先级：并发工具调用

**问题**: 目前只能顺序执行工具

**解决方案**:
```rust
// 并行执行多个工具
let results = futures::future::join_all(
    tool_calls.iter().map(|tc| execute_tool(app, tc))
).await;
```

---

## 🧪 测试用例

### 测试 1: 打开网页
```
输入: "帮我打开 baidu"
期望:
  1. AI 调用 open_url 工具
  2. 前端显示 "🔧 正在执行: open_url..."
  3. 浏览器导航到 https://www.baidu.com
  4. 前端显示 "✅ open_url 执行成功"
  5. AI 回复 "已成功为您打开百度"
```

### 测试 2: 总结页面
```
输入: "总结一下当前页面"
期望:
  1. AI 调用 summarize_page 工具
  2. 前端提取页面内容
  3. AI 生成总结
  4. 返回总结结果
```

### 测试 3: 网页操作
```
输入: "点击登录按钮"
期望:
  1. AI 调用 web_agent 工具
  2. 在页面中执行 click 操作
  3. 返回执行结果
```

---

## 📝 配置检查清单

请确认您的模型配置：

- [ ] **Provider** 字段是 `aliyun`、`dashscope` 或 `qwen`
- [ ] **Base URL** 正确（例如：`https://dashscope.aliyuncs.com/api/v1`）
- [ ] **API Key** 已配置
- [ ] **Model ID** 支持 Function Calling（例如：`qwen-max`、`qwen-plus`）

查看日志确认：
```
=== Model Config Debug ===
Model ID: qwen-max
Provider: aliyun
Base URL: Some("https://dashscope.aliyuncs.com/api/v1")
=========================

Model qwen-max (provider: aliyun) supports tool calling, injecting 7 tools
```

---

## 🚀 快速验证

1. **重启 CoSurf**（代码已自动重新编译）
2. **发送消息**: "帮我打开 baidu"
3. **观察终端日志**:
   - 应该看到 "Model ... supports tool calling"
   - 应该看到 "Executing tool: open_url"
4. **观察前端 UI**:
   - 应该显示 "🔧 正在执行: open_url..."
   - 应该显示 "✅ open_url 执行成功"
   - 浏览器应该导航到百度

如果仍然没有调用工具，请提供：
1. 终端的完整日志输出
2. 模型配置的截图
3. AI 的完整回复

---

**最后更新**: 2026-06-06
**版本**: v2.0 - Function Calling 优化版
