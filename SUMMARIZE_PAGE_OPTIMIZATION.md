# Summarize Page 功能优化方案

## 问题分析

### 原有问题
1. **异步通信缺失**：`summarize_page` 只是发送事件后立即返回，没有等待前端响应
2. **缺少请求追踪**：没有 requestId 机制，无法追踪请求和响应的对应关系
3. **跨域限制**：iframe 内容可能因跨域策略无法访问
4. **超时处理**：没有超时机制，可能导致无限等待

## 优化方案

### 1. 后端改进

#### 1.1 添加状态管理
在 `AppState` 中添加页面内容响应缓存：
```rust
pub struct AppState {
    // ... 其他字段
    pub page_content_responses: Arc<Mutex<HashMap<String, String>>>,
}
```

#### 1.2 实现请求-响应机制
- 生成唯一的 `request_id` (UUID)
- 发送事件时携带 `requestId`
- 轮询检查响应缓存（最多等待 5 秒）
- 清理已处理的响应

#### 1.3 新增命令
添加 `receive_page_content` 命令接收前端响应：
```rust
#[tauri::command]
pub fn receive_page_content(
    state: State<'_, AppState>,
    request_id: String,
    content: String,
) -> AppResult<()>
```

### 2. 前端改进

#### 2.1 修改事件监听
在 `WebContentView.tsx` 中：
- 从 `emit` 改为 `invoke` 调用后端命令
- 直接调用 `receive_page_content` 传递响应
- 移除中间的事件转发层

#### 2.2 错误处理
- 跨域限制时发送空内容
- 脚本执行失败时发送空内容
- 所有路径都确保发送响应

### 3. 工作流程

```
┌──────────┐         ┌──────────┐         ┌──────────┐
│  Backend │         │ Frontend │         │   WebView │
└────┬─────┘         └────┬─────┘         └────┬─────┘
     │                     │                    │
     │ 1. generate UUID    │                    │
     │────┐                │                    │
     │    │                │                    │
     │ 2. emit event       │                    │
     │    │ (with reqId)   │                    │
     │────────────────────>│                    │
     │                     │ 3. extract content │
     │                     │───────────────────>│
     │                     │                    │
     │                     │ 4. return content  │
     │                     │<───────────────────│
     │                     │                    │
     │ 5. invoke command   │                    │
     │    │ (reqId + data) │                    │
     │<────────────────────│                    │
     │                     │                    │
     │ 6. store in cache   │                    │
     │────┐                │                    │
     │    │                │                    │
     │ 7. poll & retrieve  │                    │
     │────┘                │                    │
     │                     │                    │
     │ 8. return summary   │                    │
     │────────────────────>│                    │
```

## 技术细节

### 1. 超时控制
- 使用 `tokio::time::timeout` 设置 5 秒超时
- 每 100ms 轮询一次响应缓存
- 超时后返回友好错误提示

### 2. 内存管理
- 响应处理后立即从缓存中删除
- 避免内存泄漏

### 3. 错误处理
- 跨域限制：返回空内容 + 错误提示
- 超时：返回详细错误信息
- 空内容：返回用户友好的提示

## 测试建议

### 1. 同域页面测试
- 打开本地 HTML 文件
- 调用 `summarize_page`
- 验证能正确提取内容

### 2. 跨域页面测试
- 打开外部网站（如百度、GitHub）
- 调用 `summarize_page`
- 验证返回适当的错误提示

### 3. 超时测试
- 打开加载缓慢的页面
- 调用 `summarize_page`
- 验证 5 秒后超时并返回错误

### 4. 大页面测试
- 打开内容丰富的页面
- 验证内容被正确截断到 15000 字符

## 后续优化方向

### 1. 使用 Tauri IPC 直接通信
考虑使用 Tauri 的 `invoke` 机制替代事件系统，实现更直接的同步调用。

### 2. 浏览器扩展集成
对于跨域限制，可以考虑开发浏览器扩展来获取页面内容。

### 3. Playwright 集成
使用 Playwright 服务进行真正的浏览器自动化，绕过 iframe 限制。

### 4. 缓存优化
- 对相同 URL 的内容进行缓存
- 设置合理的过期时间
- 减少重复提取

## 代码变更清单

### 后端文件
- ✅ `src-tauri/src/state.rs` - 添加响应缓存
- ✅ `src-tauri/src/commands/page_context.rs` - 实现完整逻辑
- ✅ `src-tauri/src/lib.rs` - 注册新命令

### 前端文件
- ✅ `src-web/src/components/layout/WebContentView.tsx` - 修改响应发送逻辑

## 注意事项

1. **跨域限制是浏览器安全策略**，无法完全绕过
2. **超时时间可根据需要调整**（当前 5 秒）
3. **内容长度限制为 15000 字符**，可根据模型上下文窗口调整
4. **建议使用 AI 工具调用**来触发页面总结，而不是直接调用此命令
