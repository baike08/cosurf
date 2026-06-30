# Codex Agent 集成进度报告

## 📊 当前状态

**分支**: `feat/codex_adapt`  
**最后更新**: 2026-06-30

---

## ✅ 已完成的工作

### Phase 1: TUI 设计元素迁移 ✅ (100%)

借鉴 Codex TUI 的设计，创建了 3 个 React 组件：

1. **ThinkingIndicator** - Thinking 过程可视化
   - ✨ 动态 Spinner 动画（thinking/tool_call/streaming）
   - ✨ 折叠/展开功能
   - 📁 [src-web/src/components/ai/ThinkingIndicator.tsx](file://d:\coding-harness\CoSurf\src-web\src\components\ai\ThinkingIndicator.tsx)

2. **ToolCallCard** - 工具调用卡片
   - ✨ 状态徽章（pending/executing/success/failed）
   - ✨ 可折叠参数和结果
   - ✨ 执行时间显示
   - 📁 [src-web/src/components/ai/ToolCallCard.tsx](file://d:\coding-harness\CoSurf\src-web\src\components\ai\ToolCallCard.tsx)

3. **SessionStatusBar** - 会话状态栏
   - ✨ Token 使用量估算
   - ✨ 模型信息显示
   - ✨ 实时状态指示
   - 📁 [src-web/src/components/ai/SessionStatusBar.tsx](file://d:\coding-harness\CoSurf\src-web\src\components\ai\SessionStatusBar.tsx)

---

### Phase 2: Codex Agent 内核对接 🔄 (30%)

#### 2.1 Tool Registry 框架 ✅
- 定义统一的 `Tool` trait
- 实现 `ToolRegistry` 管理工具
- 提供全局注册表实例
- 📁 [native/src/ai/tool_registry.rs](file://d:\coding-harness\CoSurf\native\src\ai\tool_registry.rs)

#### 2.2 Codex Adapter 框架 ✅
- 创建 `CodexAgent` 结构封装 ThreadManager
- 提供与现有 `stream.rs` 兼容的接口
- 添加 `codex-core` 依赖（path dependency）
- 📁 [native/src/ai/codex_adapter.rs](file://d:\coding-harness\CoSurf\native\src\ai\codex_adapter.rs)

---

## ⏳ 待完成的工作

### Phase 2 剩余任务（优先级高）

#### 2.3 实现 Codex Config 构建 🔴
**文件**: `native/src/ai/codex_adapter.rs:98-107`

```rust
fn build_codex_config(config: &CodexAgentConfig) -> AppResult<Config> {
    // TODO: 根据 Codex 的 Config 结构构建配置
    // 需要研究 codex-core 的 Config 类型定义
}
```

**需要研究：**
- `codex_core::Config` 的结构
- 必需的字段有哪些
- 如何映射 CoSurf 的 ModelConfig 到 Codex Config

---

#### 2.4 实现消息发送和流式响应 🔴
**文件**: `native/src/ai/codex_adapter.rs:81-95`

```rust
pub async fn send_message_stream(
    &self,
    thread_handle: &mut CodexThreadHandle,
    message: &str,
) -> AppResult<mpsc::Receiver<String>> {
    // TODO: 调用 Codex Thread 的 turn 方法
    // TODO: 将 Codex 的事件流转换为字符串流
}
```

**需要研究：**
- `CodexThread` 的 API
- 如何启动一个 turn
- 如何接收流式事件
- 如何将事件转换为文本

---

#### 2.5 集成到现有 stream.rs 🟡
**目标**: 提供开关，让用户选择使用 CoSurf Agent 还是 Codex Agent

```rust
// native/src/ai/stream.rs
pub async fn stream_chat_completion(
    config: &ModelConfig,
    messages: Vec<ChatMessage>,
    // ...
) -> AppResult<()> {
    // 检查是否启用 Codex
    if should_use_codex() {
        return stream_with_codex(messages, conversation_id, callbacks).await;
    }
    
    // 否则使用现有的 CoSurf Agent Loop
    // ...
}
```

---

### Phase 3: 工具整合（优先级中）

#### 3.1 将 Skills 包装为 Tool 🟡
- 实现 `Tool` trait for Skills
- 注册到 ToolRegistry

#### 3.2 将 MCP 工具包装为 Tool 🟡
- 实现 `Tool` trait for MCP tools
- 注册到 ToolRegistry

#### 3.3 将内置工具包装为 Tool 🟡
- browser_navigate
- web_search
- summarize_page
- 等等...

---

### Phase 4: 测试和优化（优先级低）

#### 4.1 端到端测试 🔵
- 测试 Codex Agent 基本流程
- 对比 CoSurf Agent vs Codex Agent
- 性能基准测试

#### 4.2 错误处理 🔵
- 完善的错误恢复机制
- 用户友好的错误提示

#### 4.3 文档 🔵
- 更新集成文档
- 添加使用示例
- 编写迁移指南

---

## 🎯 下一步行动建议

### 立即开始（今天）

**任务**: 研究 Codex Config 结构

```bash
# 查看 Config 定义
grep -r "pub struct Config" native/src/codex/codex-rs/core/src/

# 查看 ThreadManager::start 的签名
grep -A 20 "pub async fn start" native/src/codex/codex-rs/core/src/thread_manager.rs
```

**目标**: 理解如何正确构建 Codex Config

---

### 本周完成

1. ✅ 实现 `build_codex_config()` 函数
2. ✅ 实现基本的消息发送（非流式）
3. ✅ 测试 Codex Agent 能否正常工作

---

### 下周完成

1. ✅ 实现流式响应
2. ✅ 集成到 stream.rs
3. ✅ 添加前端切换开关
4. ✅ 端到端测试

---

## 📝 技术难点

### 难点 1: Codex 依赖复杂
- Codex core 依赖 ~80 个内部 crate
- 编译时间长
- 可能需要调整 Cargo workspace 配置

**解决方案**: 
- 先尝试 path dependency
- 如果失败，考虑编译为独立库

---

### 难点 2: API 不兼容
- Codex 的事件系统与 CoSurf 不同
- 需要适配层转换

**解决方案**:
- 创建事件适配器
- 逐步映射事件类型

---

### 难点 3: 工具系统集成
- Codex 有自己的工具系统
- CoSurf 有 Skills/MCP/内置工具

**解决方案**:
- 使用 Tool Registry 统一抽象
- 将所有工具包装为统一的 `Tool` trait

---

## 📊 进度追踪

```
Phase 1: TUI 组件          ████████████████████ 100%
Phase 2: Codex 集成        ██████░░░░░░░░░░░░░░  30%
  ├─ Tool Registry         ████████████████████ 100%
  ├─ Codex Adapter 框架    ████████████░░░░░░░░  60%
  ├─ Config 构建           ░░░░░░░░░░░░░░░░░░░░   0%
  ├─ 消息发送              ░░░░░░░░░░░░░░░░░░░░   0%
  └─ 流式响应              ░░░░░░░░░░░░░░░░░░░░   0%
Phase 3: 工具整合          ░░░░░░░░░░░░░░░░░░░░   0%
Phase 4: 测试优化          ░░░░░░░░░░░░░░░░░░░░   0%
```

---

## 🔗 相关资源

- [Codex GitHub](https://github.com/openai/codex)
- [Codex Documentation](https://github.com/openai/codex/tree/main/docs)
- [CoSurf AI Module](file://d:\coding-harness\CoSurf\native\src\ai)
- [集成计划文档](file://d:\coding-harness\CoSurf\docs\CODEx_INTEGRATION_PLAN.md)

---

## 💡 备注

**重要决策**: 
- 采用**直接对接 Codex 内核**的方案，而不是重构现有 Agent Loop
- 保持向后兼容，提供开关让用户选择
- 渐进式迁移，降低风险

**风险提示**:
- Codex 是大型项目，集成难度大
- 可能需要 2-4 周才能完成基本功能
- 建议先实现最小可用版本（MVP），再逐步完善
