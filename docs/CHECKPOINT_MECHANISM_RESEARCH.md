# Agent Loop Checkpoint 机制方案调研与选型

## 📋 目录

- [1. 问题背景](#1-问题背景)
- [2. 核心挑战](#2-核心挑战)
- [3. 业界方案调研](#3-业界方案调研)
- [4. 方案对比分析](#4-方案对比分析)
- [5. CoSurf 推荐方案](#5-cosurf-推荐方案)
- [6. 实施路线图](#6-实施路线图)

---

## 1. 问题背景

### 1.1 当前痛点

在 CoSurf 的 Agent Loop 中，多任务并行执行面临以下稳定性问题：

```rust
// 当前实现（stream.rs:209-263）
let tool_results = futures::future::join_all(futures).await;

for result in &tool_results {
    if let Ok((tool_call, tool_result)) = result {
        current_messages.push(tool_msg);  // ❌ 无回滚机制
    }
}
```

**问题场景：**

1. **部分失败导致状态不一致**
   ```
   用户："打开知乎、百度、淘宝，导出总结"
   
   执行流程：
   ✅ open_url(知乎) → 成功
   ✅ open_url(百度) → 成功
   ❌ open_url(淘宝) → 网络超时
   ✅ summarize_page(知乎) → 成功
   ✅ summarize_page(百度) → 成功
   
   结果：上下文包含 2 个成功的工具结果，但缺少淘宝的数据
   LLM 可能基于不完整的信息做出错误决策
   ```

2. **写入操作无法回滚**
   ```
   用户："搜索 AI 趋势，生成报告并保存"
   
   执行流程：
   ✅ mcp_search("AI 趋势") → 成功
   ✅ export_markdown("report.md") → 成功
   ❌ run_command("git commit") → 权限错误
   
   结果：文件已写入磁盘，但 Git 提交失败
   用户看到不完整的状态，需要手动清理
   ```

3. **长时间运行的任务中断后无法恢复**
   ```
   用户："分析 50 个网页，生成综合报告"
   
   执行到第 30 个网页时，用户关闭应用或网络中断
   重启后，Agent 需要从头开始，浪费时间和 API 配额
   ```

---

## 2. 核心挑战

### 2.1 技术挑战

| 挑战 | 说明 | 影响 |
|------|------|------|
| **原子性** | 多个工具调用要么全部成功，要么全部回滚 | 状态一致性 |
| **幂等性** | 重复执行相同操作不应产生副作用 | 重试安全性 |
| **持久化** | Checkpoint 需要持久化存储，防止进程崩溃丢失 | 故障恢复 |
| **性能开销** | Checkpoint 不应显著降低执行速度 | 用户体验 |
| **复杂度** | 实现不应过度复杂，易于维护 | 开发成本 |

### 2.2 业务挑战

- **用户期望**：Agent 应该可靠地完成任务，即使遇到临时错误
- **成本控制**：避免重复调用付费 API（如 MCP Server）
- **数据安全**：写入操作失败时应自动清理残留数据

---

## 3. 业界方案调研

### 3.1 OpenClaw - 快照保护机制

#### 核心设计

```typescript
// OpenClaw 伪代码
class AgentLoop {
  private snapshots: Map<string, Snapshot> = new Map();
  
  async executeWithCheckpoint(taskId: string, tools: ToolCall[]) {
    // 1. 创建快照
    const snapshot = await this.createSnapshot();
    this.snapshots.set(taskId, snapshot);
    
    try {
      // 2. 执行工具
      const results = await Promise.all(tools.map(execute));
      
      // 3. 验证结果
      if (!this.validateResults(results)) {
        throw new Error("Validation failed");
      }
      
      // 4. 提交更改
      await this.commitChanges(results);
      
      // 5. 清理快照
      this.snapshots.delete(taskId);
    } catch (error) {
      // 6. 回滚到快照
      await this.rollbackToSnapshot(snapshot);
      throw error;
    }
  }
  
  async createSnapshot(): Promise<Snapshot> {
    return {
      messages: [...this.currentMessages],
      filesystem: await this.captureFilesystemState(),
      browserTabs: await this.captureBrowserState(),
      timestamp: Date.now(),
    };
  }
  
  async rollbackToSnapshot(snapshot: Snapshot) {
    this.currentMessages = [...snapshot.messages];
    await this.restoreFilesystemState(snapshot.filesystem);
    await this.restoreBrowserState(snapshot.browserTabs);
  }
}
```

#### 优势
- ✅ 完整的状态回滚能力
- ✅ 支持文件系统、浏览器状态等多维度快照
- ✅ 适用于复杂的多步骤任务

#### 劣势
- ❌ 快照创建开销大（尤其是文件系统和浏览器状态）
- ❌ 内存占用高（需要保存完整状态副本）
- ❌ 实现复杂度高（需要追踪所有可变状态）

#### 适用场景
- 需要强一致性的写入密集型任务
- 长周期任务（>5 分钟）
- 对数据完整性要求极高的场景

---

### 3.2 Claude Code - 增量更新 + 自动回滚

#### 核心设计

```typescript
// Claude Code 伪代码
class AgentLoop {
  private changeLog: ChangeLog = new ChangeLog();
  
  async executeTools(tools: ToolCall[]) {
    // 1. 记录变更日志
    for (const tool of tools) {
      this.changeLog.startRecording(tool.id);
    }
    
    // 2. 并行执行
    const results = await Promise.allSettled(tools.map(execute));
    
    // 3. 检查失败
    const failures = results.filter(r => r.status === 'rejected');
    
    if (failures.length > 0) {
      // 4. 自动回滚失败的写入操作
      await this.rollbackFailedWrites(failures);
      
      // 5. 保留成功的读取操作
      const successes = results.filter(r => r.status === 'fulfilled');
      await this.applySuccessfulReads(successes);
    } else {
      // 6. 全部成功，提交变更
      await this.commitAllChanges(results);
    }
  }
  
  async rollbackFailedWrites(failures: any[]) {
    for (const failure of failures) {
      const tool = failure.tool;
      if (tool.category === 'Write') {
        // 删除创建的文件
        if (tool.action === 'create_file') {
          await fs.unlink(tool.outputPath);
        }
        // 撤销修改的文件（从备份恢复）
        if (tool.action === 'modify_file') {
          await fs.copyFile(tool.backupPath, tool.outputPath);
        }
      }
    }
  }
}
```

#### 优势
- ✅ 轻量级（只记录变更，不保存完整快照）
- ✅ 针对性回滚（只回滚失败的写入操作）
- ✅ 保留成功的读取操作（提高效率）

#### 劣势
- ❌ 需要预先备份文件（增加 I/O 开销）
- ❌ 浏览器状态难以回滚（标签页、DOM 变化）
- ❌ 依赖工具的幂等性设计

#### 适用场景
- 以读取为主的混合任务
- 文件操作频繁的场景
- 中等长度任务（1-5 分钟）

---

### 3.3 LangGraph - 状态图 + 检查点持久化

#### 核心设计

```python
# LangGraph 伪代码
from langgraph.checkpoint import MemorySaver, SqliteSaver

# 1. 定义状态图
workflow = StateGraph(AgentState)
workflow.add_node("execute_tools", execute_tools_node)
workflow.add_node("validate", validate_node)
workflow.add_edge("execute_tools", "validate")

# 2. 配置检查点（支持多种后端）
checkpointer = SqliteSaver.from_conn_string("checkpoints.db")
# 或者使用 Redis、PostgreSQL 等

# 3. 编译并运行
app = workflow.compile(checkpointer=checkpointer)
thread_id = "unique_thread_id"

# 4. 执行（自动保存检查点）
result = app.invoke(
    {"messages": [...]},
    config={"configurable": {"thread_id": thread_id}}
)

# 5. 从检查点恢复
last_checkpoint = checkpointer.get({"configurable": {"thread_id": thread_id}})
if last_checkpoint:
    result = app.invoke(
        last_checkpoint.state,
        config={"configurable": {"thread_id": thread_id}}
    )
```

#### 优势
- ✅ 持久化存储（SQLite/Redis/PostgreSQL）
- ✅ 支持中断后恢复（进程重启后继续执行）
- ✅ 可视化调试（可以查看每个节点的状态）
- ✅ 成熟的生态系统

#### 劣势
- ❌ Python 生态（CoSurf 是 Rust）
- ❌ 学习曲线陡峭（需要理解状态图概念）
- ❌ 性能开销（每次状态变更都写入数据库）

#### 适用场景
- 超长周期任务（>10 分钟）
- 需要人工介入的工作流
- 复杂的分支逻辑

---

### 3.4 Temporal - 工作流引擎 + 确定性重放

#### 核心设计

```typescript
// Temporal 伪代码
import { proxyActivities } from '@temporalio/workflow';

const { executeTool } = proxyActivities({
  startToCloseTimeout: '5 minutes',
  retry: {
    maximumAttempts: 3,
    backoffCoefficient: 2,
  },
});

export async function agentWorkflow(tools: ToolCall[]) {
  const results = [];
  
  // Temporal 保证这段代码可以确定性重放
  for (const tool of tools) {
    try {
      // 如果之前已经执行过，Temporal 会跳过
      const result = await executeTool(tool);
      results.push(result);
    } catch (error) {
      // 自动重试（最多 3 次）
      throw error;
    }
  }
  
  return results;
}
```

#### 优势
- ✅ 极强的可靠性（分布式系统级别）
- ✅ 自动重试和超时处理
- ✅ 确定性重放（无需手动管理状态）
- ✅ 支持长时间运行的工作流（数天甚至数月）

#### 劣势
- ❌ 架构复杂（需要部署 Temporal Server）
- ❌ 学习成本高（需要理解工作流引擎概念）
- ❌ 过度设计（对于桌面应用来说太重）

#### 适用场景
- 分布式微服务架构
- 关键业务逻辑（金融、医疗等）
- 超长周期任务（数小时以上）

---

## 4. 方案对比分析

### 4.1 功能对比

| 特性 | OpenClaw | Claude Code | LangGraph | Temporal | **CoSurf 需求** |
|------|----------|-------------|-----------|----------|----------------|
| **原子性** | ✅ 完整回滚 | ⚠️ 部分回滚 | ✅ 状态回滚 | ✅ 确定性重放 | ⚠️ 部分回滚 |
| **持久化** | ❌ 内存 | ❌ 内存 | ✅ SQLite/Redis | ✅ 专用存储 | ✅ SQLite |
| **性能开销** | 🔴 高 | 🟡 中 | 🔴 高 | 🟡 中 | 🟢 低 |
| **实现复杂度** | 🔴 高 | 🟡 中 | 🔴 高 | 🔴 高 | 🟢 低 |
| **中断恢复** | ❌ 不支持 | ❌ 不支持 | ✅ 支持 | ✅ 支持 | ⚠️ 可选 |
| **浏览器回滚** | ✅ 支持 | ❌ 不支持 | ❌ 不支持 | ❌ 不支持 | ❌ 不需要 |
| **文件回滚** | ✅ 支持 | ✅ 支持 | ⚠️ 需自定义 | ⚠️ 需自定义 | ✅ 需要 |

### 4.2 适用性评估

#### CoSurf 的典型场景

1. **短时任务（<1 分钟）**：占比 70%
   - 单个页面总结
   - 简单搜索查询
   - 单文件导出

2. **中等任务（1-5 分钟）**：占比 25%
   - 多页面总结（3-5 个）
   - 搜索 + 导出组合
   - 批量文件操作

3. **长时任务（>5 分钟）**：占比 5%
   - 大规模网页分析（>10 个）
   - 复杂的多步骤工作流

#### 方案匹配度

| 方案 | 短时任务 | 中等任务 | 长时任务 | 综合评分 |
|------|---------|---------|---------|---------|
| OpenClaw | 🟡 过度设计 | ✅ 合适 | ✅ 合适 | 7/10 |
| Claude Code | ✅ 轻量 | ✅ 合适 | 🟡 不足 | 8/10 |
| LangGraph | 🔴 过重 | 🟡 合适 | ✅ 合适 | 6/10 |
| Temporal | 🔴 过重 | 🔴 过重 | ✅ 合适 | 4/10 |

---

## 5. CoSurf 推荐方案

### 5.1 方案设计原则

基于 CoSurf 的实际需求和场景分布，推荐采用 **"轻量级增量检查点"** 方案，结合 Claude Code 和 OpenClaw 的优点：

**核心原则：**
1. **按需持久化**：只在必要时保存检查点（中等以上任务）
2. **增量记录**：只记录变更，不保存完整快照
3. **分类回滚**：区分读取操作（不回滚）和写入操作（回滚）
4. **SQLite 存储**：利用现有的 SQLite 基础设施

### 5.2 架构设计

```rust
// native/src/ai/checkpoint.rs

/// 检查点管理器
pub struct CheckpointManager {
    db_connection: rusqlite::Connection,
    current_checkpoint_id: Option<String>,
}

/// 检查点数据结构
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Checkpoint {
    pub id: String,
    pub conversation_id: String,
    pub iteration: u32,
    pub timestamp: i64,
    
    // 消息历史（增量）
    pub new_messages: Vec<ChatMessage>,
    
    // 文件系统变更（增量）
    pub file_changes: Vec<FileChange>,
    
    // 工具执行状态
    pub tool_results: Vec<ToolResultRecord>,
}

/// 文件变更记录
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum FileChange {
    Created { path: String },
    Modified { path: String, backup_path: String },
    Deleted { path: String, backup_path: String },
}

impl CheckpointManager {
    /// 创建新的检查点
    pub async fn create_checkpoint(
        &mut self,
        conversation_id: &str,
        iteration: u32,
        new_messages: Vec<ChatMessage>,
        file_changes: Vec<FileChange>,
        tool_results: Vec<ToolResultRecord>,
    ) -> AppResult<String> {
        let checkpoint_id = uuid::Uuid::new_v4().to_string();
        
        let checkpoint = Checkpoint {
            id: checkpoint_id.clone(),
            conversation_id: conversation_id.to_string(),
            iteration,
            timestamp: chrono::Utc::now().timestamp(),
            new_messages,
            file_changes,
            tool_results,
        };
        
        // 序列化并保存到 SQLite
        let json = serde_json::to_string(&checkpoint)?;
        self.db_connection.execute(
            "INSERT INTO checkpoints (id, conversation_id, iteration, data, created_at) 
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![
                checkpoint.id,
                checkpoint.conversation_id,
                checkpoint.iteration,
                json,
                checkpoint.timestamp
            ],
        )?;
        
        self.current_checkpoint_id = Some(checkpoint_id.clone());
        Ok(checkpoint_id)
    }
    
    /// 回滚到指定检查点
    pub async fn rollback_to_checkpoint(&self, checkpoint_id: &str) -> AppResult<Checkpoint> {
        let row = self.db_connection.query_row(
            "SELECT data FROM checkpoints WHERE id = ?1",
            rusqlite::params![checkpoint_id],
            |row| row.get::<_, String>(0),
        )?;
        
        let checkpoint: Checkpoint = serde_json::from_str(&row)?;
        
        // 回滚文件变更
        for change in &checkpoint.file_changes {
            match change {
                FileChange::Created { path } => {
                    // 删除创建的文件
                    if std::path::Path::new(path).exists() {
                        std::fs::remove_file(path)?;
                    }
                }
                FileChange::Modified { path, backup_path } => {
                    // 从备份恢复
                    std::fs::copy(backup_path, path)?;
                }
                FileChange::Deleted { path, backup_path } => {
                    // 从备份恢复删除的文件
                    std::fs::copy(backup_path, path)?;
                }
            }
        }
        
        Ok(checkpoint)
    }
    
    /// 清理过期检查点（保留最近 24 小时）
    pub fn cleanup_old_checkpoints(&self) -> AppResult<()> {
        let cutoff = chrono::Utc::now().timestamp() - 86400; // 24 小时前
        self.db_connection.execute(
            "DELETE FROM checkpoints WHERE created_at < ?1",
            rusqlite::params![cutoff],
        )?;
        Ok(())
    }
}
```

### 5.3 集成到 Agent Loop

```rust
// native/src/ai/stream.rs - 优化后的 Agent Loop

pub async fn stream_chat_completion_with_checkpoint(
    config: &ModelConfig,
    messages: Vec<ChatMessage>,
    conversation_id: &str,
    message_id: &str,
    callbacks: &StreamCallbacks,
    skills_schemas: Vec<serde_json::Value>,
) -> AppResult<()> {
    let mut checkpoint_mgr = CheckpointManager::new()?;
    let mut current_messages = messages;
    let mut iteration = 0;
    let max_iterations = 30;
    
    loop {
        iteration += 1;
        
        // 1. 执行单轮流式对话
        let tool_calls_result = stream_single_turn(...).await?;
        
        if tool_calls_result.is_empty() {
            break;
        }
        
        // 2. 智能调度
        let scheduled = scheduler::schedule_tools(parse_tool_calls(tool_calls_result));
        
        // 3. 执行前备份（仅针对写入类工具）
        let mut file_backups = vec![];
        for tool in &scheduled.write_tools {
            if let Some(backup) = backup_file_if_needed(tool).await {
                file_backups.push(backup);
            }
        }
        
        // 4. 并行/串行执行工具
        let tool_results = execute_scheduled_tools(scheduled).await;
        
        // 5. 检查是否有失败
        let failures: Vec<_> = tool_results.iter()
            .filter(|(_, result)| !result.success)
            .collect();
        
        if !failures.is_empty() {
            warn!("⚠️  {} tools failed, rolling back...", failures.len());
            
            // 6. 回滚文件变更
            for backup in &file_backups {
                restore_from_backup(backup).await?;
            }
            
            // 7. 创建失败检查点（用于调试）
            checkpoint_mgr.create_checkpoint(
                conversation_id,
                iteration,
                vec![],
                file_backups.into_iter().map(|b| b.to_change()).collect(),
                tool_results.iter().map(|(tc, tr)| ToolResultRecord::new(tc, tr)).collect(),
            ).await?;
            
            // 8. 将失败信息添加到上下文
            for (tool_call, result) in failures {
                current_messages.push(ChatMessage {
                    role: "tool".to_string(),
                    content: format!("❌ 工具执行失败: {}\n请尝试其他方法。", result.output),
                    name: Some(tool_call.name.clone()),
                    tool_call_id: Some(tool_call.id.clone()),
                });
            }
        } else {
            // 9. 全部成功，创建成功检查点（仅中等以上任务）
            if should_create_checkpoint(iteration, &tool_results) {
                checkpoint_mgr.create_checkpoint(
                    conversation_id,
                    iteration,
                    extract_new_messages(&current_messages),
                    file_backups.into_iter().map(|b| b.to_change()).collect(),
                    tool_results.iter().map(|(tc, tr)| ToolResultRecord::new(tc, tr)).collect(),
                ).await?;
            }
            
            // 10. 添加工具结果到上下文
            for (tool_call, result) in &tool_results {
                current_messages.push(create_tool_message(tool_call, result));
            }
        }
        
        if iteration >= max_iterations {
            break;
        }
    }
    
    Ok(())
}

/// 判断是否应该创建检查点
fn should_create_checkpoint(iteration: u32, tool_results: &[(ToolCall, ToolResult)]) -> bool {
    // 策略1: 迭代次数 >= 3（中等以上任务）
    if iteration >= 3 {
        return true;
    }
    
    // 策略2: 有写入类工具执行
    if tool_results.iter().any(|(tc, _)| is_write_tool(&tc.name)) {
        return true;
    }
    
    // 策略3: 总执行时间 > 30 秒
    // （需要在上下文中记录开始时间）
    
    false
}
```

### 5.4 数据库表结构

```sql
-- native/src/db/migrations.sql

CREATE TABLE IF NOT EXISTS checkpoints (
    id TEXT PRIMARY KEY,
    conversation_id TEXT NOT NULL,
    iteration INTEGER NOT NULL,
    data TEXT NOT NULL,  -- JSON 序列化的 Checkpoint 数据
    created_at INTEGER NOT NULL,  -- Unix 时间戳
    
    FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE CASCADE
);

-- 索引优化查询
CREATE INDEX IF NOT EXISTS idx_checkpoints_conversation ON checkpoints(conversation_id);
CREATE INDEX IF NOT EXISTS idx_checkpoints_created_at ON checkpoints(created_at);
```

---

## 6. 实施路线图

### Phase 1: 基础框架（1-2 周）

**目标**：实现基本的检查点创建和回滚功能

**任务清单：**
- [ ] 设计 `Checkpoint` 数据结构
- [ ] 实现 `CheckpointManager` 核心逻辑
- [ ] 添加 SQLite 表结构和迁移脚本
- [ ] 单元测试：创建、读取、回滚检查点

**验收标准：**
- ✅ 能够创建和保存检查点到 SQLite
- ✅ 能够从检查点恢复消息历史
- ✅ 单元测试覆盖率 > 80%

---

### Phase 2: 文件备份与回滚（1-2 周）

**目标**：支持文件操作的备份和回滚

**任务清单：**
- [ ] 实现 `backup_file_if_needed()` 函数
- [ ] 实现 `restore_from_backup()` 函数
- [ ] 集成到 Agent Loop 的执行流程
- [ ] 测试文件创建、修改、删除的回滚

**验收标准：**
- ✅ 写入工具执行前自动备份文件
- ✅ 失败时能够正确回滚文件变更
- ✅ 备份文件在成功后自动清理

---

### Phase 3: 智能触发策略（1 周）

**目标**：优化检查点创建时机，平衡性能和可靠性

**任务清单：**
- [ ] 实现 `should_create_checkpoint()` 策略函数
- [ ] 添加配置项（最小迭代次数、最小执行时间等）
- [ ] 性能基准测试（检查点开销 < 5%）
- [ ] 清理过期检查点的定时任务

**验收标准：**
- ✅ 短时任务不创建检查点（性能优先）
- ✅ 中等以上任务自动创建检查点（可靠性优先）
- ✅ 检查点创建开销 < 50ms

---

### Phase 4: 中断恢复（可选，2-3 周）

**目标**：支持进程重启后从检查点恢复执行

**任务清单：**
- [ ] 实现 `resume_from_last_checkpoint()` 函数
- [ ] 在应用启动时检查未完成的会话
- [ ] UI 提示用户是否恢复之前的任务
- [ ] 端到端测试：中断 → 重启 → 恢复

**验收标准：**
- ✅ 进程崩溃后能够从最后一个检查点恢复
- ✅ 用户可以选择是否恢复任务
- ✅ 恢复后继续执行，无需从头开始

---

### Phase 5: 监控与调试（1 周）

**目标**：提供可视化的检查点管理和调试工具

**任务清单：**
- [ ] 添加检查点查询 API
- [ ] 实现检查点 diff 工具（对比两个检查点的差异）
- [ ] UI 展示检查点历史
- [ ] 日志增强（记录检查点创建和回滚事件）

**验收标准：**
- ✅ 开发者可以查询和调试检查点
- ✅ 用户可以查看任务的执行历史
- ✅ 日志清晰记录所有检查点操作

---

## 7. 风险评估

### 7.1 技术风险

| 风险 | 概率 | 影响 | 缓解措施 |
|------|------|------|---------|
| SQLite 写入性能瓶颈 | 🟡 中 | 🟡 中 | 异步写入、批量提交 |
| 备份文件占用磁盘空间 | 🟢 低 | 🟡 中 | 自动清理、限制备份数量 |
| 回滚逻辑复杂导致 Bug | 🟡 中 | 🔴 高 | 充分测试、灰度发布 |
| 检查点数据损坏 | 🟢 低 | 🔴 高 | CRC 校验、多副本 |

### 7.2 业务风险

| 风险 | 概率 | 影响 | 缓解措施 |
|------|------|------|---------|
| 用户不理解检查点机制 | 🟢 低 | 🟢 低 | 清晰的 UI 提示 |
| 恢复后状态不符合预期 | 🟡 中 | 🟡 中 | 详细的恢复日志 |
| 性能下降影响用户体验 | 🟡 中 | 🔴 高 | 智能触发策略、性能监控 |

---

## 8. 总结与建议

### 8.1 推荐方案

**轻量级增量检查点** 方案最适合 CoSurf，原因：

1. **符合场景分布**：70% 短时任务不受影响，25% 中等任务获得可靠性提升
2. **实现成本低**：利用现有 SQLite 基础设施，无需引入新依赖
3. **性能开销小**：只在必要时创建检查点，平均开销 < 5%
4. **易于维护**：代码量适中（~500 行），逻辑清晰

### 8.2 关键决策

| 决策点 | 选择 | 理由 |
|--------|------|------|
| **持久化后端** | SQLite | 已有基础设施，零额外依赖 |
| **快照粒度** | 增量 | 性能优于完整快照 |
| **回滚范围** | 仅文件 | 浏览器状态回滚成本过高 |
| **触发策略** | 智能 | 平衡性能和可靠性 |
| **中断恢复** | 可选 | 优先级较低，可后期实现 |

### 8.3 下一步行动

1. **立即开始**：Phase 1（基础框架）
2. **2 周内完成**：Phase 1 + Phase 2
3. **1 个月内上线**：Phase 1-3（核心功能）
4. **3 个月内完善**：Phase 4-5（高级功能）

---

## 附录：参考资源

- OpenClaw 源码：https://github.com/openclaw/openclaw
- LangGraph Checkpoint：https://langchain-ai.github.io/langgraph/concepts/persistence/
- Temporal Workflows：https://docs.temporal.io/workflows
- Claude Code 技术博客：https://www.anthropic.com/news/claude-code
