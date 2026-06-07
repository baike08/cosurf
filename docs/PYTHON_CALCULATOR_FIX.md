# Python Calculator Skill 加载问题修复

## 📋 问题描述

用户报告导入的 python-calculator skill 无法正确加载和使用。

---

## 🔍 问题分析

### 根本原因

`ScriptConfig` 结构体缺少 `timeout` 字段，但 Markdown YAML 配置中包含该字段：

```yaml
language: python
source: |
  import sys
  ...
is_file: false
timeout: 10  # ← 这个字段被忽略了！
```

**影响**：
1. ❌ YAML 解析时 `timeout` 字段被丢弃
2. ❌ 脚本执行没有超时控制
3. ❌ 可能导致无限挂起

---

## ✅ 修复方案

### 1. 添加 timeout 字段到 ScriptConfig

**文件**: [skills.rs](file://d:\coding-harness\CoSurf\src-tauri\src\ai\skills.rs#L115-L129)

```rust
pub struct ScriptConfig {
    /// 脚本语言
    pub language: ScriptLanguage,
    /// 脚本内容或文件路径
    pub source: String,
    /// 是否为文件路径
    #[serde(default)]
    pub is_file: bool,
    /// 超时时间（秒）← 新增
    #[serde(default = "default_script_timeout")]
    pub timeout: u64,
}

fn default_script_timeout() -> u64 {
    30  // 默认 30 秒
}
```

---

### 2. 更新配置提取逻辑

**文件**: [skills.rs](file://d:\coding-harness\CoSurf\src-tauri\src\ai\skills.rs#L647-L653)

```rust
let script_config = ScriptConfig {
    language,
    source: script_data.source,
    is_file: script_data.is_file.unwrap_or(false),
    timeout: script_data.timeout.unwrap_or(30),  // ← 使用 YAML 中的值
};
```

---

### 3. 添加超时控制到所有脚本执行器

**文件**: [script.rs](file://d:\coding-harness\CoSurf\src-tauri\src\ai\skills_executors\script.rs)

#### Python 执行器

```rust
async fn execute_python_script(
    source: &str,
    is_file: bool,
    arguments: &serde_json::Value,
    timeout_secs: u64,  // ← 新增参数
) -> AppResult<String> {
    // ... 准备脚本 ...
    
    // 执行 Python（带超时）
    let output = tokio::time::timeout(
        std::time::Duration::from_secs(timeout_secs),
        tokio::process::Command::new("python3")
            .arg("-c")
            .arg(&script_content)
            .arg(&args_file)
            .output()
    )
    .await
        .map_err(|_| AppError::Internal(
            format!("Python script timed out after {}s", timeout_secs)
        ))?
        .map_err(|e| AppError::Internal(
            format!("Failed to execute Python: {}", e)
        ))?;
    
    // ... 处理结果 ...
}
```

#### JavaScript 执行器

```rust
async fn execute_js_script(
    source: &str,
    is_file: bool,
    arguments: &serde_json::Value,
    timeout_secs: u64,  // ← 新增
) -> AppResult<String> {
    let output = tokio::time::timeout(
        std::time::Duration::from_secs(timeout_secs),
        tokio::process::Command::new("node")
            .arg("-e")
            .arg(&script_content)
            .arg(&args_file)
            .output()
    )
    .await
        .map_err(|_| AppError::Internal(
            format!("JavaScript script timed out after {}s", timeout_secs)
        ))?;
    // ...
}
```

#### Bash 执行器

```rust
async fn execute_bash_script(
    source: &str,
    is_file: bool,
    _arguments: &serde_json::Value,
    timeout_secs: u64,  // ← 新增
) -> AppResult<String> {
    let output = tokio::time::timeout(
        std::time::Duration::from_secs(timeout_secs),
        tokio::process::Command::new("bash")
            .arg("-c")
            .arg(&script_content)
            .output()
    )
    .await
        .map_err(|_| AppError::Internal(
            format!("Bash script timed out after {}s", timeout_secs)
        ))?;
    // ...
}
```

#### PowerShell 执行器

```rust
async fn execute_powershell_script(
    source: &str,
    is_file: bool,
    _arguments: &serde_json::Value,
    timeout_secs: u64,  // ← 新增
) -> AppResult<String> {
    let output = tokio::time::timeout(
        std::time::Duration::from_secs(timeout_secs),
        tokio::process::Command::new("powershell")
            .arg("-Command")
            .arg(&script_content)
            .output()
    )
    .await
        .map_err(|_| AppError::Internal(
            format!("PowerShell script timed out after {}s", timeout_secs)
        ))?;
    // ...
}
```

---

### 4. 更新主执行函数

**文件**: [script.rs](file://d:\coding-harness\CoSurf\src-tauri\src\ai\skills_executors\script.rs#L10-L23)

```rust
pub async fn execute_script_skill(
    skill: &Skill,
    arguments: &serde_json::Value
) -> AppResult<String> {
    let script_config = skill.config.script.as_ref()
        .ok_or_else(|| AppError::Internal("Script config not found".to_string()))?;
    
    info!(
        language = ?script_config.language,
        timeout = script_config.timeout,  // ← 记录超时配置
        "Executing script skill"
    );
    
    match script_config.language {
        ScriptLanguage::Python => execute_python_script(
            &script_config.source,
            script_config.is_file,
            arguments,
            script_config.timeout  // ← 传递超时值
        ).await,
        ScriptLanguage::JavaScript => execute_js_script(
            &script_config.source,
            script_config.is_file,
            arguments,
            script_config.timeout
        ).await,
        ScriptLanguage::Bash => execute_bash_script(
            &script_config.source,
            script_config.is_file,
            arguments,
            script_config.timeout
        ).await,
        ScriptLanguage::PowerShell => execute_powershell_script(
            &script_config.source,
            script_config.is_file,
            arguments,
            script_config.timeout
        ).await,
    }
}
```

---

## 📊 修改的文件

1. ✅ [skills.rs](file://d:\coding-harness\CoSurf\src-tauri\src\ai\skills.rs)
   - 添加 `timeout` 字段到 `ScriptConfig`
   - 添加 `default_script_timeout()` 函数
   - 更新 `extract_script_config()` 使用 timeout

2. ✅ [script.rs](file://d:\coding-harness\CoSurf\src-tauri\src\ai\skills_executors\script.rs)
   - 更新所有执行函数签名，添加 `timeout_secs` 参数
   - 为所有命令执行添加 `tokio::time::timeout` 包装
   - 添加超时错误消息

---

## 🧪 测试验证

### 测试 1: 正常执行

```typescript
// 使用 python-calculator skill
const result = await invoke('execute_skill', {
  request: {
    skill_id: 'python-calculator',
    arguments: {
      expression: '2 + 3 * 4'
    }
  }
});

console.log(result);
// 输出: {"success": true, "result": 14, "expression": "2 + 3 * 4"}
```

**预期**：
- ✅ Skill 正确加载
- ✅ 超时配置为 10 秒（从 YAML 读取）
- ✅ 计算成功返回结果

---

### 测试 2: 超时保护

```python
# 创建无限循环脚本
infinite_loop.md:
---
id: infinite-test
name: Infinite Loop Test
type: script
enabled: true
---

# Test

```yaml
language: python
source: |
  import time
  while True:
      time.sleep(1)
is_file: false
timeout: 5
```
```

```typescript
// 执行（应该在 5 秒后超时）
const result = await invoke('execute_skill', {
  request: {
    skill_id: 'infinite-test',
    arguments: {}
  }
});

// 预期错误: "Python script timed out after 5s"
```

**预期**：
- ✅ 5 秒后自动终止
- ✅ 返回清晰的超时错误消息
- ✅ 不会无限挂起

---

### 测试 3: 默认超时

```markdown
# 没有指定 timeout 的 skill
no-timeout.md:
---
id: no-timeout
name: No Timeout
type: script
---

```yaml
language: python
source: |
  print("Hello")
is_file: false
# 没有 timeout 字段
```
```

**预期**：
- ✅ 使用默认超时 30 秒
- ✅ 正常执行

---

## 💡 技术要点

### 1. Tokio 超时机制

```rust
use tokio::time::timeout;

let result = timeout(
    Duration::from_secs(10),
    some_async_operation()
).await;

match result {
    Ok(output) => {
        // 操作在超时前完成
        handle_success(output)
    }
    Err(_) => {
        // 超时
        handle_timeout()
    }
}
```

**优势**：
- ✅ 非阻塞
- ✅ 自动取消未完成的操作
- ✅ 资源清理

---

### 2. Serde 默认值

```rust
#[serde(default = "default_script_timeout")]
pub timeout: u64,

fn default_script_timeout() -> u64 {
    30
}
```

**行为**：
- YAML 中有 `timeout` → 使用 YAML 的值
- YAML 中没有 `timeout` → 使用默认值 30

---

### 3. 错误消息标准化

```rust
.map_err(|_| AppError::Internal(
    format!("Python script timed out after {}s", timeout_secs)
))
```

**好处**：
- ✅ 清晰的错误提示
- ✅ 包含超时时长
- ✅ 便于调试

---

## 🎯 其他 Skills 的影响

此修复同时改进了所有 Script 类型的 Skills：

| Skill | 之前 | 现在 |
|-------|------|------|
| python-calculator | ❌ 无超时控制 | ✅ 10 秒超时 |
| 其他 Python skills | ❌ 可能挂起 | ✅ 30 秒默认超时 |
| JavaScript skills | ❌ 无超时 | ✅ 可配置超时 |
| Bash scripts | ❌ 无超时 | ✅ 可配置超时 |
| PowerShell scripts | ❌ 无超时 | ✅ 可配置超时 |

---

## 🚀 最佳实践

### 1. 始终设置合理的超时

```yaml
# ✅ 推荐
timeout: 10  # 简单计算

timeout: 30  # 复杂操作

timeout: 60  # 网络请求
```

### 2. 根据任务类型调整

```yaml
# 快速计算
timeout: 5

# 数据处理
timeout: 30

# API 调用
timeout: 60
```

### 3. 监控超时日志

```
INFO Executing script skill { language=Python, timeout=10 }
ERROR Python script timed out after 10s
```

---

## 📝 总结

### 修复的问题

1. ✅ **ScriptConfig 缺少 timeout 字段** - 已添加
2. ✅ **YAML timeout 被忽略** - 已正确解析
3. ✅ **脚本执行无超时控制** - 已添加 tokio::time::timeout
4. ✅ **可能无限挂起** - 现在会自动终止

### 关键改进

- 🔒 **安全性提升** - 防止无限循环和长时间运行
- ⏱️ **可控性增强** - 可配置超时时间
- 📊 **可观测性** - 记录超时配置和错误
- 🎯 **用户体验** - 清晰的错误消息

### 向后兼容

- ✅ 现有 Skills 无需修改
- ✅ 没有 timeout 字段的 Skills 使用默认值 30 秒
- ✅ API 保持不变

---

**最后更新**: 2026-05-23  
**版本**: 1.3.0  
**作者**: CoSurf Team
