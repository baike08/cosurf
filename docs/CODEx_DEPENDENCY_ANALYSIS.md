# Codex Agent Loop 依赖分析报告

## 📊 核心文件依赖统计

### 分析的文件
- `session/session.rs` (1213 行)
- `session/turn.rs` (2297 行)

---

## 🔗 内部 Crate 依赖（11个）

### P0 - 必需（无法避免）

1. **codex_protocol** ⭐⭐⭐
   - 用途：消息类型、事件类型、错误类型
   - 关键类型：
     - `Message`, `EventMsg`, `TurnItem`
     - `ResponseInputItem`, `ResponseItem`
     - `CodexErr`, `Result`
   - **估计行数**: ~2000-3000 行
   - **建议**: 必须复制或内联

2. **codex_tools** ⭐⭐
   - 用途：工具接口和注册表
   - 关键类型：
     - `ToolName`
     - 工具过滤函数
   - **估计行数**: ~500-1000 行
   - **建议**: 可以简化，只保留核心 trait

3. **codex_utils_stream_parser** ⭐⭐
   - 用途：流式响应解析
   - 关键类型：
     - `AssistantTextStreamParser`
     - `AssistantTextChunk`
   - **估计行数**: ~300-500 行
   - **建议**: 可以简化或重写

---

### P1 - 重要（可能需要）

4. **codex_extension_api**
   - 用途：扩展 API 接口
   - 关键类型：
     - `TurnInputContext`
     - `TurnInputEnvironment`
   - **估计行数**: ~200-400 行

5. **codex_core_skills**
   - 用途：Skills 注入
   - 关键类型：
     - `InjectedHostSkillPrompts`
   - **估计行数**: ~100-200 行

6. **codex_features**
   - 用途：功能开关
   - 关键类型：
     - `Feature`
   - **估计行数**: ~50-100 行

---

### P2 - 可选（可以移除）

7. **codex_analytics**
   - 用途：埋点和监控
   - **建议**: 可以完全移除或用 stub 替代

8. **codex_async_utils**
   - 用途：异步工具函数
   - **建议**: 可以用标准库替代

9. **codex_core_plugins**
   - 用途：插件系统
   - **建议**: 可以移除

10. **codex_git_utils**
    - 用途：Git 相关工具
    - **建议**: 可以移除

11. **codex_login**
    - 用途：登录认证
    - **建议**: 可以移除

---

## 🎯 提取策略

### 方案 A：最小化提取（推荐）

**目标**：只提取 P0 依赖，移除或简化 P1/P2

#### 需要复制的代码量估算

```
codex_protocol:           ~2500 行
codex_tools:              ~800 行
codex_utils_stream_parser: ~400 行
-----------------------------------
总计:                     ~3700 行
```

#### 需要创建的 stubs

```rust
// codex_analytics stub
pub fn build_track_events_context() {}
pub enum CompactionPhase { /* empty */ }

// codex_async_utils stub
trait OrCancelExt { /* empty impl */ }

// codex_extension_api stub
struct TurnInputContext { /* minimal fields */ }

// ... 其他 stubs
```

**预计工作量**: 3-5 天

---

### 方案 B：完整提取

**目标**：复制所有 11 个 crates 的核心代码

#### 需要复制的代码量估算

```
P0 crates:                ~3700 行
P1 crates:                ~800 行
P2 crates (stubs):        ~500 行
-----------------------------------
总计:                     ~5000 行
```

**预计工作量**: 5-8 天

---

### 方案 C：内联简化（最激进）

**目标**：不创建独立的 crates，直接将所有类型内联到 `codex-extracted/src/lib.rs`

#### 结构

```rust
// codex-extracted/src/lib.rs

// === Protocol Types ===
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventMsg {
    // ...
}

// === Tool Types ===
pub trait Tool {
    fn name(&self) -> &str;
    // ...
}

// === Stream Parser ===
pub struct AssistantTextStreamParser {
    // ...
}

// === Stubs ===
// 空的或最小化的类型定义
```

**优点**：
- ✅ 最简单，无需管理多个 crate
- ✅ 编译快速
- ✅ 易于理解

**缺点**：
- ⚠️ 代码组织较差
- ⚠️ 与原始 Codex 难以同步

**预计工作量**: 2-3 天

---

## 🛠️ 推荐实施方案

### 阶段 1：创建 Cargo.toml（今天）

```toml
[package]
name = "codex-extracted"
version = "0.1.0"
edition = "2021"

[dependencies]
# 外部依赖
anyhow = "1.0"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1.0"
tracing = "0.1"
futures = "0.3"
async-trait = "0.1"

# CoSurf 已有的依赖
rusqlite = { version = "0.31", features = ["bundled"] }
once_cell = "1.19"
```

---

### 阶段 2：创建 protocol stubs（明天）

```bash
mkdir native/src/codex-extracted/protocol
```

创建最小化的协议类型：

```rust
// protocol/mod.rs

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: Vec<ContentItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentItem {
    Text { text: String },
    // ... 其他类型
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventMsg {
    AgentMessageContentDelta(AgentMessageContentDeltaEvent),
    // ... 其他事件
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessageContentDeltaEvent {
    pub delta: String,
}

pub type Result<T> = std::result::Result<T, CodexErr>;

#[derive(Debug, thiserror::Error)]
pub enum CodexErr {
    #[error("Internal error: {0}")]
    Internal(String),
    // ... 其他错误
}
```

---

### 阶段 3：创建 tools stubs（后天）

```rust
// tools/mod.rs

pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
}

pub struct ToolName(pub String);
```

---

### 阶段 4：创建 stream parser（第 4 天）

```rust
// stream_parser/mod.rs

pub struct AssistantTextStreamParser {
    buffer: String,
}

impl AssistantTextStreamParser {
    pub fn new() -> Self {
        Self { buffer: String::new() }
    }
    
    pub fn parse_chunk(&mut self, chunk: &str) -> Option<AssistantTextChunk> {
        // 简化实现
        Some(AssistantTextChunk {
            text: chunk.to_string(),
        })
    }
}

pub struct AssistantTextChunk {
    pub text: String,
}
```

---

### 阶段 5：修改 session.rs 和 turn.rs（第 5-6 天）

替换所有 `use codex_*` 为本地导入：

```rust
// 原本
use codex_protocol::models::Message;
use codex_tools::Tool;

// 改为
use crate::protocol::Message;
use crate::tools::Tool;
```

---

### 阶段 6：编译和修复（第 7-8 天）

```bash
cd native/src/codex-extracted
cargo check

# 根据错误逐个修复
```

---

## ⏱️ 总预估工作量

| 阶段 | 任务 | 预计时间 |
|------|------|---------|
| 1 | 创建 Cargo.toml | 0.5 天 |
| 2 | Protocol stubs | 1 天 |
| 3 | Tools stubs | 0.5 天 |
| 4 | Stream parser | 0.5 天 |
| 5 | 修改 imports | 1-2 天 |
| 6 | 编译和修复 | 2-3 天 |
| **总计** | | **5.5-7.5 天** |

---

## ⚠️ 风险评估

### 高风险

1. **Protocol 类型过多**
   - `codex_protocol` 可能有数百个类型
   - 可能遗漏关键类型导致编译失败

2. **深层依赖链**
   - 一个类型可能依赖多个其他类型
   - 形成递归依赖

3. **编译错误数量**
   - 可能有数百个错误
   - 修复耗时超出预期

---

### 缓解措施

1. **分阶段编译**
   - 每次只解决一类错误
   - 使用 `#[allow(dead_code)]` 暂时忽略未使用的代码

2. **优先保证核心路径**
   - 先让 `turn()` 方法能编译通过
   - 非关键功能用 `todo!()` 占位

3. **及时止损**
   - 如果超过 3 天仍有大量错误
   - 回退到 CLI 调用方案

---

## 💡 最终建议

**如果选择性提取在 3 天内无法编译通过，强烈建议回退到 CLI 方案。**

CLI 方案：
- ✅ 已完成 80%
- ✅ 无依赖问题
- ✅ 预计 1-2 天即可完成

选择性提取：
- ⚠️ 风险高
- ⚠️ 工作量大
- ⚠️ 维护成本高

---

## 🚀 下一步行动

1. **立即开始**：创建 `Cargo.toml`
2. **今天完成**：创建 protocol stubs
3. **明天尝试编译**：查看错误数量
4. **决策点**：如果错误 > 50 个，考虑回退
