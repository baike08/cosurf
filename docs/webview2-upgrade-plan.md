# WebView2 架构升级方案

## 📋 概述

本文档讨论了在 CoSurf 中使用原生 WebView2 控件替代当前 iframe 方案的可行性、挑战和实施路径。

---

## 🔍 当前架构分析

### 现有实现（iframe 方案）

```
┌─────────────────────────────────────┐
│   Tauri Window (WebView2)           │
│  ┌───────────────────────────────┐  │
│  │   React App                   │  │
│  │  ┌─────────────────────────┐  │  │
│  │  │   <iframe>              │  │  │
│  │  │   - 同源策略限制        │  │  │
│  │  │   - 跨域无法注入脚本    │  │  │
│  │  │   - postMessage 通信    │  │  │
│  │  └─────────────────────────┘  │  │
│  └───────────────────────────────┘  │
└─────────────────────────────────────┘
```

**优势**：
- ✅ 简单易实现
- ✅ 支持多标签页（通过 React 状态管理）
- ✅ 已集成 Playwright 用于深度交互

**劣势**：
- ❌ 同源策略限制（跨域网站无法注入脚本）
- ❌ `shell.open` 权限错误（第三方扩展尝试调用 Tauri API）
- ❌ 链接点击拦截不完整（依赖 `allow-popups-to-escape-sandbox`）

---

## 🎯 WebView2 原生方案

### 理想架构

```
┌─────────────────────────────────────┐
│   Tauri Window                      │
│  ┌───────────────────────────────┐  │
│  │   React UI Overlay            │  │
│  │   (地址栏、标签栏等)           │  │
│  └───────────────────────────────┘  │
│  ┌───────────────────────────────┐  │
│  │   WebView2 Control            │  │
│  │   - 无同源策略限制            │  │
│  │   - postMessage 通信          │  │
│  │   - 捕获所有点击事件          │  │
│  │   - 原生交互能力              │  │
│  └───────────────────────────────┘  │
└─────────────────────────────────────┘
```

**优势**：
- ✅ 无同源策略限制
- ✅ 可以捕获所有点击事件
- ✅ 完美的原生交互（postMessage、Host Objects）
- ✅ 支持透明背景、UI 叠加

**挑战**：
- ❌ **Tauri 2.x 不支持动态创建多个 WebView2 实例**
- ❌ 需要重新设计整个标签页管理系统
- ❌ UI 分层复杂（React UI 需要悬浮在 WebView2 之上）

---

## ⚠️ Tauri 2.x 的技术限制

根据官方文档和实际测试，Tauri 2.x 存在以下限制：

1. **WebviewBuilder 是私有结构体**：
   ```rust
   // ❌ 无法这样做
   let webview = WebviewBuilder::new(...)
       .build()?;
   ```

2. **WebviewWindow 不支持子窗口**：
   ```rust
   // ❌ 这些方法已移除
   window.add_child(...);
   window.remove_child(...);
   ```

3. **每个窗口只能有一个 WebView2**：
   - 无法在一个窗口中创建多个独立的 WebView2 控件
   - 多标签页需要通过多窗口实现（用户体验差）

---

## 💡 可行方案对比

### 方案 1：单 WebView2 + URL 切换

**思路**：
- 主窗口直接使用 WebView2 加载网页
- React UI 作为覆盖层
- 通过 Tauri 命令控制导航

**评估**：
- ❌ **不可行**：Tauri 的 WebView2 和 React UI 在同一层级，无法实现覆盖效果
- ❌ 需要 fork Tauri 或等待官方支持

---

### 方案 2：混合方案 - 临时 WebView2 窗口

**思路**：
- 日常浏览使用 iframe（当前方案）
- AI Agent 操作时创建临时 WebView2 窗口
- 通过 postMessage 通信

**实现示例**：

```rust
// src-tauri/src/commands/webview2_window.rs
#[tauri::command]
pub async fn create_interactive_webview(
    app: AppHandle,
    url: String,
    tab_id: String,
) -> Result<(), String> {
    let window = WebviewWindowBuilder::new(
        &app,
        format!("interactive-{}", tab_id),
        WebviewUrl::App(url.into()),
    )
    .title("AI Interactive Mode")
    .inner_size(1200.0, 800.0)
    .build()
    .map_err(|e| e.to_string())?;
    
    // 注入 JavaScript 捕获点击
    window.eval(r#"
        document.addEventListener('click', (e) => {
            const target = e.target.closest('a');
            if (target && target.href) {
                window.chrome.webview.postMessage({
                    type: 'LINK_CLICKED',
                    url: target.href
                });
            }
        });
    "#).ok();
    
    Ok(())
}
```

**评估**：
- ✅ **可行**：利用 Tauri 的多窗口能力
- ⚠️ 用户体验一般（弹出新窗口）
- ✅ 适合 AI Agent 场景

---

### 方案 3：增强现有 iframe 方案（推荐）⭐

**思路**：
- 保持当前的 iframe 架构
- 通过后端代理解决跨域问题
- 使用 Playwright 进行深度交互

**已实现功能**：
- ✅ Playwright 内容提取（三层降级策略）
- ✅ 同源网站的链接拦截
- ✅ postMessage 通信
- ✅ shell.open 错误静默处理

**改进措施**：
1. 添加用户友好的提示（已完成）
2. 优化 AI Agent 的网页操作能力
3. 考虑未来升级到 Tauri 3.x（如果支持多 WebView2）

**评估**：
- ✅ **最佳选择**：成本低，见效快
- ✅ 无需大规模重构
- ✅ 已有 Playwright 集成
- ⚠️ 跨域网站仍有局限（但可接受）

---

## 🚀 实施建议

### 短期（当前版本）

1. **保持 iframe 架构**
   - 稳定可靠
   - 开发成本低

2. **增强 Playwright 集成**
   - 用于 AI Agent 的深度网页操作
   - 绕过同源策略限制

3. **优化用户体验**
   - 添加清晰的提示和指导
   - 静默无害的错误

### 中期（未来 3-6 个月）

1. **监控 Tauri 3.x 进展**
   - 关注是否支持多 WebView2 实例
   - 评估升级成本

2. **探索混合方案**
   - 为 AI Agent 创建专用的交互式窗口
   - 提升自动化操作体验

### 长期（未来 1 年+）

1. **完全迁移到 WebView2 原生方案**（如果 Tauri 支持）
   - 重新设计标签页管理
   - 实现完美的原生交互

2. **或者考虑其他框架**
   - WPF/WinUI + WebView2（纯原生）
   - Electron + webview tag（更灵活）

---

## 📊 决策矩阵

| 方案 | 开发成本 | 用户体验 | 技术可行性 | 维护成本 | 推荐度 |
|------|---------|---------|-----------|---------|--------|
| 单 WebView2 + URL 切换 | 高 | 中 | ❌ 不可行 | 高 | ⭐ |
| 混合方案（临时窗口） | 中 | 中 | ✅ 可行 | 中 | ⭐⭐⭐ |
| 增强 iframe（当前） | 低 | 良好 | ✅ 可行 | 低 | ⭐⭐⭐⭐⭐ |

---

## 🎯 结论

**当前最佳选择**：继续增强现有的 iframe + Playwright 方案

**理由**：
1. Tauri 2.x 的技术限制使得原生 WebView2 方案不可行
2. Playwright 已经提供了强大的网页交互能力
3. 开发成本低，可以快速迭代
4. 用户体验已经足够好

**未来方向**：
- 密切关注 Tauri 3.x 的发展
- 如果官方支持多 WebView2 实例，再考虑全面升级
- 在此之前，专注于优化现有功能和 AI Agent 能力

---

## 📚 参考资料

- [Tauri 2.x Webview 限制](https://tauri.app/v2/guide/concepts/multiwindow/)
- [WebView2 官方文档](https://learn.microsoft.com/en-us/microsoft-edge/webview2/)
- [CoSurf Playwright 集成](../playwright-service/README.md)
