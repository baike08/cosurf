# CoSurf（伴游）- AI Native Desktop Browser

<div align="center">

**一款 AI 原生的桌面浏览器，让浏览更智能**

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Tauri](https://img.shields.io/badge/Tauri-2.5-FFC131?logo=tauri)](https://tauri.app/)
[![React](https://img.shields.io/badge/React-18-61DAFB?logo=react)](https://reactjs.org/)
[![Rust](https://img.shields.io/badge/Rust-1.88-DEA584?logo=rust)](https://www.rust-lang.org/)

</div>

---

## ✨ 特性

### 🌐 浏览器核心
- **WebView2 内核** — 基于 Microsoft Edge WebView2，兼容所有现代网站
- **多标签页管理** — 支持标签页分组、置顶、拖拽排序
- **导航历史** — 前进/后退历史栈，快速导航
- **书签管理** — 文件夹分类，快速收藏
- **浏览历史** — 记录访问历史，快速回溯
- **下载管理** — 实时进度监控，断点续传
- **截图工具** — 全屏/选区截图，支持标注

### 🤖 AI 原生能力
- **多模型支持** — OpenAI、Anthropic Claude、Google Gemini、智谱、Kimi、DeepSeek、通义千问、Ollama 等
- **流式对话** — 实时流式响应（SSE），打字机效果，支持思考过程（thinking）展示
- **智能工具调用** — AI 可自动打开网页、总结页面、操作网页、截图理解、联网搜索
- **Agent Loop** — 多轮工具调用，自主完成复杂任务（如：打开网页 → 总结内容 → 回答问题）
- **MCP 协议集成** — 支持 stdio / SSE / Streamable HTTP 三种传输模式，无缝接入 MCP Server
- **MCP 工具直通** — MCP Server 的工具自动注册为 Agent 可用 function，AI 直接调用
- **Skills 系统** — 支持导入自定义 Skills（CLI、MCP、脚本），扩展 AI 能力
- **页面上下文感知** — AI 可感知当前浏览页面，提供精准回答
- **消息反馈** — 支持对 AI 回复点赞/点踩/复制，反馈持久化到数据库
- **对话历史** — SQLite 持久化存储，随时回顾

### 🎨 用户体验
- **现代化 UI** — 简洁优雅的设计，支持亮色/暗色主题
- **快捷键** — 丰富的快捷键，提升操作效率
- **窗口记忆** — 记住窗口位置和大小，下次启动恢复
- **自动更新** — 静默检查并安装更新

## 🏗️ 技术架构

```
┌─────────────────────────────────────────────────────────────────┐
│                          CoSurf Desktop                         │
├─────────────────────────────┬───────────────────────────────────┤
│     Frontend (React)        │        Backend (Tauri/Rust)       │
│     src-web/                │        src-tauri/                 │
│                             │                                   │
│  ┌──────────────────────┐   │  ┌─────────────────────────────┐  │
│  │  TabBar              │   │  │  Tauri Commands              │  │
│  │  NavigationBar       │   │  │  • ai / ai_agent            │  │
│  │  Sidebar             │   │  │  • conversation / message   │  │
│  │  WebContentView      │◄──┼──│  • bookmark / browser       │  │
│  │  (WebView2)          │   │  │  • page_context / screenshot│  │
│  │  AIPanel             │   │  │  • settings / skills        │  │
│  │  SettingsPage        │   │  └────────────┬────────────────┘  │
│  │  McpServersSettings  │   │               │                   │
│  └──────────────────────┘   │  ┌────────────▼────────────────┐  │
│                             │  │  SQLite (rusqlite)           │  │
│  zustand stores             │  │  conversations, messages,   │  │
│  • tabStore                 │  │  bookmarks, history,        │  │
│  • conversationStore        │  │  settings, mcp_servers      │  │
│  • settingsStore            │  └─────────────────────────────┘  │
│  • uiStore                  │                                   │
│  • downloadStore            │  ┌─────────────────────────────┐  │
│  • screenshotStore          │  │  AI Provider Layer           │  │
│                             │  │  • OpenAI-compatible API    │  │
│  @cosurf/shared ◄───────────┼──│  • Anthropic Messages API   │  │
│  (共享类型定义)              │  │  • SSE Streaming            │  │
│                             │  │  • Tool Calling (Function)  │  │
│                             │  └─────────────────────────────┘  │
│                             │                                   │
│                             │  ┌─────────────────────────────┐  │
│                             │  │  MCP Client                 │  │
│                             │  │  • stdio transport          │  │
│                             │  │  • SSE transport            │  │
│                             │  │  • Streamable HTTP          │  │
│                             │  │  • JSON-RPC 2.0             │  │
│                             │  └─────────────────────────────┘  │
│                             │                                   │
│                             │  ┌─────────────────────────────┐  │
│                             │  │  Playwright Service (可选)   │  │
│                             │  │  • Browser automation       │  │
│                             │  │  • Web Agent for AI         │  │
│                             │  └─────────────────────────────┘  │
└─────────────────────────────┴───────────────────────────────────┘
```

### 技术栈

| 层级 | 技术 |
|------|------|
| 桌面框架 | Tauri 2.x |
| 前端 | React 18 + TypeScript + Vite 6 |
| UI | Tailwind CSS 3 + Lucide Icons |
| 状态管理 | Zustand 5 |
| 后端 | Rust 1.88 |
| 数据库 | SQLite (rusqlite) |
| HTTP | reqwest + reqwest-eventsource |
| 浏览器内核 | WebView2 (Windows) |
| 自动化 | Playwright (可选侧车服务) |
| MCP 客户端 | 原生 Rust 实现 (JSON-RPC 2.0) |

## 🚀 快速开始

### 环境要求

- **Node.js** >= 20.0.0
- **pnpm** >= 9.0.0
- **Rust** >= 1.88.0 (Tauri 会自动安装)
- **Windows 10/11** (WebView2 已内置，无需额外安装)

### 克隆项目

```bash
git clone https://github.com/baike08/cosurf.git
cd cosurf
```

### 安装依赖

```bash
pnpm install
```

这会自动安装所有工作区的依赖：
- `@cosurf/web` - React 前端
- `@cosurf/shared` - 共享类型定义
- Playwright 服务（可选）

### 首次使用

1. **启动应用**：
   ```bash
   pnpm dev:full
   ```

2. **配置 AI 模型**：
   - 点击左下角设置图标 ⚙️
   - 选择「模型」标签页
   - 点击「添加模型」
   - 填写 API Key（参考下方「AI 模型配置」章节）

3. **开始对话**：
   - 点击右下角 AI 面板图标 💬
   - 在输入框中输入问题，例如：
     - "你好，请介绍一下自己"
     - "帮我打开百度并总结首页内容"
     - "翻译这段文字：Hello World"

4. **体验 Agent Loop**：
   尝试以下复杂任务，观察 AI 如何自主调用工具：
   - "打开知乎，总结热门话题"
   - "访问 GitHub，搜索 Rust 项目"
   - "打开新闻网站，用中文总结头条新闻"

### 开发模式

```bash
# 方式 1：仅启动前端 Vite 开发服务器
pnpm dev

# 方式 2：完整开发模式（推荐）- 同时启动前端和 Tauri 后端
pnpm dev:full
# 或
pnpm dev:tauri
```

开发模式下：
- 前端运行在 `http://localhost:1420`
- 支持热模块替换 (HMR)，修改代码后自动刷新
- Rust 后端会监控文件变化并自动重新编译

### 构建发布版本

```bash
# 完整构建（包含类型检查和 lint）
pnpm build:release

# 仅构建，不进行检查
pnpm build
pnpm build:tauri
```

构建产物位于 `src-tauri/target/release/bundle/`：
- `nsis/CoSurf_0.1.0_x64-setup.exe` — NSIS 安装包（推荐）
- `msi/CoSurf_0.1.0_x64_en-US.msi` — MSI 安装包

双击 `.exe` 文件即可安装。

### 全量检查

```bash
pnpm check
# 执行：TypeScript 类型检查 + ESLint + Cargo Clippy
```

建议在提交代码前运行此命令。

## 📁 项目结构

```
CoSurf/
├── src-web/                    # React 前端
│   ├── src/
│   │   ├── components/         # UI 组件
│   │   │   ├── layout/         # 布局组件
│   │   │   │   ├── AIPanel.tsx              # AI 对话面板（流式输出、消息反馈）
│   │   │   │   ├── AppLayout.tsx            # 应用主布局
│   │   │   │   ├── BrowserActionPanel.tsx   # 浏览器操作面板
│   │   │   │   ├── NavigationBar.tsx        # 导航栏
│   │   │   │   ├── Sidebar.tsx              # 侧边栏
│   │   │   │   ├── TabBar.tsx               # 标签页栏
│   │   │   │   ├── WebContentView.tsx       # WebView2 容器
│   │   │   │   └── WebView2Container.tsx    # WebView2 底层封装
│   │   │   ├── settings/       # 设置页面
│   │   │   │   ├── SettingsPage.tsx         # 设置主页面
│   │   │   │   ├── McpServersSettings.tsx   # MCP Server 配置
│   │   │   │   └── SkillsSettings.tsx       # Skills 管理
│   │   │   ├── sidebar/        # 侧边栏面板
│   │   │   │   └── DownloadsPanel.tsx       # 下载面板
│   │   │   └── ui/             # 基础 UI 组件
│   │   │       ├── IconButton.tsx           # 图标按钮
│   │   │       ├── Tooltip.tsx              # 工具提示
│   │   │       ├── ScreenshotOverlay.tsx    # 截图遮罩
│   │   │       └── ScreenshotSelector.tsx   # 截图选区
│   │   ├── stores/             # Zustand 状态管理
│   │   │   ├── conversationStore.ts         # 对话状态（流式消息、反馈）
│   │   │   ├── tabStore.ts                  # 标签页状态
│   │   │   ├── settingsStore.ts             # 设置状态
│   │   │   ├── uiStore.ts                   # UI 状态
│   │   │   ├── downloadStore.ts             # 下载状态
│   │   │   └── screenshotStore.ts           # 截图状态
│   │   ├── hooks/              # 自定义 Hooks
│   │   │   └── useTheme.ts                  # 主题切换
│   │   └── lib/                # 工具函数
│   │       ├── browserEngine.ts             # 浏览器引擎抽象
│   │       ├── eventManager.ts              # 事件管理器
│   │       ├── tauri.ts                     # Tauri API 封装
│   │       ├── tools.ts                     # 工具函数
│   │       ├── utils.ts                     # 通用工具
│   │       └── mock.ts                      # Mock 数据
│   └── ...
│
├── src-tauri/                  # Rust 后端
│   ├── src/
│   │   ├── commands/           # Tauri 命令处理器
│   │   │   ├── ai.rs                        # AI 对话命令（流式聊天）
│   │   │   ├── ai_agent.rs                  # Agent Loop 命令
│   │   │   ├── conversation.rs              # 会话管理
│   │   │   ├── message.rs                   # 消息管理（含反馈）
│   │   │   ├── browser.rs                   # 浏览器控制
│   │   │   ├── browser_nav.rs               # 浏览器导航
│   │   │   ├── page_context.rs              # 页面上下文提取
│   │   │   ├── page_cache.rs                # 页面缓存
│   │   │   ├── bookmark.rs                  # 书签管理
│   │   │   ├── screenshot.rs                # 截图
│   │   │   ├── settings.rs                  # 设置管理（含 MCP Server）
│   │   │   └── skills.rs                    # Skills 管理
│   │   ├── ai/                 # AI 模块
│   │   │   ├── agent.rs                     # Agent Loop 核心逻辑
│   │   │   ├── stream.rs                    # 流式对话实现
│   │   │   ├── provider.rs                  # AI 提供商抽象层
│   │   │   ├── tools.rs                     # 工具 schema 定义与 MCP 集成
│   │   │   ├── skills.rs                    # Skills 加载与解析
│   │   │   ├── sandbox.rs                   # 沙箱执行环境
│   │   │   ├── mcp.rs                       # MCP 客户端（JSON-RPC 2.0）
│   │   │   ├── playwright_client.rs         # Playwright 客户端
│   │   │   ├── tools_impl/    # 工具实现（模块化）
│   │   │   │   ├── dispatcher.rs            # 工具调度器（含 MCP 路由）
│   │   │   │   ├── open_url.rs              # 打开网页
│   │   │   │   ├── summarize_page.rs        # 页面总结
│   │   │   │   ├── web_search.rs            # 联网搜索（阿里云 IQS）
│   │   │   │   ├── web_agent.rs             # 网页操作
│   │   │   │   └── run_command.rs           # 命令执行
│   │   │   └── skills_executors/ # Skills 执行器
│   │   │       ├── mcp.rs                   # MCP Skill 执行器
│   │   │       └── command_utils.rs         # CLI/Script 工具
│   │   ├── db/                 # SQLite 数据库
│   │   │   ├── mod.rs                       # 数据库初始化与迁移
│   │   │   ├── conversations.rs             # 会话表操作
│   │   │   ├── messages.rs                  # 消息表操作（含反馈）
│   │   │   ├── bookmarks.rs                 # 书签表操作
│   │   │   ├── history.rs                   # 历史表操作
│   │   │   └── settings.rs                  # 设置表操作
│   │   ├── state.rs            # 全局应用状态
│   │   ├── error.rs            # 错误类型定义
│   │   └── lib.rs              # Tauri 入口
│   ├── tauri.conf.json         # Tauri 配置
│   └── capabilities/           # 权限配置
│
├── packages/
│   └── shared/                 # 共享 TypeScript 类型
│       └── src/
│           ├── conversation.ts              # 对话类型
│           ├── message.ts                   # 消息类型（含 feedback）
│           ├── model.ts                     # 模型类型
│           ├── settings.ts                  # 设置类型
│           ├── tool.ts                      # 工具类型
│           ├── bookmark.ts                  # 书签类型
│           ├── tab.ts                       # 标签页类型
│           └── download.ts                  # 下载类型
│
├── playwright-service/         # Playwright 自动化（可选）
│
├── examples/                   # 示例 Skills
│   ├── echo-skill.json
│   ├── python-calculator-skill.json
│   └── alibabacloud-iqs-search-skill.json
│
└── scripts/                    # 构建/检查脚本
    ├── dev.ps1                 # 开发模式启动脚本
    ├── build.ps1               # 构建脚本
    └── check.ps1               # 全量检查脚本
```

### 核心模块说明

#### 1. Agent Loop (`src-tauri/src/ai/agent.rs` + `stream.rs`)

Agent Loop 是 CoSurf 最核心的 AI 能力，允许 AI 自主完成多步任务：

```
用户请求: "打开百度并总结首页内容"

第1轮: AI → open_url("baidu.com") → 打开新标签页
第2轮: AI → summarize_page() → 获取页面内容
第3轮: AI → 生成最终回答 → 流式输出给用户
```

关键能力：
- **多轮迭代**：循环执行 AI 推理和工具调用
- **流式输出**：通过 SSE 协议实时发送 AI 响应
- **MCP 工具直通**：MCP Server 的工具自动注册为 Agent 可用 function
- **重复调用检测**：防止 AI 陷入工具调用死循环
- **工具调用后续传**：工具执行完成后，流式输出自动继续

#### 2. MCP 客户端 (`src-tauri/src/ai/skills_executors/mcp.rs`)

原生 Rust 实现的 MCP (Model Context Protocol) 客户端：

- **三种传输模式**：stdio（子进程）、SSE、Streamable HTTP
- **JSON-RPC 2.0**：标准协议通信
- **工具自动发现**：连接后自动调用 `tools/list`，注册为 Agent 可用工具
- **工具直通注册**：每个 MCP 工具以 `mcp_{server}_{tool}` 命名，独立注册为 function
- **配置持久化**：MCP Server 配置存储在 SQLite，重启不丢失

#### 3. 工具调度器 (`src-tauri/src/ai/tools_impl/dispatcher.rs`)

统一的工具调度中心：
- **内置工具路由**：根据工具名称分发到对应实现模块
- **MCP 工具路由**：匹配 `mcp_*` 前缀，查找注册表并调用 MCP Server
- **Skills 路由**：匹配 Skills 名称，调用对应执行器

#### 4. 流式消息处理 (`src-web/src/stores/conversationStore.ts`)

前端流式消息管理：
- **appendStreamDelta**：追加流式内容块（区分 content / thinking）
- **消息反馈**：点赞/点踩状态管理，调用后端 API 持久化
- **工具调用后续传**：工具执行完成后重置消息状态为 streaming

## 🤖 AI 模型配置

CoSurf 支持以下 AI 模型提供商：

| 提供商 | 模型示例 | API 类型 |
|--------|---------|----------|
| OpenAI | gpt-4, gpt-4o, o1 | Chat Completions |
| Anthropic | claude-3-5-sonnet, claude-3-opus | Messages API |
| Google | gemini-2.0-flash, gemini-2.5-pro | GenerateContent |
| 智谱 AI | glm-4, glm-4v | Chat Completions |
| 月之暗面 | moonshot-v1 | Chat Completions |
| DeepSeek | deepseek-chat, deepseek-coder | Chat Completions |
| 豆包 | doubao-pro | Chat Completions |
| 通义千问 | qwen-max, qwen-plus | Chat Completions |
| Ollama | llama3, mistral (本地) | Chat Completions |

### 配置步骤

1. 打开 CoSurf，点击左下角 **设置** 图标
2. 选择 **模型** 标签页
3. 点击 **添加模型**
4. 填写以下信息：
   - **名称**：自定义显示名称（如 "GPT-4"）
   - **提供商**：选择对应的提供商
   - **模型 ID**：具体的模型名称（如 "gpt-4"）
   - **API Key**：从提供商处获取的密钥
   - **Base URL**：API 地址（大多数情况下使用默认值即可）
5. 点击 **保存**
6. 在模型列表中点击新添加的模型，设为 **活跃模型**

### 获取 API Key

- **OpenAI**: https://platform.openai.com/api-keys
- **Anthropic**: https://console.anthropic.com/
- **Google Gemini**: https://aistudio.google.com/app/apikey
- **智谱 AI**: https://open.bigmodel.cn/
- **DeepSeek**: https://platform.deepseek.com/
- **通义千问**: https://dashscope.console.aliyun.com/

### 本地模型 (Ollama)

1. 安装 [Ollama](https://ollama.ai/)
2. 拉取模型：`ollama pull llama3`
3. 在 CoSurf 中添加模型：
   - 提供商：Ollama
   - Base URL: `http://localhost:11434/v1`
   - 模型 ID: `llama3`

## 🛠️ AI 工具与 MCP 集成

### 内置工具

CoSurf 为 AI 提供了以下内置工具，使其能够与浏览器交互：

| 工具名称 | 功能描述 | 实现文件 |
|---------|---------|----------|
| `open_url` | 打开新的网页标签页 | `tools_impl/open_url.rs` |
| `summarize_page` | 总结当前页面内容 | `tools_impl/summarize_page.rs` |
| `web_search` | 联网搜索（阿里云 IQS） | `tools_impl/web_search.rs` |
| `run_command` | 执行系统命令 | `tools_impl/run_command.rs` |
| `web_agent` | 网页元素操作 | `tools_impl/web_agent.rs` |

> **注意**: web_search 需要在设置中配置 ALIYUN_IQS_API_KEY

### MCP Server 配置

CoSurf 内置 MCP 客户端，可直接连接任意 MCP Server：

1. 打开设置 → **MCP Servers** 标签页
2. 点击 **添加 MCP Server**
3. 选择传输类型：
   - **stdio**：本地进程（如 `npx @modelcontextprotocol/server-filesystem`）
   - **SSE**：Server-Sent Events 远程服务
   - **Streamable HTTP**：HTTP 流式远程服务
4. 填写连接参数并保存

配置示例（stdio 模式）：
```
名称: filesystem
类型: stdio
命令: npx
参数: -y @modelcontextprotocol/server-filesystem /path/to/dir
```

配置示例（HTTP 模式）：
```
名称: remote-tools
类型: streamableHttp
URL: http://localhost:8080/mcp
```

MCP Server 配置会**自动持久化到 SQLite**，重启后保留。连接成功后，MCP Server 提供的工具会自动注册为 Agent 可用 function，AI 可以直接调用。

### Skills 系统

Skills 是 CoSurf 的可扩展能力系统：

#### Skill 类型

1. **CLI Skills** — 执行命令行工具
2. **Script Skills** — 执行脚本（Python、JavaScript、PowerShell）
3. **MCP Skills** — 调用 MCP Server 工具

#### 如何导入 Skills

1. **通过设置界面**：打开设置 → Skills 标签页 → 点击"导入 Skill"
2. **直接放置文件**：将 `.json` 文件放到 `%APPDATA%\CoSurf\skills\` 目录，重启后自动加载

### Agent Loop 工作原理

```
用户请求: "打开百度并总结首页内容"

第1轮:
┌─────────────────────────────────────┐
│ AI 分析请求 → 决定调用 open_url     │
│ 返回: tool_calls=[open_url(url=...)]│
└─────────────────────────────────────┘
              ↓
┌─────────────────────────────────────┐
│ 后端执行工具 → 打开新标签页          │
│ 返回: { success: true }             │
└─────────────────────────────────────┘
              ↓
第2轮:
┌─────────────────────────────────────┐
│ AI 再次分析 → 决定调用 summarize_page│
│ 返回: tool_calls=[summarize_page()]  │
└─────────────────────────────────────┘
              ↓
┌─────────────────────────────────────┐
│ 后端提取页面内容 → 发送给 AI         │
│ 返回: 页面文本内容                   │
└─────────────────────────────────────┘
              ↓
第3轮:
┌─────────────────────────────────────┐
│ AI 生成最终回答 → 流式输出给用户     │
│ 返回: "百度首页主要包含..."          │
└─────────────────────────────────────┘
```

## 🔧 开发指南

### 添加新的 Tauri 命令

1. 在 `src-tauri/src/commands/` 创建新文件
2. 实现命令函数并添加 `#[tauri::command]` 宏
3. 在 `commands/mod.rs` 中导出模块
4. 在 `lib.rs` 的 `invoke_handler` 中注册
5. 前端通过 `invoke('command_name', { ... })` 调用

### 添加新的 AI 工具

1. 在 `tools.rs` 的 `get_available_tools_schemas()` 中添加工具 schema
2. 在 `tools_impl/` 下创建对应的实现文件
3. 在 `tools_impl/dispatcher.rs` 中注册路由
4. 更新系统提示词，让 AI 知道何时使用新工具

### 调试技巧

#### 前端调试
- 打开开发者工具：`Ctrl + Shift + I`
- 流式输出日志：控制台搜索 `[ConversationStore]`
- AIPanel 渲染日志：搜索 `[AIPanel]`

#### 后端调试
- 日志级别：设置 `RUST_LOG=debug` 查看详细日志
- Agent Loop 日志：搜索 `Agent Loop iteration` 查看工具调用过程
- 工具执行日志：搜索 `Found X tool calls` 查看 AI 返回的工具调用
- MCP 通信日志：搜索 `MCP tool call response` 查看 MCP 工具调用响应
- 数据库：使用 [DB Browser for SQLite](https://sqlitebrowser.org/) 查看数据

#### 常见问题

**端口冲突** — 如果 1420 端口被占用，修改 `src-web/vite.config.ts` 中的 `server.port`

**WebView2 问题** — 确保 Windows 已安装最新版本的 WebView2 Runtime

**Rust 编译失败** — 旧进程可能锁定 exe 文件，关闭所有 CoSurf 进程后重试

**MCP 工具调用无结果** — 确保 MCP Server 正常运行，检查设置中的连接参数

## 📝 路线图

### 近期计划
- [ ] 标签页分组和搜索
- [ ] 广告拦截（基于规则过滤）
- [ ] 阅读模式（提取正文）
- [ ] 密码管理器

### 中期计划
- [ ] 扩展系统（类似 Chrome 扩展）
- [ ] 多配置文件（工作/个人）
- [ ] 语音输入/输出

### 长期计划
- [ ] 跨平台支持（macOS, Linux）
- [ ] 同步功能（书签、历史、设置云同步）
- [ ] 协作浏览（多人同时浏览同一页面）

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

### 贡献流程

1. **Fork 本仓库**
2. **创建特性分支**：`git checkout -b feature/amazing-feature`
3. **提交更改**：`git commit -m 'Add some amazing feature'`
4. **推送到分支**：`git push origin feature/amazing-feature`
5. **开启 Pull Request**

### 代码规范

- **Rust**: 遵循 [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- **TypeScript**: 遵循项目中的 ESLint 配置
- **提交信息**: 使用 [Conventional Commits](https://www.conventionalcommits.org/)

在提交 PR 前，请运行：
```bash
pnpm check
```

## 📄 许可证

MIT License
