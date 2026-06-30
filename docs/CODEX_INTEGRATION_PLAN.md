# Codex 源码集成计划

## 📦 源码位置

- **Codex 仓库**: `native/src/codex/`
- **核心 Rust 代码**: `native/src/codex/codex-rs/`
- **分支**: `feat/codex_adapt`

---

## 🏗️ Codex 架构概览

### 核心模块（codex-rs/）

```
codex-rs/
├── core/              # ⭐⭐⭐ Agent 核心引擎
│   └── src/
│       ├── session/   # 会话管理、Agent Loop
│       ├── tools/     # 工具系统
│       ├── mcp/       # MCP 集成
│       └── state/     # 状态持久化
│
├── tools/             # ⭐⭐ 工具定义和实现
│   └── src/
│       ├── tool_call.rs
│       ├── mcp_tool.rs
│       └── dynamic_tool.rs
│
├── tui/               # ⭐⭐⭐ 终端 UI（借鉴到 AIPanel）
│   └── src/
│       ├── streaming/ # 流式响应处理
│       ├── status/    # 状态栏
│       ├── chatwidget/# 聊天组件
│       └── render/    # Markdown 渲染
│
├── cli/               # CLI 入口（不需要）
├── exec/              # 执行模式（可选参考）
├── sandboxing/        # 沙箱（可选参考）
├── config/            # 配置管理
├── login/             # 认证
└── protocol/          # 内部协议
```

---

## 🎯 集成目标

### Phase 1: TUI 设计元素迁移（优先级 P0）

**借鉴 Codex TUI 的设计到 CoSurf AIPanel：**

#### 1. Thinking 过程可视化
- **源文件**: `codex-rs/tui/src/streaming/controller.rs`
- **目标**: 创建 `ThinkingIndicator` React 组件
- **特性**:
  - Spinner 动画（thinking/executing/streaming）
  - 折叠/展开功能
  - 进度指示

#### 2. 工具调用卡片
- **源文件**: `codex-rs/tui/src/chatwidget/tool_calls.rs` (推测)
- **目标**: 创建 `ToolCallCard` React 组件
- **特性**:
  - 状态徽章（pending/executing/success/failed）
  - 可折叠参数和结果
  - 执行时间显示

#### 3. 会话状态栏
- **源文件**: `codex-rs/tui/src/status/card.rs`
- **目标**: 创建 `SessionStatusBar` React 组件
- **特性**:
  - Token 使用量
  - 模型信息
  - 工具调用统计

#### 4. 流式响应优化
- **源文件**: `codex-rs/tui/src/streaming/chunking.rs`
- **目标**: 优化 AIPanel 的流式更新
- **特性**:
  - 打字机效果
  - 光标闪烁
  - 平滑滚动

---

### Phase 2: Agent Engine 重构（优先级 P1）

**从 Codex core 提取设计模式：**

#### 1. 简化的 Agent Loop
- **源文件**: `codex-rs/core/src/session/session.rs`
- **目标**: 重构 `native/src/ai/stream.rs`
- **改进点**:
  - 更清晰的循环结构
  - 简化的错误处理
  - 更好的取消支持

#### 2. 统一工具注册表
- **源文件**: `codex-rs/tools/src/lib.rs`
- **目标**: 创建 `native/src/ai/tool_registry.rs`
- **特性**:
  - 统一的 Tool trait
  - Skills/MCP/内置工具整合
  - 动态工具加载

#### 3. MCP 协议改进
- **源文件**: `codex-rs/core/src/session/mcp.rs`
- **目标**: 优化 `native/src/ai/mcp.rs`
- **改进点**:
  - 更完善的 JSON-RPC 实现
  - 更好的错误处理
  - 连接池管理

---

### Phase 3: 状态持久化（优先级 P2）

**借鉴 Codex 的 Checkpoint 机制：**

#### 1. SQLite 存储优化
- **源文件**: `codex-rs/state/src/`
- **目标**: 优化 `native/src/db/`
- **改进点**:
  - 更高效的索引
  - 批量操作
  - WAL 模式优化

#### 2. 会话恢复
- **源文件**: `codex-rs/core/src/session/rollout_reconstruction.rs`
- **目标**: 增强 `native/src/ai/checkpoint.rs`
- **特性**:
  - 更快的恢复速度
  - 增量保存
  - 自动清理

---

## 📋 实施步骤

### Week 1: TUI 组件实现

**Day 1-2: ThinkingIndicator 组件**
```tsx
// src-web/src/components/ai/ThinkingIndicator.tsx
- 实现 spinner 动画
- 添加折叠/展开功能
- 集成到 AIPanel
```

**Day 3-4: ToolCallCard 组件**
```tsx
// src-web/src/components/ai/ToolCallCard.tsx
- 实现状态徽章
- 添加可折叠详情
- 显示执行时间
```

**Day 5: SessionStatusBar 组件**
```tsx
// src-web/src/components/ai/SessionStatusBar.tsx
- 显示 Token 使用量
- 显示模型信息
- 显示工具统计
```

---

### Week 2: Agent Engine 重构

**Day 1-2: 创建 Tool Registry**
```rust
// native/src/ai/tool_registry.rs
- 定义 Tool trait
- 实现注册表
- 包装现有工具
```

**Day 3-4: 简化 Agent Loop**
```rust
// native/src/ai/codex_engine.rs
- 实现简化的循环逻辑
- 集成 Tool Registry
- 保持向后兼容
```

**Day 5: 测试和优化**
- 端到端测试
- 性能对比
- Bug 修复

---

### Week 3: 集成和优化

**Day 1-2: MCP 改进**
- 研究 Codex 的 MCP 实现
- 优化现有代码
- 添加新功能

**Day 3-4: 状态持久化优化**
- 改进 Checkpoint 机制
- 优化数据库查询
- 添加自动清理

**Day 5: 文档和清理**
- 更新文档
- 清理旧代码
- 提交 PR

---

## 🔍 关键文件映射

| Codex 文件 | CoSurf 对应文件 | 操作 |
|-----------|----------------|------|
| `codex-rs/tui/src/streaming/controller.rs` | `src-web/src/components/layout/AIPanel.tsx` | 借鉴设计 |
| `codex-rs/core/src/session/session.rs` | `native/src/ai/stream.rs` | 重构 |
| `codex-rs/tools/src/lib.rs` | `native/src/ai/tools.rs` | 统一接口 |
| `codex-rs/core/src/session/mcp.rs` | `native/src/ai/mcp.rs` | 优化 |
| `codex-rs/state/src/` | `native/src/db/` | 参考改进 |

---

## ⚠️ 注意事项

### 许可证
- Codex 使用 **Apache-2.0** 许可证
- 可以商用，但需要保留版权声明
- 修改的代码需要注明

### 兼容性
- 保持 CoSurf 现有 API 不变
- 渐进式替换，不要一次性重写
- 提供 feature flag 切换新旧引擎

### 性能
- Codex 可能更重量级
- 需要评估对 CoSurf 的影响
- 必要时进行优化

---

## 📊 预期收益

### UI/UX
- ✅ 更直观的 Thinking 过程
- ✅ 更好的工具调用反馈
- ✅ 专业的状态指示
- ✅ 更快的交互响应

### 代码质量
- ✅ 更清晰的架构
- ✅ 更容易维护
- ✅ 更容易扩展
- ✅ 更好的测试性

### 产品能力
- ✅ 更强的代码生成
- ✅ 更好的工具集成
- ✅ 更稳定的 Agent Loop
- ✅ 更完善的状态管理

---

## 🚀 下一步行动

1. **立即开始**: 实现 ThinkingIndicator 组件
2. **并行进行**: 分析 Codex core/session.rs 的 Agent Loop
3. **持续跟进**: 根据实际效果调整方案

---

## 📝 参考资料

- [Codex GitHub](https://github.com/openai/codex)
- [Codex Documentation](https://github.com/openai/codex/tree/main/docs)
- [Ratatui Framework](https://ratatui.rs/)
- [CoSurf AI Module](file://d:\coding-harness\CoSurf\native\src\ai)
