# Codex CLI 集成指南

## 📋 概述

CoSurf 通过 CLI 方式调用 OpenAI Codex，避免复杂的依赖问题。使用 stdio/JSON-RPC 进行通信。

---

## 🎯 架构

```
CoSurf Electron App
    ↓ N-API
native/src/ai/codex_adapter.rs
    ↓ tokio::process::Command
Codex CLI (外部进程)
    ↓ stdio JSON-RPC
LLM API (OpenAI/Qwen/etc)
```

---

## 🔧 使用方法

### 模式 1：使用 Mock CLI（开发测试）

设置环境变量启用 mock 模式：

```bash
# Windows PowerShell
$env:CODEX_USE_MOCK="true"

# Linux/Mac
export CODEX_USE_MOCK=true
```

Mock CLI 位于：`native/src/ai/mock_codex_cli.js`

**特点：**
- ✅ 无需安装真实 Codex
- ✅ 快速测试集成逻辑
- ✅ 模拟流式响应
- ⚠️ 返回固定回复，不连接真实 LLM

---

### 模式 2：使用真实 Codex CLI

#### 步骤 1：安装 Codex CLI

```bash
# 从源码构建
cd native/src/codex/codex-rs
cargo build --release -p codex-cli

# 或者等待官方发布二进制
```

#### 步骤 2：配置路径

```bash
# 设置环境变量
$env:CODEX_BINARY_PATH="D:\coding-harness\CoSurf\native\src\codex\codex-rs\target\release\codex.exe"

# 或者添加到 PATH
```

#### 步骤 3：初始化 Agent

在 Rust 代码中：

```rust
use crate::ai::codex_adapter::{CodexAgent, CodexAgentConfig};

let config = CodexAgentConfig {
    provider: "openai".to_string(),
    model: "gpt-4o".to_string(),
    api_key: "sk-xxx".to_string(),
    base_url: None,
    cwd: "/path/to/workdir".to_string(),
    codex_home: "/path/to/codex/home".to_string(),
};

let agent = CodexAgent::new(config).await?;
let mut thread = agent.start_thread().await?;
let mut stream = agent.send_message_stream(&mut thread, "Hello").await?;

while let Some(text) = stream.recv().await {
    println!("Received: {}", text);
}
```

---

## 📊 JSON 通信格式

### 请求格式（stdin）

```json
{
  "type": "user_message",
  "content": "Your message here"
}
```

### 响应格式（stdout）

```json
{
  "type": "agent.message.content.delta",
  "content": "Streaming text chunk",
  "timestamp": "2026-07-01T00:00:00Z"
}
```

### 完成信号

```json
{
  "type": "agent.turn.completed",
  "timestamp": "2026-07-01T00:00:00Z"
}
```

---

## 🧪 测试

### 运行集成测试

```bash
cd native
$env:CODEX_USE_MOCK="true"
cargo test --lib codex_adapter_integration
```

### 手动测试 Mock CLI

```bash
cd native/src/ai
echo '{"type":"user_message","content":"Hello"}' | node mock_codex_cli.js --json
```

---

## 🛠️ 开发指南

### 修改 Mock CLI

编辑 `native/src/ai/mock_codex_cli.js`：

```javascript
function generateMockResponses(userText) {
  // 自定义回复逻辑
  return ['Response 1', 'Response 2'];
}
```

### 调整 JSON 解析

编辑 `native/src/ai/codex_adapter.rs` 中的 `extract_text_from_codex_response()`：

```rust
fn extract_text_from_codex_response(json: &Value) -> Option<String> {
    // 根据实际 Codex 输出格式调整
    if let Some(text) = json.get("content").and_then(|v| v.as_str()) {
        return Some(text.to_string());
    }
    // ...
}
```

---

## ⚠️ 注意事项

1. **Mock 模式仅用于开发测试**
   - 不要在生产环境使用
   - 回复是硬编码的

2. **真实 Codex CLI 需要 API Key**
   - 确保配置了有效的 `api_key`
   - 可能需要付费订阅

3. **进程管理**
   - CLI 进程会在 `CodexThreadHandle` drop 时自动终止
   - 避免泄漏子进程

4. **错误处理**
   - 检查 stderr 输出
   - 处理 JSON 解析失败

---

## 📂 相关文件

- [codex_adapter.rs](../native/src/ai/codex_adapter.rs) - CLI 适配器实现
- [mock_codex_cli.js](../native/src/ai/mock_codex_cli.js) - Mock CLI
- [codex_adapter_tests.rs](../native/src/ai/codex_adapter_tests.rs) - 集成测试

---

## 🚀 下一步

1. **完善 JSON 解析**
   - 研究真实 Codex CLI 的输出格式
   - 调整 `extract_text_from_codex_response()`

2. **添加工具支持**
   - 实现工具调用协议
   - 集成 Skills/MCP

3. **优化性能**
   - 减少进程启动开销
   - 复用 CLI 进程

4. **错误恢复**
   - 处理 CLI 崩溃
   - 自动重启机制

---

**最后更新**: 2026-07-01  
**状态**: ✅ 基础框架完成，待完善
