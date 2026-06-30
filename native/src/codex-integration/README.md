# Codex Core Integration

## 📦 已复制的模块

- ✅ `core/` - Agent 核心引擎 (568 files, 9.55 MB)
- ✅ `protocol/` - 协议定义 (37 files, 0.71 MB)
- ✅ `config/` - 配置管理 (61 files, 0.66 MB)

**总计**: 666 files, ~11 MB

---

## ⚠️ 下一步任务

### 1. 修复 Cargo.toml 依赖

当前 `core/Cargo.toml` 使用 workspace dependencies：
```toml
anyhow = { workspace = true }
tokio = { workspace = true }
```

需要改为具体版本或引用 CoSurf 的依赖。

### 2. 解决内部 Crate 依赖

`core` 依赖许多其他 codex crates：
```toml
codex-protocol = { workspace = true }
codex-config = { workspace = true }
codex-tools = { workspace = true }
# ... 约 50+ 个依赖
```

**选项 A**: 一起复制所有依赖的 crates
**选项 B**: 简化依赖，只保留核心功能

### 3. 创建集成入口

```rust
// native/src/ai/codex_integration/mod.rs
pub mod core;
pub mod protocol;
pub mod config;
pub mod adapter;  // 适配器层
```

### 4. 实现适配器

```rust
// native/src/ai/codex_integration/adapter.rs
use codex_core::{ThreadManager, Config};

pub struct CodexAdapter {
    thread_manager: ThreadManager,
}

impl CodexAdapter {
    pub async fn new(config: ModelConfig) -> AppResult<Self> {
        // 初始化 Codex
    }
    
    pub async fn stream_chat(...) -> AppResult<()> {
        // 调用 Codex core
        // 转发到 CoSurf callbacks
    }
}
```

---

## 🔧 快速开始

### 尝试编译（预期会失败）

```bash
cd native/src/codex-integration/core
cargo check
```

这会显示所有缺失的依赖和编译错误。

### 分析依赖树

```bash
# 查看 core 的所有依赖
grep "codex-" core/Cargo.toml
```

---

## 📊 预计工作量

- **修复依赖**: 2-3 天
- **解决编译错误**: 3-5 天
- **实现适配器**: 2-3 天
- **测试**: 2-3 天

**总计**: 9-14 天

---

## 💡 建议

由于 Codex core 依赖非常复杂（~80 个内部 crate），建议：

1. **先评估可行性** - 尝试编译，看有多少错误
2. **如果太复杂** - 考虑回退到 CLI 方案
3. **分阶段实施** - 每次只解决一部分依赖

---

## 🔗 相关文档

- [Codex Core Integration Plan](../../docs/CODEx_CORE_INTEGRATION_PLAN.md)
- [Codex Integration Progress](../../docs/CODEx_INTEGRATION_PROGRESS.md)
