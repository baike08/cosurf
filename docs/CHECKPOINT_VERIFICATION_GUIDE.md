# Checkpoint 机制验证指南

## 📋 概述

Checkpoint 机制用于在 Agent Loop 执行过程中保存中间状态，支持失败时回滚到稳定状态。

---

## 🔍 验证方法

### 方法 1：实时日志监控（推荐）

**步骤：**

1. **启动 CoSurf 应用**
   ```powershell
   cd d:\coding-harness\CoSurf
   pnpm dev
   ```

2. **发送需要多次迭代的请求**
   
   示例请求：
   - "帮我创建一个 Python 计算器，包含加减乘除功能"
   - "分析当前网页并生成总结报告"
   - "搜索今天的天气信息并保存到文件"

3. **观察终端日志输出**

   **正常情况应该看到：**
   ```
   🔄 Agent Loop iteration 1/30
   🔄 Agent Loop iteration 2/30
   🔄 Agent Loop iteration 3/30
   📸 Created checkpoint: <uuid> (iteration=3)
   🔄 Agent Loop iteration 4/30
   📦 Backed up file before modification: C:\path\to\file.md
   📸 Updated checkpoint with file changes: <uuid> (iteration=5)
   ```

   **会话结束时：**
   ```
   🧹 Cleaned up 2 old checkpoints (retention: 24h)
   🧹 Cleaned up 1 old backup files (retention: 24h)
   ```

   **如果发生连续失败（≥ 3 次）：**
   ```
   🔄 Detected 3 consecutive failures, attempting rollback...
   🔄 Rolling back to checkpoint: <uuid> (iteration=4)
   ✅ Successfully rolled back file changes
   ```

---

### 方法 2：运行验证脚本

**步骤：**

```powershell
cd d:\coding-harness\CoSurf
powershell -ExecutionPolicy Bypass -File .\scripts\verify_checkpoint.ps1
```

**脚本会检查：**
- ✅ 检查点数据库文件是否存在
- ✅ 备份文件目录及内容
- ✅ 日志中的 Checkpoint 活动统计
- ✅ 最近的 Checkpoint 相关日志

**输出示例：**
```
Checkpoint Mechanism Verification
================================

1. Checking checkpoint databases...
   [OK] Found 2 checkpoint databases
   - checkpoint_abc123.db (15.2 KB)
   - checkpoint_def456.db (8.7 KB)

2. Checking backup directory...
   [OK] Found 3 backup files
     - calculator.py_1718534400000.bak (2.1 KB)
     - report.md_1718534401000.bak (5.3 KB)
     - data.csv_1718534402000.bak (1.8 KB)

3. Checking recent checkpoint logs...
   Statistics:
      - Checkpoints created: 5
      - Files backed up: 3
      - Checkpoints cleaned: 1
      - Rollbacks triggered: 0

   Recent checkpoint logs:
      2026-06-16T16:41:22Z INFO cosurf_native::ai::stream: Created checkpoint: abc123 (iteration=3)
      2026-06-16T16:41:25Z INFO cosurf_native::ai::stream: Backed up file before modification: ...
      ...

Verification Complete

Quick Diagnosis:
  [PASS] Database files exist
  [PASS] Backup directory exists
  [PASS] Log file accessible
```

---

### 方法 3：手动检查数据库文件

**步骤：**

1. **找到检查点数据库文件**
   ```powershell
   Get-ChildItem -Path $env:APPDATA\cosurf\cosurf-data -Filter "checkpoint_*.db"
   ```

2. **使用 SQLite 浏览器查看内容**
   
   下载 [DB Browser for SQLite](https://sqlitebrowser.org/) 或使用命令行：
   
   ```powershell
   # 安装 sqlite3 命令行工具
   winget install SQLite.SQLite
   
   # 查询检查点表
   sqlite3 "$env:APPDATA\cosurf\cosurf-data\checkpoint_<conversation_id>.db" \
     "SELECT id, conversation_id, iteration, timestamp FROM checkpoints ORDER BY timestamp DESC LIMIT 10;"
   
   # 查看文件变更
   sqlite3 "$env:APPDATA\cosurf\cosurf-data\checkpoint_<conversation_id>.db" \
     "SELECT checkpoint_id, file_path, change_type FROM file_changes;"
   ```

---

### 方法 4：触发回滚测试（高级）

**目的：** 验证 Checkpoint 的回滚功能是否正常工作

**步骤：**

1. **准备一个会导致连续失败的场景**
   
   例如：尝试修改一个被锁定的文件或访问不存在的 API

2. **观察日志**
   ```
   ❌ Tool execution failed: Permission denied
   ❌ Tool execution failed: Permission denied
   ❌ Tool execution failed: Permission denied
   🔄 Detected 3 consecutive failures, attempting rollback...
   🔄 Rolling back to checkpoint: <uuid> (iteration=4)
   ✅ Successfully rolled back file changes
   ```

3. **验证文件已恢复**
   ```powershell
   # 检查文件是否恢复到之前的状态
   Get-Content "C:\path\to\modified\file.md"
   ```

---

## 📊 Checkpoint 数据存储位置

### 1. 检查点数据库
- **路径：** `%APPDATA%\cosurf\cosurf-data\checkpoint_<conversation_id>.db`
- **格式：** SQLite 数据库
- **内容：** 
  - `checkpoints` 表：检查点元数据
  - `messages` 表：新增消息（JSON 序列化）
  - `file_changes` 表：文件变更记录
  - `tool_results` 表：工具执行结果

### 2. 文件备份
- **路径：** `%TEMP%\cosurf-checkpoint-backups\`
- **命名规则：** `<filename>_<timestamp>.bak`
- **清理策略：** 保留最近 24 小时

### 3. 日志输出
- **位置：** 终端标准输出（开发模式）
- **关键字：** `Created checkpoint`, `Backed up file`, `Rolling back`, `Cleaned up`

---

## 🎯 关键指标

| 指标 | 预期值 | 说明 |
|------|--------|------|
| 检查点创建时机 | iteration ≥ 3 | 前 3 次迭代跳过以减少开销 |
| 文件备份触发 | 文件修改前 | 自动检测并备份 |
| 回滚触发条件 | 连续失败 ≥ 3 次 | 自动回滚到上一个稳定检查点 |
| 清理周期 | 会话结束时 | 清理 > 24h 的旧数据 |
| 数据库大小 | < 100 KB / 会话 | 增量存储，只保存变更 |
| 备份文件大小 | 原始文件大小 | 完整文件副本 |

---

## ⚠️ 常见问题

### Q1: 为什么没有看到 "Created checkpoint" 日志？

**可能原因：**
1. Agent Loop 迭代次数 < 3（检查点从第 3 次迭代后开始创建）
2. 对话未触发工具调用
3. CheckpointManager 初始化失败（查看是否有 "Failed to initialize CheckpointManager" 警告）

**解决方法：**
- 发送需要多次工具调用的复杂请求
- 检查日志中是否有错误信息

---

### Q2: 为什么没有看到 "Backed up file" 日志？

**可能原因：**
1. 工具未修改文件系统（如只读操作）
2. 文件路径未被正确提取
3. 文件不存在或无法访问

**解决方法：**
- 使用会修改文件的工具（如 `write_file`, `edit_file`）
- 检查工具返回的文件路径是否正确

---

### Q3: 如何确认回滚功能正常工作？

**验证步骤：**
1. 创建一个测试文件
2. 让 AI 修改该文件
3. 模拟连续失败（如断开网络）
4. 观察是否触发回滚
5. 检查文件是否恢复到修改前的状态

---

### Q4: 检查点数据库会占用多少空间？

**估算：**
- 每个检查点：~5-20 KB（取决于消息数量和文件大小）
- 每次会话：通常 3-10 个检查点
- 总计：< 200 KB / 会话
- 自动清理：> 24h 的检查点会被删除

---

## 📝 示例场景

### 场景 A：正常流程

```
用户：帮我创建一个 Python 计算器

日志输出：
🔄 Agent Loop iteration 1/30
🔄 Agent Loop iteration 2/30
🔄 Agent Loop iteration 3/30
📸 Created checkpoint: cp_001 (iteration=3)
🔧 Executing tool: write_file (calculator.py)
📦 Backed up file before modification: calculator.py
📸 Updated checkpoint with file changes: cp_001
🔄 Agent Loop iteration 4/30
✅ Task completed
🧹 Cleaned up 0 old checkpoints
```

### 场景 B：回滚流程

```
用户：修改系统配置文件

日志输出：
🔄 Agent Loop iteration 1/30
...
📸 Created checkpoint: cp_002 (iteration=3)
❌ Tool execution failed: Permission denied
❌ Tool execution failed: Permission denied
❌ Tool execution failed: Permission denied
🔄 Detected 3 consecutive failures, attempting rollback...
🔄 Rolling back to checkpoint: cp_002 (iteration=3)
✅ Successfully rolled back file changes
[系统] 检测到连续失败，已回滚到上一个稳定状态。请重试。
```

---

## 🔗 相关文档

- [Checkpoint 实施总结](./CHECKPOINT_IMPLEMENTATION_SUMMARY.md)
- [Agent Loop 架构](../docs/AGENT_TOOLS_STATUS.md)
- [验证脚本](../scripts/verify_checkpoint.ps1)

---

## 💡 提示

- **最佳实践：** 定期运行验证脚本检查 Checkpoint 机制状态
- **调试技巧：** 在日志中搜索 "checkpoint" 关键字快速定位相关问题
- **性能优化：** 如果检查点创建影响性能，可以调整触发阈值（默认 iteration ≥ 3）
