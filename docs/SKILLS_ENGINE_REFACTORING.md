# Skills 执行引擎重构指南

## 📋 概述

参考 **Claude Code Skills** 的设计理念，对 CoSurf 的 Skills 系统进行了全面重构，实现了：

1. **统一的执行引擎** - `SkillsEngine` 作为核心调度器
2. **插件化架构** - 基于 trait 的执行器扩展机制
3. **标准化结果** - `SkillExecutionResult` 统一返回格式
4. **参数验证** - 支持 JSON Schema 验证（预留接口）
5. **性能监控** - 自动记录执行时间

---

## 🏗️ 架构设计

### Claude Code Skills 核心理念

```
┌─────────────────────────────────────────┐
│         Claude Code Skills              │
├─────────────────────────────────────────┤
│ 1. Declarative Definition (YAML)        │
│ 2. Tool-like Interface                  │
│ 3. Sandboxed Execution                  │
│ 4. Parameter Validation                 │
│ 5. Standardized Output                  │
└─────────────────────────────────────────┘
```

### CoSurf 实现

```
┌──────────────────────────────────────────────────────┐
│                  SkillsEngine                        │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐            │
│  │ CLI      │ │ Script   │ │ MCP      │  Executors  │
│  │Executor  │ │Executor  │ │Executor  │            │
│  └──────────┘ └──────────┘ └──────────┘            │
└──────────────────────────────────────────────────────┘
         ▲                ▲                ▲
         │                │                │
    ┌────┴────┐     ┌────┴────┐     ┌────┴────┐
    │ Skill A │     │ Skill B │     │ Skill C │
    │ (CLI)   │     │(Script) │     │ (MCP)   │
    └─────────┘     └─────────┘     └─────────┘
```

---

## 📁 文件结构

### 重构前

```
src-tauri/src/ai/
├── skills.rs              # 960 行，包含所有逻辑
└── skills_executors/
    ├── cli.rs             # CLI 执行
    ├── script.rs          # Script 执行
    └── mcp.rs             # MCP 执行
```

### 重构后

```
src-tauri/src/ai/
├── skills.rs              # ~720 行，管理逻辑
├── skills_engine.rs       # ✨ 新增：执行引擎（270 行）
└── skills_executors/
    ├── mod.rs
    ├── cli.rs             # CLI 执行
    ├── script.rs          # Script 执行
    └── mcp.rs             # MCP 执行
```

---

## 🔧 核心组件

### 1. SkillsEngine（执行引擎）

**位置**: `src-tauri/src/ai/skills_engine.rs`

#### 主要特性

```rust
pub struct SkillsEngine {
    executors: HashMap<SkillType, Box<dyn SkillExecutor>>,
}
```

**职责**：
- ✅ 注册和管理执行器
- ✅ 统一调度 Skill 执行
- ✅ 自动计时和日志记录
- ✅ 错误处理和结果标准化

#### 关键方法

```rust
impl SkillsEngine {
    /// 创建新引擎并注册默认执行器
    pub fn new() -> Self
    
    /// 注册自定义执行器
    pub fn register_executor(&mut self, executor: Box<dyn SkillExecutor>)
    
    /// 执行 Skill（带性能监控）
    pub async fn execute_skill(
        &self,
        skill: &Skill,
        arguments: &serde_json::Value,
    ) -> AppResult<SkillExecutionResult>
}
```

---

### 2. SkillExecutor Trait（执行器接口）

```rust
#[async_trait::async_trait]
pub trait SkillExecutor: Send + Sync {
    /// 执行 Skill
    async fn execute(
        &self,
        skill: &Skill,
        arguments: &serde_json::Value,
    ) -> AppResult<SkillExecutionResult>;
    
    /// 获取执行器类型
    fn executor_type(&self) -> SkillType;
    
    /// 验证参数（可选实现）
    fn validate_arguments(
        &self,
        skill: &Skill,
        arguments: &serde_json::Value,
    ) -> AppResult<()> {
        // 默认实现
        Ok(())
    }
}
```

**优势**：
- 🎯 **开闭原则** - 易于扩展新类型
- 🔒 **类型安全** - 编译时检查
- 🧪 **可测试** - 每个执行器独立测试

---

### 3. SkillExecutionResult（标准化结果）

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillExecutionResult {
    /// 是否成功
    pub success: bool,
    
    /// 输出内容
    pub output: String,
    
    /// 错误信息（如果失败）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    
    /// 执行时间（毫秒）
    pub duration_ms: u64,
    
    /// 元数据
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}
```

**使用示例**：

```rust
// 成功结果
let result = SkillExecutionResult::success(
    "Command output".to_string(),
    150  // ms
);

// 失败结果
let result = SkillExecutionResult::failure(
    "Permission denied".to_string(),
    50  // ms
);
```

---

### 4. 内置执行器

#### CLI Executor

```rust
struct CliExecutor;

#[async_trait::async_trait]
impl SkillExecutor for CliExecutor {
    async fn execute(...) -> AppResult<SkillExecutionResult> {
        use crate::ai::skills_executors::cli::execute_cli_skill;
        
        let output = execute_cli_skill(skill, arguments).await?;
        Ok(SkillExecutionResult::success(output.trim().to_string(), 0))
    }
    
    fn executor_type(&self) -> SkillType {
        SkillType::Cli
    }
}
```

**特点**：
- 委托给现有的 `execute_cli_skill` 函数
- 自动去除首尾空白
- 计时由引擎统一管理

#### Script Executor

```rust
struct ScriptExecutor;

#[async_trait::async_trait]
impl SkillExecutor for ScriptExecutor {
    async fn execute(...) -> AppResult<SkillExecutionResult> {
        use crate::ai::skills_executors::script::execute_script_skill;
        
        let output = execute_script_skill(skill, arguments).await?;
        Ok(SkillExecutionResult::success(output.trim().to_string(), 0))
    }
    
    fn executor_type(&self) -> SkillType {
        SkillType::Script
    }
}
```

**支持的脚本语言**：
- Python
- JavaScript
- Bash
- PowerShell

#### MCP Executor

```rust
struct McpExecutor;

#[async_trait::async_trait]
impl SkillExecutor for McpExecutor {
    async fn execute(...) -> AppResult<SkillExecutionResult> {
        use crate::ai::skills_executors::mcp::execute_mcp_skill;
        
        let output = execute_mcp_skill(skill, arguments).await?;
        Ok(SkillExecutionResult::success(output, 0))
    }
    
    fn executor_type(&self) -> SkillType {
        SkillType::Mcp
    }
}
```

**API Key 处理**：
```rust
// 支持环境变量替换
let api_key = if key_template.starts_with("${") {
    std::env::var(env_var).ok()  // ${ALIBABA_CLOUD_API_KEY}
} else {
    Some(key_template.clone())   // sk-xxxxx
};
```

#### Built-in Executor

```rust
struct BuiltInExecutor;

#[async_trait::async_trait]
impl SkillExecutor for BuiltInExecutor {
    async fn execute(...) -> AppResult<SkillExecutionResult> {
        Err(AppError::Internal(
            "Built-in skills should be handled by main agent loop"
        ))
    }
    
    fn executor_type(&self) -> SkillType {
        SkillType::BuiltIn
    }
}
```

**说明**：内置工具由 Agent 主循环直接处理，不通过 Skills 引擎。

---

## 🔄 执行流程

### 完整流程图

```
用户请求执行 Skill
         ↓
SkillsManager::execute_skill()
         ↓
检查 Skill 是否存在且启用
         ↓
SkillsEngine::execute_skill()
         ↓
┌─────────────────────────────┐
│ 1. 记录开始时间              │
│ 2. 查找对应执行器            │
│ 3. 验证参数（可选）          │
│ 4. 调用执行器                │
│ 5. 捕获错误并转换为结果      │
│ 6. 记录执行时间              │
│ 7. 记录日志                  │
└─────────────────────────────┘
         ↓
SkillExecutionResult
         ↓
返回给调用者
```

### 代码示例

```rust
// 在 SkillsManager 中
pub async fn execute_skill(&self, skill_id: &str, arguments: &serde_json::Value) -> AppResult<String> {
    let skill = self.skills.get(skill_id)
        .ok_or_else(|| AppError::Internal(format!("Skill not found: {}", skill_id)))?;
    
    if !skill.enabled {
        return Err(AppError::Internal(format!("Skill is disabled: {}", skill.name)));
    }
    
    // 使用执行引擎
    let result = self.engine.execute_skill(skill, arguments).await?;
    
    if result.success {
        Ok(result.output)
    } else {
        Err(AppError::Internal(result.error.unwrap_or_else(|| "Unknown error".to_string())))
    }
}
```

---

## 📊 性能监控

### 自动计时

引擎自动记录每个 Skill 的执行时间：

```rust
let start_time = std::time::Instant::now();

// ... 执行逻辑 ...

result.duration_ms = start_time.elapsed().as_millis() as u64;
```

### 日志输出

```
INFO Executing skill { skill_id="echo-skill", skill_name="Echo 消息", skill_type=Cli }
INFO Skill execution completed { skill_id="echo-skill", success=true, duration_ms=45 }
```

或失败时：

```
ERROR Skill execution failed { skill_id="test-skill", error="Permission denied", duration_ms=12 }
```

---

## 🧩 扩展新执行器

### 步骤 1: 实现 SkillExecutor trait

```rust
use async_trait::async_trait;
use crate::ai::skills_engine::{SkillExecutor, SkillExecutionResult};
use crate::ai::skills::{Skill, SkillType};

struct MyCustomExecutor;

#[async_trait::async_trait]
impl SkillExecutor for MyCustomExecutor {
    async fn execute(
        &self,
        skill: &Skill,
        arguments: &serde_json::Value,
    ) -> AppResult<SkillExecutionResult> {
        // 你的执行逻辑
        let output = my_custom_logic(skill, arguments).await?;
        
        Ok(SkillExecutionResult::success(output, 0))
    }
    
    fn executor_type(&self) -> SkillType {
        SkillType::Custom  // 需要先在 SkillType enum 中添加
    }
}
```

### 步骤 2: 注册执行器

```rust
// 在 SkillsEngine::new() 中添加
engine.register_executor(Box::new(MyCustomExecutor));
```

### 步骤 3: 添加 SkillType

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SkillType {
    Cli,
    Mcp,
    BuiltIn,
    Script,
    Custom,  // ← 新增
}
```

---

## 🔍 与 Claude Code 对比

| 特性 | Claude Code | CoSurf (重构后) |
|------|-------------|-----------------|
| 声明式定义 | ✅ YAML frontmatter | ✅ Markdown + YAML |
| 工具化接口 | ✅ Tool-like API | ✅ SkillExecutor trait |
| 沙箱执行 | ⚠️ 部分支持 | ⚠️ 基础超时控制 |
| 参数验证 | ✅ JSON Schema | 🔜 预留接口 |
| 标准化输出 | ✅ 统一格式 | ✅ SkillExecutionResult |
| 性能监控 | ✅ 内置 | ✅ 自动计时 |
| 插件扩展 | ✅ 可扩展 | ✅ 执行器注册 |
| 错误处理 | ✅ 结构化 | ✅ 统一错误格式 |

---

## 💡 设计优势

### 1. 单一职责原则

- **skills.rs**: 只负责管理（CRUD、加载、验证）
- **skills_engine.rs**: 只负责调度和监控
- **skills_executors/**: 只负责具体执行

### 2. 开闭原则

- ✅ 对扩展开放：轻松添加新执行器
- ❌ 对修改封闭：无需改动现有代码

### 3. 依赖倒置

```rust
// 高层模块依赖抽象
engine.executors: HashMap<SkillType, Box<dyn SkillExecutor>>

// 而非具体实现
// engine.cli_executor: CliExecutor  ❌
```

### 4. 可测试性

每个组件都可以独立测试：

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_cli_executor() {
        let executor = CliExecutor;
        let skill = create_test_skill();
        let args = serde_json::json!({});
        
        let result = executor.execute(&skill, &args).await.unwrap();
        assert!(result.success);
    }
}
```

---

## 🚀 未来优化方向

### 1. 参数验证（JSON Schema）

```rust
fn validate_arguments(
    &self,
    skill: &Skill,
    arguments: &serde_json::Value,
) -> AppResult<()> {
    if let Some(schema) = skill.config.parameters.as_object() {
        // 使用 jsonschema crate 验证
        let validator = jsonschema::validator_for(schema)?;
        if let Err(errors) = validator.validate(arguments) {
            return Err(AppError::InvalidInput(
                errors.map(|e| e.to_string()).collect::<Vec<_>>().join(", ")
            ));
        }
    }
    Ok(())
}
```

### 2. 沙箱执行

```rust
// 使用 seccomp 限制系统调用（Linux）
// 或使用 Windows Job Objects
let sandbox_config = SandboxConfig {
    allowed_syscalls: vec![SYS_read, SYS_write],
    max_memory_mb: 256,
    timeout_secs: 30,
};

execute_in_sandbox(sandbox_config, || {
    // 执行代码
}).await
```

### 3. 缓存执行结果

```rust
use moka::future::Cache;

pub struct SkillsEngine {
    executors: HashMap<SkillType, Box<dyn SkillExecutor>>,
    cache: Cache<String, SkillExecutionResult>,  // 缓存键 -> 结果
}

impl SkillsEngine {
    async fn execute_skill(...) -> AppResult<SkillExecutionResult> {
        let cache_key = format!("{}:{:?}", skill.id, arguments);
        
        // 尝试从缓存读取
        if let Some(cached) = self.cache.get(&cache_key).await {
            return Ok(cached);
        }
        
        // 执行并缓存
        let result = executor.execute(skill, arguments).await?;
        self.cache.insert(cache_key, result.clone()).await;
        
        Ok(result)
    }
}
```

### 4. 并发限流

```rust
use tokio::sync::Semaphore;

pub struct SkillsEngine {
    executors: HashMap<SkillType, Box<dyn SkillExecutor>>,
    semaphore: Arc<Semaphore>,  // 限制并发数
}

impl SkillsEngine {
    async fn execute_skill(...) -> AppResult<SkillExecutionResult> {
        let permit = self.semaphore.acquire().await?;
        
        let result = executor.execute(skill, arguments).await?;
        
        drop(permit);  // 释放许可
        Ok(result)
    }
}
```

### 5. 指标收集

```rust
use prometheus::{Counter, Histogram};

pub struct SkillsMetrics {
    executions_total: Counter,
    execution_duration: Histogram,
    failures_total: Counter,
}

impl SkillsEngine {
    async fn execute_skill(...) -> AppResult<SkillExecutionResult> {
        let start = Instant::now();
        
        let result = executor.execute(skill, arguments).await?;
        
        // 记录指标
        self.metrics.executions_total.inc();
        self.metrics.execution_duration.observe(start.elapsed().as_secs_f64());
        
        if !result.success {
            self.metrics.failures_total.inc();
        }
        
        Ok(result)
    }
}
```

---

## 📝 迁移指南

### 从旧版本升级

**无需迁移！** 重构是向后兼容的：

- ✅ API 保持不变
- ✅ 现有 Skills 继续工作
- ✅ 前端无需修改

### 代码变更

如果你直接调用了执行器函数：

```rust
// ❌ 旧方式（仍然可用）
let output = crate::ai::skills_executors::execute_cli_skill(&skill, &args).await?;

// ✅ 新方式（推荐）
let result = engine.execute_skill(&skill, &args).await?;
if result.success {
    println!("{}", result.output);
}
```

---

## 🧪 测试策略

### 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_engine_creation() {
        let engine = SkillsEngine::new();
        assert_eq!(engine.executors.len(), 4);  // CLI, Script, MCP, BuiltIn
    }
    
    #[tokio::test]
    async fn test_execute_cli_skill() {
        let engine = SkillsEngine::new();
        let skill = create_echo_skill();
        let args = serde_json::json!({"message": "Hello"});
        
        let result = engine.execute_skill(&skill, &args).await.unwrap();
        
        assert!(result.success);
        assert!(result.output.contains("Hello"));
        assert!(result.duration_ms > 0);
    }
}
```

### 集成测试

```rust
#[tokio::test]
async fn test_full_skill_lifecycle() {
    // 1. 导入 Skill
    let markdown = include_str!("../../examples/echo-skill.md");
    let mut manager = SkillsManager::new(temp_dir());
    let skill = manager.import_skill_from_markdown(markdown).unwrap();
    
    // 2. 执行 Skill
    let args = serde_json::json!({"message": "Test"});
    let output = manager.execute_skill(&skill.id, &args).await.unwrap();
    
    // 3. 验证结果
    assert!(output.contains("Test"));
}
```

---

## 📚 参考资料

- [Claude Code Skills Documentation](https://docs.anthropic.com/en/docs/claude-code/skills)
- [Model Context Protocol](https://modelcontextprotocol.io/)
- [Rust Async Book](https://rust-lang.github.io/async-book/)
- [Design Patterns in Rust](https://rust-unofficial.github.io/patterns/)

---

**最后更新**: 2026-05-23  
**版本**: 2.0.0  
**作者**: CoSurf Team
