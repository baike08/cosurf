# iframe 链接点击处理指南

## 📋 问题背景

在 CoSurf 中，网页内容通过 `<iframe>` 元素加载。由于浏览器的**同源策略（Same-Origin Policy）**，父页面无法直接访问跨域 iframe 的内容，这导致：

- ✅ **同源网站**：可以注入脚本拦截链接点击
- ⚠️ **跨域网站**：无法注入脚本，链接点击使用浏览器默认行为

---

## 🔍 当前实现方案

### 1. 同源网站（Same-Origin）✅

**工作原理**：
```typescript
// 在 iframe 加载完成后注入脚本
injectLinkInterceptor(iframe.contentDocument);

// 脚本功能：
// 1. 拦截所有 <a> 标签的点击事件
// 2. 覆盖 window.open() 方法
// 3. 通过 postMessage 通知父窗口
// 4. 父窗口创建新标签页并导航
```

**支持的场景**：
- 本地开发环境（`http://localhost:*`）
- 同一域名下的子页面
- 使用 `document.domain` 设置为相同域名的网站

**用户体验**：
- ✅ 点击任何链接都会在当前应用内打开新标签页
- ✅ 完全无缝的体验

---

### 2. 跨域网站（Cross-Origin）⚠️

**限制原因**：
```javascript
try {
  iframe.contentDocument; // ❌ SecurityError: Blocked a frame with origin "X" from accessing a cross-origin frame
} catch (e) {
  // 无法访问跨域 iframe 的内容
}
```

**当前行为**：
- ⚠️ 无法注入拦截脚本
- ✅ `allow-popups-to-escape-sandbox` 允许弹窗
- ℹ️ 链接会在**系统默认浏览器**中打开（不是 CoSurf 新标签页）

**为什么选择这个方案？**
1. **保持网站功能完整性**：不阻止弹窗，避免破坏网站功能
2. **用户可控**：用户可以选择在系统浏览器中打开
3. **技术限制**：无法在不修改浏览器安全策略的情况下解决

---

## 💡 解决方案对比

### 方案 1：增强用户提示（当前采用）⭐⭐⭐⭐⭐

**优点**：
- ✅ 简单可靠，无需复杂的技术实现
- ✅ 不破坏网站原有功能
- ✅ 用户可以自由选择

**缺点**：
- ⚠️ 跨域链接不在 CoSurf 内打开

**实施方式**：
```typescript
console.log('[WebPageView] ℹ️ For cross-origin sites:');
console.log('[WebPageView]    - Links with target="_blank" will open in system browser');
console.log('[WebPageView]    - Right-click links and select "Open link in new tab" if available');
console.log('[WebPageView]    - Or use the AI Agent to navigate: "open [url] in new tab"');
console.log('[WebPageView] 💡 Tip: For advanced interaction, use AI Agent commands like "click the login button"');
```

---

### 方案 2：AI Agent 辅助（推荐用于复杂交互）⭐⭐⭐⭐⭐

**核心思想**：
利用 Playwright 的强大能力，让 AI Agent 执行网页操作。

**使用示例**：
```
用户："请点击登录按钮"
AI Agent：
  1. 使用 Playwright 连接到当前页面
  2. 定位登录按钮元素
  3. 执行点击操作
  4. 等待页面响应
  5. 返回结果给用户
```

**优势**：
- ✅ 不受同源策略限制（Playwright 是无头浏览器）
- ✅ 可以执行复杂的网页交互（点击、填写表单、滚动等）
- ✅ 智能理解用户意图

**技术实现**：
```rust
// src-tauri/src/ai/tools_impl/web_agent.rs
async fn execute_web_action(app: &AppHandle, action: &str) -> AppResult<String> {
    // 1. 获取当前活跃标签页的 URL
    let (tab_id, url) = get_active_tab_info(app).await?;
    
    // 2. 使用 Playwright 连接
    let client = reqwest::Client::new();
    
    // 3. 发送动作到 Playwright 服务
    let response = client.post("http://127.0.0.1:3100/execute")
        .json(&serde_json::json!({
            "url": url,
            "action": action
        }))
        .send()
        .await?;
    
    // 4. 返回结果
    Ok(response.text().await?)
}
```

---

### 方案 3：代理页面方案（技术上可行但不推荐）⭐⭐

**原理**：
```
CoSurf Window
  └─ React App
      └─ Proxy Page (同源)
          └─ <iframe> Target Website (跨域)
```

**优点**：
- ✅ 可以在代理页面中注入脚本

**缺点**：
- ❌ 增加了一层嵌套，性能下降
- ❌ 某些网站会检测被嵌入（X-Frame-Options）
- ❌ Cookie 和 Session 可能无法正常传递
- ❌ 复杂度大幅增加

**为什么不采用**：
- 维护成本高
- 用户体验不如直接使用 AI Agent
- 仍然无法绕过所有安全防护

---

### 方案 4：WebView2 原生控件（未来方案）⭐⭐⭐⭐

**前提条件**：
- Tauri 3.x 支持多 WebView2 实例
- 或者迁移到其他框架（WPF/WinUI/Electron）

**优势**：
- ✅ 完全控制网页渲染
- ✅ 无同源策略限制
- ✅ 可以捕获所有导航事件
- ✅ 原生性能

**实施路径**：
参见 [webview2-upgrade-plan.md](./webview2-upgrade-plan.md)

---

## 🎯 最佳实践建议

### 对于普通用户

#### 同源网站
- ✅ 直接点击链接，会自动在新标签页打开

#### 跨域网站
1. **右键菜单**：
   - 右键点击链接
   - 选择"在新标签页中打开"（如果浏览器支持）

2. **手动复制**：
   - 右键复制链接地址
   - 在 CoSurf 地址栏粘贴并回车

3. **使用 AI Agent**（推荐）：
   ```
   用户："请打开 https://example.com"
   AI：好的，正在为您打开...
   ```

---

### 对于开发者

#### 调试同源网站
```javascript
// 在浏览器控制台中检查脚本是否注入
console.log(document.__cosurfLinkInterceptorInjected); // true 表示已注入
```

#### 调试跨域网站
```javascript
// 查看控制台日志
[WebPageView] ⚠️ Cannot access iframe content (cross-origin)
[WebPageView] ℹ️ For cross-origin sites: ...
```

#### 测试 AI Agent
```bash
# 确保 Playwright 服务运行
cd playwright-service
npm run start

# 在 CoSurf 中测试
# 打开任意网页，让 AI 执行操作
```

---

## 📊 方案对比总结

| 方案 | 同源支持 | 跨域支持 | 复杂度 | 推荐度 |
|------|---------|---------|--------|--------|
| **增强用户提示** | ✅ | ⚠️ 系统浏览器 | 低 | ⭐⭐⭐⭐⭐ |
| **AI Agent 辅助** | ✅ | ✅ | 中 | ⭐⭐⭐⭐⭐ |
| 代理页面 | ✅ | ⚠️ 部分 | 高 | ⭐⭐ |
| WebView2 原生 | ✅ | ✅ | 很高 | ⭐⭐⭐⭐ (未来) |

---

## 🔮 未来展望

### 短期（当前版本 v0.1.x）
- ✅ 保持 iframe + Playwright 架构
- ✅ 优化 AI Agent 功能
- ✅ 改进用户提示和引导

### 中期（v0.2.x - v0.3.x）
- 🔍 探索混合方案（为 AI Agent 创建专用 WebView2 窗口）
- 💡 优化 Playwright 集成，提升响应速度
- 📈 收集用户反馈，了解哪些场景需要更深的网页交互

### 长期（v1.0+）
- 🚀 如果 Tauri 支持多 WebView2，考虑全面升级
- 🔄 或者评估其他框架（WPF/WinUI、Electron）
- 🎯 实现真正的原生浏览器体验

---

## ❓ 常见问题

### Q1: 为什么不能直接禁用同源策略？
**A**: 这是浏览器的核心安全机制，禁用会导致严重的安全风险（如 XSS 攻击）。

### Q2: 为什么不用 Electron？
**A**: Electron 打包体积大（100MB+），内存占用高。Tauri 更轻量（~3MB），但牺牲了一些灵活性。

### Q3: AI Agent 能做什么？
**A**: 
- 点击页面元素
- 填写表单
- 滚动页面
- 提取内容
- 截图
- 执行 JavaScript
- 等等...

### Q4: 如何知道当前网站是同源还是跨域？
**A**: 查看浏览器控制台日志：
```
[WebPageView] ✅ Injected link interceptor (same-origin)  // 同源
[WebPageView] ⚠️ Cannot access iframe content (cross-origin)  // 跨域
```

### Q5: 为什么还会看到 "shell.open not allowed" 错误？
**A**: 这是因为某些网站扩展（如 Bilibili 的 biliMirror）尝试调用 Tauri 的 shell API。

**处理方式**：
1. ✅ **已自动静默**：CoSurf 已经添加了全局错误处理器，这些错误不会影响功能
2. ℹ️ **可以忽略**：这只是 Promise rejection，不影响页面正常运行
3. 🔧 **技术原因**：即使设置了 `withGlobalTauri: false`，某些扩展脚本仍会尝试访问 `window.__TAURI__`

**如果您想完全消除这些错误**：
- 禁用浏览器扩展（如 biliMirror）
- 或者在无痕模式下使用 CoSurf

---

## 📚 相关文档

- [WebView2 升级方案](./webview2-upgrade-plan.md)
- [Playwright 服务文档](../playwright-service/README.md)
- [AI Agent 工具列表](../src-tauri/src/ai/tools.rs)

---

**最后更新**: 2026-05-23  
**作者**: CoSurf Team
