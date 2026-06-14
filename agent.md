# CoSurf Agent Guide

> 本文件为 AI Coding Agent 提供项目开发指南。在修改代码前请先阅读此文档。

## 项目概述

CoSurf（伴游）是你的 **AI 阅读伴侣和思考搭档**，帮你读得更深、记得更牢、想得更快。

- **定位**：AI 阅读伴侣（读过的，都算数。）
- **仓库**：https://github.com/cosurf/cosurf.git
- **代码量**：~18,000 行（Rust 8.4K + React/TS 7.9K + 配置 1.7K）

## 技术栈

| 层级 | 技术 | 版本 |
|------|------|------|
| 桌面框架 | Tauri | 2.5 |
| 后端语言 | Rust | 1.88+ (edition 2021) |
| 前端框架 | React | 18.3 |
| 构建工具 | Vite | 6.x |
| UI 样式 | Tailwind CSS | 3.4 |
| 状态管理 | Zustand | 5.x |
| 数据库 | SQLite (rusqlite) | 0.32 (bundled) |
| HTTP 客户端 | reqwest + reqwest-eventsource | 0.12 / 0.6 |
| 截图 | xcap + image | 0.4 / 0.25 |
| Markdown 渲染 | react-markdown + remark-gfm | 10.x / 4.x |
| 图标 | lucide-react | 0.468 |
| 包管理 | pnpm | 9.15 (monorepo workspace) |
| Node.js | ≥ 20.0 | |

## 目录结构

```
CoSurf/
├── src-tauri/            # Rust 后端（Tauri 2.x）
│   ├── src/
│   │   ├── ai/           # AI 核心模块
│   │   │   ├── agent.rs          # Agent Loop 实现
│   │   │   ├── provider.rs       # AI Provider 抽象（多模型支持）
│   │   │   ├── stream.rs         # 流式响应处理
│   │   │   ├── mcp.rs            # MCP 客户端（JSON-RPC 2.0）
│   │   │   ├── skills.rs         # Skills 管理器（Markdown 加载/解析）
│   │   │   ├── skills_engine.rs  # Skills 引擎
│   │   │   ├── skills_executors/ # 执行器子模块
│   │   │   │   ├── mod.rs
│   │   │   │   ├── mcp.rs        # MCP 协议执行器
│   │   │   │   └── command_utils.rs  # 共享命令工具（PATH增强、cmd解析）
│   │   │   ├── tools.rs          # 工具定义（JSON Schema）
│   │   │   ├── tools_impl/       # 工具实现
│   │   │   │   └── dispatcher.rs # 工具调用分发器
│   │   │   ├── playwright_client.rs  # Playwright 无头浏览器客户端
│   │   │   └── sandbox.rs        # 沙箱环境
│   │   ├── commands/     # Tauri IPC 命令层
│   │   │   ├── mod.rs            # 命令注册
│   │   │   ├── ai.rs             # AI 对话命令（send_chat_message等）
│   │   │   ├── ai_agent.rs       # Agent 执行命令
│   │   │   ├── settings.rs       # 设置/MCP Server 管理
│   │   │   ├── browser.rs        # 浏览历史
│   │   │   ├── browser_nav.rs    # WebView 导航/元素操作
│   │   │   ├── page_context.rs   # 页面上下文提取
│   │   │   ├── screenshot.rs     # 截图功能
│   │   │   ├── skills.rs         # Skills 管理命令
│   │   │   └── ...
│   │   ├── db/           # 数据库层
│   │   │   ├── mod.rs            # 建表 + 迁移
│   │   │   ├── conversations.rs
│   │   │   ├── messages.rs
│   │   │   ├── bookmarks.rs
│   │   │   ├── history.rs
│   │   │   └── settings.rs       # 设置 + 模型配置 + MCP Server CRUD
│   │   ├── lib.rs        # 入口：插件注册、命令注册、全局快捷键
│   │   ├── state.rs      # AppState 定义
│   │   └── error.rs      # 错误类型定义
│   ├── capabilities/default.json  # Tauri 权限配置
│   ├── tauri.conf.json   # Tauri 应用配置
│   └── Cargo.toml
├── src-web/              # React 前端
│   ├── src/
│   │   ├── components/
│   │   │   ├── layout/
│   │   │   │   ├── AppLayout.tsx       # 主布局
│   │   │   │   ├── AIPanel.tsx         # AI 对话面板（可拖拽）
│   │   │   │   ├── Sidebar.tsx         # 侧边栏（可拖拽）
│   │   │   │   ├── TabBar.tsx          # 标签栏
│   │   │   │   ├── NavigationBar.tsx   # 导航栏
│   │   │   │   ├── WebContentView.tsx  # Web 内容视图（iframe 多标签）
│   │   │   │   └── BrowserActionPanel.tsx  # 浏览器操作面板
│   │   │   ├── settings/
│   │   │   │   ├── SettingsPage.tsx    # 设置页面（Monaco Editor）
│   │   │   │   └── McpServersSettings.tsx  # MCP Server 管理
│   │   │   ├── sidebar/
│   │   │   └── ui/                     # 通用 UI 组件
│   │   ├── stores/       # Zustand 状态管理
│   │   │   ├── conversationStore.ts    # 对话/消息状态
│   │   │   ├── settingsStore.ts        # 设置/模型配置
│   │   │   ├── tabStore.ts             # 标签页状态
│   │   │   ├── screenshotStore.ts      # 截图状态
│   │   │   ├── downloadStore.ts        # 下载状态
│   │   │   └── uiStore.ts              # UI 状态
│   │   ├── hooks/
│   │   │   └── useTheme.ts
│   │   ├── lib/          # 工具函数
│   │   ├── App.tsx       # 根组件
│   │   └── main.tsx
│   ├── package.json
│   └── vite.config.ts
├── packages/shared/      # 共享类型定义（前后端共用）
│   ├── src/
│   │   ├── conversation.ts
│   │   ├── message.ts
│   │   ├── model.ts
│   │   ├── settings.ts
│   │   ├── bookmark.ts
│   │   ├── tab.ts
│   │   ├── tool.ts
│   │   └── index.ts
│   └── package.json
├── playwright-service/   # Playwright 无头浏览器服务（独立进程）
├── examples/             # Skills 示例文件（Markdown 格式）
├── scripts/              # 构建/开发脚本（PowerShell）
└── .tools/               # 本地 Node.js 等工具链
```

## 开发命令

```bash
# 开发模式（推荐：同时启动前端 + Rust 后端）
pnpm dev:tauri
# 或
pnpm dev:full   # PowerShell -ExecutionPolicy Bypass -File scripts/dev.ps1

# 仅前端开发服务器
pnpm dev

# 构建
pnpm build              # 前端构建（shared + web）
pnpm build:tauri        # Tauri 完整构建
pnpm build:release      # 发布构建（含检查）

# 代码检查
pnpm check              # TypeScript + ESLint + Clippy 全部检查
```

## 核心架构模式

### 1. 前后端通信

使用 **Tauri IPC**（`invoke`）：

```typescript
// 前端调用
import { invoke } from '@tauri-apps/api/core';
const result = await invoke<ReturnType>('command_name', { param1: value1 });
```

```rust
// 后端命令
#[tauri::command]
pub async fn command_name(param1: String) -> Result<ReturnType, ErrorResponse> { ... }
```

### 2. 错误处理

后端统一使用 `AppError` 枚举 + `AppResult<T>`：

```rust
// error.rs 定义了统一的错误类型
pub enum AppError {
    Database(rusqlite::Error),
    Http(reqwest::Error),
    Json(serde_json::Error),
    Tauri(tauri::Error),
    AiProvider(String),
    Config(String),
    NotFound(String),
    Internal(String),
}

// 前端返回使用 ErrorResponse
pub struct ErrorResponse {
    pub code: String,     // 如 "DATABASE_ERROR"
    pub message: String,  // 人类可读的错误信息
}
```

### 3. 状态管理

**后端 AppState**（`state.rs`）：
- `db: Mutex<Database>` — SQLite 数据库连接
- `cancel_flag: Arc<AtomicBool>` — 流式生成取消标志
- `active_tab_id: Arc<Mutex<Option<String>>>` — 当前活跃标签页
- `skills_manager: Arc<Mutex<SkillsManager>>` — Skills 管理
- `mcp_tool_registry: Arc<Mutex<HashMap>>` — MCP 工具路由表

**前端 Zustand Stores**：每个 store 独立管理一块状态，通过 `invoke` 与后端同步。

### 4. AI Agent Loop

Agent 采用 **React Agent Loop** 模式：
1. 接收用户消息 → 构建 system prompt + 工具列表
2. 调用 LLM 流式响应 → 解析 tool_calls
3. 并行执行工具调用（`join_all`）→ 将结果追加到上下文
4. 继续下一轮 LLM 调用，直到无 tool_calls 或达到最大迭代（30次）

**工具命名约定**：
- 内置工具：`summarize_page`, `web_agent`, `open_url` 等
- Skills 工具：`skill_{skill_id}`
- MCP 工具：`mcp_{server_name}_{tool_name}`

### 5. 多标签页实现

Tauri 2.x **不支持动态创建多个 WebView 实例**，因此采用 **iframe 方案**：
- 每个标签页对应一个 iframe
- `WebContentView.tsx` 管理所有 iframe 的生命周期
- 通过 Tauri 的 WebView2 导航事件驱动标签页状态更新

### 6. 流式响应

AI 对话使用 SSE 流式传输：
- 后端 `stream.rs` 处理 SSE 事件流
- 通过 Tauri `emit` 事件推送到前端
- 前端 `conversationStore` 实时更新消息内容
- thinking 内容和 content 内容分别存储在独立字段

## 数据库 Schema

SQLite，WAL 模式，外键约束启用。7 张核心表：

| 表名 | 用途 |
|------|------|
| conversations | 对话列表（标题、置顶、模型ID） |
| messages | 消息（role、content、thinking_content、feedback） |
| bookmarks | 书签（URL、标题、文件夹） |
| bookmark_folders | 书签文件夹（树形结构） |
| history | 浏览历史 |
| settings | 键值对设置（包括 Skills 目录等） |
| model_configs | AI 模型配置（多模型支持） |
| mcp_servers | MCP Server 配置（stdio/http/sse） |

**数据库迁移**：在 `db/mod.rs` 的 `run_migrations()` 中执行。新增列用 `ensure_*` 方法检查后 `ALTER TABLE`。

**注意**：SQLite SQL 只支持 `--` 注释，**不支持 `//` 注释**（会导致编译 panic）。

## 关键开发规范

### Rust 后端

1. **序列化**：使用 `serde(rename_all = "camelCase")` 时注意前后端字段名一致
2. **MCP 协议**：camelCase 序列化是 MCP 标准，确保 JSON-RPC 字段名正确
3. **异步**：所有 IO 操作使用 tokio 异步，数据库操作在 `spawn_blocking` 中执行
4. **日志**：使用 `tracing` 库，级别：info（关键事件）、warn（非致命错误）、error（错误）
5. **命令注册**：新命令必须在 `lib.rs` 的 `invoke_handler` 中注册

### 前端

1. **状态管理**：使用 Zustand，每个 store 文件管理一个领域
2. **组件结构**：`components/layout/` 放布局组件，`components/settings/` 放设置相关，`components/ui/` 放通用组件
3. **样式**：Tailwind CSS utility-first，不要写自定义 CSS
4. **Tauri 事件**：前端通过 `listen`/`emit` 接收/发送事件（需导入 `Event` trait）
5. **Monaco Editor**：MCP Server JSON 配置编辑使用 `@monaco-editor/react`

### Windows 特殊处理

1. **命令执行**：`echo`、`npx`、`npm` 等是 shell 内建命令或 `.cmd` 文件，必须通过 `cmd /c` 执行。使用 `command_utils::resolve_command()` 统一处理
2. **Python**：Windows 使用 `python` 而非 `python3`
3. **PATH**：系统 PATH 可能不包含 Node.js/Python 安装位置，使用 `command_utils::build_enhanced_path()` 增强
4. **PowerShell**：在终端执行命令时使用 PowerShell 语法，用 `;` 分隔命令，不用 `&&`
5. **pnpm 脚本**：可能因执行策略被阻止，需要 `-ExecutionPolicy Bypass`

## Skills 系统

Skills 以 **Markdown 文件** 形式管理，包含 YAML frontmatter + 配置块：

```markdown
---
name: my-skill
description: 描述
type: cli | script | mcp
enabled: true
---

## 配置
```json
{
  "command": "...",
  "args": [...]
}
```
```

- Skills 目录可通过设置页面自定义
- 启动时自动从 `examples/` 同步示例 Skills
- 支持 CLI、Script、MCP 三种执行器类型

## MCP Server 支持

支持三种传输模式：
- **stdio**：通过子进程 stdin/stdout 通信（如 `npx` 启动的 MCP Server）
- **streamableHttp**：HTTP POST + SSE（推荐）
- **sse**：Server-Sent Events

配置支持 JSON 批量导入，格式兼容开源 MCP 标准：
```json
{
  "mcpServers": {
    "server-name": {
      "type": "streamableHttp",
      "url": "https://...",
      "headers": { "X-API-Key": "..." }
    }
  }
}
```

## 新增功能检查清单

添加新 Tauri 命令时：
1. 在 `commands/` 对应模块中实现 `#[tauri::command]` 函数
2. 在 `lib.rs` 的 `invoke_handler` 中注册
3. 如果需要数据库操作，在 `db/` 对应模块添加 CRUD 方法
4. 前端通过 `invoke` 调用，在 store 中封装

添加新 AI 工具时：
1. 在 `ai/tools.rs` 定义工具的 JSON Schema
2. 在 `ai/tools_impl/dispatcher.rs` 实现工具调用逻辑
3. 在 `ai/stream.rs` 的工具列表中注册
4. 确保工具失败不会终止 Agent Loop

添加新设置项时：
1. 在 `db/settings.rs` 添加 getter/setter 方法
2. 在 `commands/settings.rs` 添加 Tauri 命令
3. 在 `lib.rs` 注册命令
4. 前端在 `stores/settingsStore.ts` 添加状态和 action
5. 在设置页面添加 UI

## 安全规范

- **不得在代码中硬编码 API Key**，所有密钥从数据库设置或环境变量获取
- `.gitignore` 已排除 `.env`、`*.db`、`*.key`、`*.pem` 等敏感文件
- 示例文件中的密钥使用占位符（`your-api-key-here`）
- `target/`、`node_modules/`、`dist/` 不提交

## Tauri 权限

新增功能如需 Tauri 权限，在 `src-tauri/capabilities/default.json` 中添加对应 permission。

常见权限：
- `shell:allow-open` — 打开外部 URL
- `dialog:allow-save` — 文件保存对话框
- `fs:default` — 文件系统访问
- `http:default` — HTTP 请求

## 快捷键

### 前端快捷键（AppLayout.tsx 全局监听）

| 快捷键 | 功能 | 实现 |
|---------|------|------|
| `Ctrl+J` | 切换 AI 面板 | `uiStore.toggleAIPanel()` |
| `Ctrl+T` | 新建标签页 | `tabStore.addTab()` |
| `Ctrl+W` | 关闭当前标签页 | `tabStore.closeTab(activeTabId)` |
| `Ctrl+L` | 聚焦地址栏 | 触发 `focus-address-bar` 自定义事件，NavigationBar 监听并聚焦输入框 |
| `Ctrl+Shift+N` | 新建对话 | 自动打开 AI 面板 + `conversationStore.createConversation()` |
| `Ctrl+Shift+X` | 全局截图 | Rust 后端全局快捷键（lib.rs 注册） |

### 后端快捷键（lib.rs 注册）

- `Control+Shift+X`：全局截图（Tauri global-shortcut 插件）

## 常见问题

| 问题 | 原因 | 解决 |
|------|------|------|
| 端口 1420 被占用 | 旧进程未退出 | `netstat -ano \| findstr :1420` → `taskkill /PID xxx /F` |
| HotKey already registered | 旧进程锁定热键 | 先 kill 所有 cosurf.exe 再启动 |
| exe 被锁定编译失败 | 旧进程占用 exe | 同上，kill 旧进程 |
| SQL `near "/": syntax error` | 在 SQL 中使用了 `//` 注释 | 改为 `--` 注释 |
| `program not found` | 命令不在 PATH 中 | 使用 `command_utils::build_enhanced_path()` |
| Windows echo/npx 找不到 | shell 内建/.cmd 文件 | 使用 `command_utils::resolve_command()` → `cmd /c` |
| Serde 字段不匹配 | camelCase 转换不一致 | 检查 `#[serde(rename_all)]` 和前端字段名 |
| `withGlobalTauri` 为 false | 前端不能用 `window.__TAURI__` | 必须通过 `@tauri-apps/api` 导入 |
