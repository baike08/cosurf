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

### 🤖 AI 原生能力
- **多模型支持** — OpenAI、Anthropic Claude、Google Gemini、智谱、Kimi、DeepSeek、通义千问等
- **流式对话** — 实时流式响应，打字机效果，支持思考过程展示
- **智能工具调用** — AI 可自动打开网页、总结页面、操作网页、截图理解、翻译内容
- **Agent Loop** — 多轮工具调用，自主完成任务（如：打开网页 → 总结内容 → 回答问题）
- **Skills 系统** — 支持导入自定义 Skills（CLI、MCP、脚本），扩展 AI 能力
- **页面上下文感知** — AI 可感知当前浏览页面，提供精准回答
- **对话历史** — SQLite 持久化存储，随时回顾

### 🎨 用户体验
- **现代化 UI** — 简洁优雅的设计，支持亮色/暗色主题
- **快捷键** — 丰富的快捷键，提升操作效率
- **窗口记忆** — 记住窗口位置和大小，下次启动恢复
- **自动更新** — 静默检查并安装更新

## 🏗️ 技术架构

```
┌─────────────────────────────────────────────────────────────┐
│                      CoSurf Desktop                         │
├───────────────────────┬─────────────────────────────────────┤
│   Frontend (React)    │       Backend (Tauri/Rust)          │
│   src-web/            │       src-tauri/                    │
│                       │                                     │
│  ┌─────────────────┐  │  ┌───────────────────────────────┐  │
│  │  TabBar         │  │  │  Tauri Commands (40+)         │  │
│  │  NavigationBar  │  │  │  • conversation/message       │  │
│  │  Sidebar        │  │  │  • bookmark/browser           │  │
│  │  WebContentView │  │  │  • ai/page_context            │  │
│  │  (WebView2)     │◄─┼──┤  • settings/downloads         │  │
│  │  AIPanel        │  │  └───────────────┬───────────────┘  │
│  │  SettingsPage   │  │                  │                  │
│  └─────────────────┘  │  ┌───────────────▼───────────────┐  │
│                       │  │  SQLite (rusqlite)            │  │
│  zustand stores       │  │  7 tables:                    │  │
│  • tabStore           │  │  conversations, messages,     │  │
│  • conversationStore  │  │  bookmarks, history,          │  │
│  • settingsStore      │  │  settings, model_configs      │  │
│  • uiStore            │  └───────────────────────────────┘  │
│  • downloadStore      │                                     │
│                       │  ┌───────────────────────────────┐  │
│  @cosurf/shared ◄─────┼──┤  AI Provider Layer            │  │
│  (类型定义)            │  │  • OpenAI-compatible API      │  │
└───────────────────────┤  │  • Anthropic Messages API     │  │
                        │  │  • SSE Streaming              │  │
                        │  │  • Tool Calling (Function)    │  │
                        │  └───────────────────────────────┘  │
                        │                                     │
                        │  ┌───────────────────────────────┐  │
                        │  │  Playwright Service (可选)     │  │
                        │  │  • Browser automation         │  │
                        │  │  • Web Agent for AI           │  │
                        │  └───────────────────────────────┘  │
                        └─────────────────────────────────────┤
                                                              │
                        ┌─────────────────────────────────────┤
                        │  Windows Installer (NSIS/MSI)       │
                        │  • Auto-update (tauri-plugin-updater)│
                        │  • WebView2 bootstrapper            │
                        └─────────────────────────────────────┘
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
| HTTP | reqwest + eventsource |
| 浏览器内核 | WebView2 (Windows) |
| 自动化 | Playwright (可选侧车服务) |

## 🚀 快速开始

### 环境要求

- **Node.js** >= 20.0.0
- **pnpm** >= 9.0.0
- **Rust** >= 1.88.0 (Tauri 会自动安装)
- **Windows 10/11** (WebView2 已内置，无需额外安装)

### 克隆项目

```bash
git clone https://github.com/your-org/CoSurf.git
cd CoSurf
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
│   │   │   │   ├── AIPanel.tsx       # AI 对话面板（流式输出核心）
│   │   │   │   ├── AppLayout.tsx     # 应用主布局
│   │   │   │   ├── NavigationBar.tsx # 导航栏
│   │   │   │   ├── Sidebar.tsx       # 侧边栏
│   │   │   │   ├── TabBar.tsx        # 标签页栏
│   │   │   │   └── WebContentView.tsx # WebView2 容器
│   │   │   ├── sidebar/        # 侧边栏面板
│   │   │   ├── settings/       # 设置页面
│   │   │   └── ui/             # 基础 UI 组件
│   │   ├── stores/             # Zustand 状态管理
│   │   │   ├── conversationStore.ts  # 对话状态（流式消息处理）
│   │   │   ├── tabStore.ts           # 标签页状态
│   │   │   ├── settingsStore.ts      # 设置状态
│   │   │   ├── uiStore.ts            # UI 状态
│   │   │   └── downloadStore.ts      # 下载状态
│   │   ├── hooks/              # 自定义 Hooks
│   │   └── lib/                # 工具函数
│   └── ...
│
├── src-tauri/                  # Rust 后端
│   ├── src/
│   │   ├── commands/           # Tauri 命令处理器
│   │   │   ├── ai.rs                 # AI 对话命令
│   │   │   ├── conversation.rs       # 会话管理
│   │   │   ├── message.rs            # 消息管理
│   │   │   ├── browser.rs            # 浏览器控制
│   │   │   ├── page_context.rs       # 页面上下文提取
│   │   │   ├── bookmark.rs           # 书签管理
│   │   │   └── settings.rs           # 设置管理
│   │   ├── ai/                 # AI 提供者（流式、工具调用）
│   │   │   ├── stream.rs             # Agent Loop 核心实现
│   │   │   ├── provider.rs           # AI 提供商抽象
│   │   │   └── tools.rs              # 工具定义与执行
│   │   ├── db/                 # SQLite 数据库
│   │   │   ├── conversations.rs      # 会话表操作
│   │   │   ├── messages.rs           # 消息表操作
│   │   │   ├── bookmarks.rs          # 书签表操作
│   │   │   ├── history.rs            # 历史表操作
│   │   │   └── settings.rs           # 设置表操作
│   │   ├── state.rs            # 全局应用状态
│   │   ├── error.rs            # 错误类型定义
│   │   └── lib.rs              # Tauri 入口
│   ├── tauri.conf.json         # Tauri 配置
│   └── capabilities/           # 权限配置
│
├── packages/
│   └── shared/                 # 共享 TypeScript 类型
│       └── src/
│           ├── conversation.ts       # 对话类型定义
│           ├── message.ts            # 消息类型定义
│           ├── model.ts              # 模型类型定义
│           └── tool.ts               # 工具类型定义
│
├── playwright-service/         # Playwright 自动化（可选）
│
└── scripts/                    # 构建/检查脚本
    ├── dev.ps1                 # 开发模式启动脚本
    ├── build.ps1               # 构建脚本
    └── check.ps1               # 全量检查脚本
```

### 核心模块说明

#### 1. Agent Loop (`src-tauri/src/ai/stream.rs`)

这是 CoSurf 最核心的 AI 能力实现，负责：
- **多轮迭代**：循环执行 AI 推理和工具调用
- **流式输出**：通过 SSE 协议实时发送 AI 响应
- **工具检测**：解析 AI 返回的 `tool_calls`
- **工具执行**：调用内置工具并返回结果给 AI
- **状态管理**：维护对话历史和消息状态

关键函数：
```rust
pub async fn stream_chat(...) -> AppResult<()> {
    loop {
        // 1. 调用 AI API（流式）
        let tool_calls = stream_single_turn(...).await?;
        
        // 2. 如果没有工具调用，结束
        if tool_calls.is_empty() {
            break;
        }
        
        // 3. 执行工具
        for tool_call in tool_calls {
            let result = execute_tool(tool_call).await?;
            // 4. 将工具结果添加到对话历史
            current_messages.push(result.to_message());
        }
    }
}
```

#### 2. 流式消息处理 (`src-web/src/stores/conversationStore.ts`)

负责前端的流式消息更新：
- **appendStreamDelta**：追加流式内容块
- **finishStream**：标记流结束
- **状态重置**：工具调用后重置消息状态为 `streaming`

关键逻辑：
```typescript
appendStreamDelta: (delta, isThinking) => {
  set((state) => {
    const last = msgs[msgs.length - 1];
    if (last.status === "complete" && delta.length > 0) {
      // 工具调用后的新一轮，重置为 streaming
      msgs[msgs.length - 1] = { ...last, status: "streaming", content: last.content + delta };
    } else {
      // 正常流式追加
      msgs[msgs.length - 1] = { ...last, content: last.content + delta };
    }
    return { messages: msgs };
  });
}
```

#### 3. AIPanel 细粒度订阅 (`src-web/src/components/layout/AIPanel.tsx`)

解决 Zustand 浅比较导致的渲染问题：
```typescript
// ❌ 错误：浅比较无法检测嵌套对象变化
const messages = useConversationStore((s) => s.messages);

// ✅ 正确：订阅整个 store，确保检测到所有变化
const { messages, isStreaming } = useConversationStore();

// MessageList key 包含内容长度，确保内容变化时重新渲染
const listKey = `${messages.length}-${lastMsg.id}-${lastMsg.content.length}`;
```

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

## 🛠️ AI 工具与 Agent Loop

### 内置工具

CoSurf 为 AI 提供了以下内置工具，使其能够与浏览器交互：

| 工具名称 | 功能描述 | 参数 |
|---------|---------|------|
| `open_url` | 打开新的网页标签页 | `url`: 要打开的网址 |
| `summarize_page` | 总结当前页面内容 | `max_length`: 最大长度（可选） |
| `page_operation` | 操作页面元素（点击、输入等） | `action`, `selector`, `value` |
| `screenshot` | 截取当前页面截图 | `full_page`: 是否全屏截图 |
| `translate` | 翻译选中的文本 | `text`, `target_language` |
| `web_search` | 联网搜索（阿里云 IQS） | `query`, `engine_type`, `time_range`, `max_results` |

**注意**: web_search 需要在设置中配置 ALIYUN_IQS_API_KEY，详见 [IQS 配置指南](docs/IQS_CONFIGURATION.md)

### Skills 系统

Skills 是 CoSurf 的可扩展能力系统，允许用户导入自定义技能来增强 AI 的能力。

#### Skill 类型

1. **CLI Skills** - 执行命令行工具
   ```json
   {
     "type": "cli",
     "config": {
       "cli": {
         "command": "curl",
         "args_template": ["{{url}}"]
       }
     }
   }
   ```

2. **Script Skills** - 执行脚本（Python、JavaScript、Bash、PowerShell）
   ```json
   {
     "type": "script",
     "config": {
       "script": {
         "language": "python",
         "source": "print('Hello, ' + args['name'])"
       }
     }
   }
   ```

3. **MCP Skills** - 调用 MCP (Model Context Protocol) 服务器
   ```json
   {
     "type": "mcp",
     "config": {
       "mcp": {
         "server_url": "http://localhost:8080",
         "tool_name": "my_tool"
       }
     }
   }
   ```

#### 如何导入 Skills

1. **通过设置界面**：
   - 打开设置 → Skills 标签页
   - 点击“导入 Skill”按钮
   - 粘贴 JSON 定义或从文件导入

2. **直接放置文件**：
   - 将 `.json` 文件放到 `%APPDATA%\CoSurf\skills\` 目录
   - 重启应用后自动加载

#### 示例 Skills

项目提供了示例 Skills（在 `examples/` 目录）：
- `echo-skill.json` - 简单的回显测试
- `python-calculator-skill.json` - Python 计算器

#### Skill JSON 格式

```json
{
  "id": "unique-skill-id",
  "name": "Skill 名称",
  "description": "Skill 描述",
  "type": "cli|script|mcp|built_in",
  "enabled": true,
  "tags": ["tag1", "tag2"],
  "config": {
    // 根据 type 不同，配置也不同
    "cli": { ... },
    "script": { ... },
    "mcp": { ... },
    "parameters": {
      "type": "object",
      "properties": {
        "param1": { "type": "string" }
      }
    }
  }
}
```

### Agent Loop 工作原理

Agent Loop 是 CoSurf 的核心 AI 能力，允许 AI 自主完成多步任务：

```
用户请求: "打开百度并总结首页内容"

第1轮:
┌─────────────────────────────────────┐
│ AI 分析请求 → 决定调用 open_url     │
│ 返回: tool_calls=[open_url(url="..." )] │
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

### 流式输出机制

CoSurf 实现了真正的实时流式输出：

1. **SSE (Server-Sent Events)**: 后端通过 SSE 协议逐块发送 AI 响应
2. **Zustand 细粒度订阅**: 前端使用 Zustand store 管理消息状态，确保内容变化时立即重新渲染
3. **思考过程展示**: 区分 AI 的思考内容 (`thinkingContent`) 和正式回复 (`content`)
4. **工具调用后的续传**: 当 AI 调用工具后，流式输出会暂停，工具执行完成后自动继续

**关键实现细节**：
- 工具调用时不发送 `done=true`，保持连接活跃
- 使用字符级别的切片（`chars().take()`）避免 UTF-8 边界问题
- 消息状态重置逻辑确保工具调用后的新一轮输出能正确显示

## 🔧 开发指南

### 添加新的 Tauri 命令

1. 在 `src-tauri/src/commands/` 创建新文件（如 `my_feature.rs`）
2. 实现命令函数：
```rust
use tauri::AppHandle;
use crate::error::AppResult;

#[tauri::command]
pub async fn my_command(app: AppHandle, param: String) -> AppResult<String> {
    // 你的逻辑
    Ok(format!("Hello, {}!", param))
}
```
3. 在 `src-tauri/src/commands/mod.rs` 中导出模块：
```rust
pub mod my_feature;
```
4. 在 `src-tauri/src/lib.rs` 的 `invoke_handler` 中注册：
```rust
commands::my_feature::my_command,
```
5. 在前端通过 `invoke` 调用：
```typescript
import { invoke } from '@tauri-apps/api/core';

const result = await invoke('my_command', { param: 'World' });
```

### 添加新的 AI 工具

1. 在 `src-tauri/src/ai/tools.rs` 添加工具定义：
```rust
pub enum BuiltInTool {
    // ... 现有工具
    MyNewTool,
}

impl BuiltInTool {
    pub fn name(&self) -> &str {
        match self {
            // ...
            Self::MyNewTool => "my_new_tool",
        }
    }
    
    pub fn description(&self) -> &str {
        match self {
            // ...
            Self::MyNewTool => "描述这个工具的用途",
        }
    }
    
    pub fn parameters(&self) -> serde_json::Value {
        match self {
            // ...
            Self::MyNewTool => {
                serde_json::json!({
                    "type": "object",
                    "properties": {
                        "param1": {
                            "type": "string",
                            "description": "参数说明"
                        }
                    },
                    "required": ["param1"]
                })
            }
        }
    }
}
```
2. 在 `get_available_tools_schemas()` 中添加工具：
```rust
pub fn get_available_tools_schemas() -> Vec<serde_json::Value> {
    vec![
        // ... 现有工具
        BuiltInTool::MyNewTool.to_openai_schema(),
    ]
}
```
3. 在 `src-tauri/src/ai/stream.rs` 的 `execute_tool` 函数中添加工具执行逻辑：
```rust
match tool_call.name.as_str() {
    // ... 现有工具
    "my_new_tool" => {
        let param1 = tool_call.arguments.get("param1")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::Internal("Missing param1".into()))?;
        
        // 执行你的逻辑
        let result = do_something(param1).await?;
        
        Ok(ToolResult {
            tool_call_id: tool_call.id.clone(),
            output: result,
            success: true,
        })
    }
    _ => Err(AppError::Internal(format!("Unknown tool: {}", tool_call.name))),
}
```
4. **重要**：更新系统提示词，在 `src-tauri/src/commands/ai.rs` 中添加新工具的说明，让 AI 知道何时使用它。

### 添加新的前端页面

1. 在 `src-web/src/components/` 创建组件（如 `MyPage.tsx`）
2. 如需状态管理，在 `src-web/src/stores/` 创建 store：
```typescript
import { create } from 'zustand';

interface MyState {
  value: string;
  setValue: (v: string) => void;
}

export const useMyStore = create<MyState>((set) => ({
  value: '',
  setValue: (v) => set({ value: v }),
}));
```
3. 在 `src-web/src/components/layout/AppLayout.tsx` 中集成新页面
4. 添加路由或导航入口

### 调试技巧

#### 前端调试
- 打开开发者工具：`Ctrl + Shift + I`
- 查看 Zustand 状态：安装 [Zustand Devtools](https://github.com/charkour/zustand-devtools)
- 网络请求：Network 标签页查看 API 调用
- **流式输出日志**：在控制台搜索 `[ConversationStore]` 查看消息更新日志
- **AIPanel 渲染日志**：搜索 `[AIPanel]` 查看组件重新渲染情况

#### 后端调试
- 查看日志：运行 `pnpm dev:full` 时，Rust 日志会输出到终端
- 日志级别：设置环境变量 `RUST_LOG=debug` 查看详细日志
- 数据库：使用 [DB Browser for SQLite](https://sqlitebrowser.org/) 查看 `app_data_dir` 下的数据库文件
- **Agent Loop 日志**：搜索 `🔄 Agent Loop iteration` 查看多轮工具调用过程
- **工具执行日志**：搜索 `🔧 Found X tool calls` 查看 AI 返回的工具调用
- **SSE 流日志**：搜索 `📤 Emitting chunk` 查看流式输出情况

#### 常见问题

**1. AIPanel 流式输出不实时更新**
- 症状：AI 回复内容不会自动显示，需要切换标签页才能看到
- 原因：Zustand 浅比较无法检测嵌套对象变化
- 解决：已采用细粒度订阅机制，确保 `lastMessageContent` 和 `lastMessageThinking` 变化时触发重新渲染

**2. 工具调用后流式输出失效**
- 症状：AI 调用工具后，第二轮回复不显示
- 原因：第一轮结束时错误发送了 `done=true`，导致前端取消监听
- 解决：工具调用时不发送 `done=true`，保持连接活跃直到真正结束

**3. 后端 panic - byte index is not a char boundary**
- 症状：处理中文内容时崩溃
- 原因：使用字节切片在 UTF-8 字符中间截断
- 解决：改用字符级别的切片 `chars().take(n)`

**4. summarize_page 功能不可用**
- 症状：页面总结功能一直等待或返回空内容
- 原因：跨域限制导致无法提取 iframe 内容
- 解决：实现了基于 UUID 的请求-响应机制，但跨域网站仍受浏览器安全策略限制

**5. 端口冲突**
- 如果 1420 端口被占用，修改 `src-web/vite.config.ts` 中的 `server.port`

**6. WebView2 问题**
- 确保 Windows 已安装最新版本的 WebView2 Runtime
- 可在 Microsoft Edge 官网下载

**7. Rust 编译慢**
- 首次编译需要下载依赖，后续会使用缓存
- 可以使用 `cargo build --release` 进行增量编译

## 📝 路线图

### 近期计划
- [ ] 标签页分组和搜索
- [ ] 广告拦截（基于规则过滤）
- [ ] 阅读模式（提取正文）
- [ ] 密码管理器

### 中期计划
- [ ] 扩展系统（类似 Chrome 扩展）
- [ ] 多配置文件（工作/个人）
- [ ] AI 代理（自主浏览网页并回答问题）
- [ ] 语音输入/输出

### 长期计划
- [ ] 跨平台支持（macOS, Linux）
- [ ] 同步功能（书签、历史、设置云同步）
- [ ] 协作浏览（多人同时浏览同一页面）

## ⚡ 性能优化

### 前端优化

1. **Zustand 细粒度订阅**
   - 避免不必要的重新渲染
   - 使用选择器函数只订阅需要的状态
   - 对于流式更新，订阅整个 store 以确保检测到所有变化

2. **React Key 优化**
   - MessageList 使用内容长度作为 key，确保内容变化时重新渲染
   - 避免使用索引作为 key

3. **懒加载与代码分割**
   - Vite 自动进行代码分割
   - 大型组件可以使用 React.lazy 懒加载

### 后端优化

1. **UTF-8 字符串处理**
   - 使用 `chars().take(n)` 而非字节切片，避免 panic
   - 适用于所有涉及中文字符串截断的场景

2. **异步并发**
   - 使用 Tokio 异步运行时
   - 工具执行可以并行化（当前为串行，未来可优化）

3. **数据库优化**
   - SQLite 使用 WAL 模式提高并发性能
   - 定期清理过期数据（历史、下载记录等）

4. **SSE 流优化**
   - 工具调用时不发送 `done=true`，保持连接活跃
   - 减少不必要的网络往返

### 内存管理

1. **消息缓存**
   - 限制对话历史长度，避免内存泄漏
   - 定期归档旧对话

2. **WebView2 资源释放**
   - 关闭标签页时及时释放 WebView2 实例
   - 监控内存使用情况

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

### 运行检查

在提交 PR 前，请运行：

```bash
pnpm check
```

这会执行：
- TypeScript 类型检查
- ESLint 代码风格检查
- Cargo Clippy Rust 代码检查

### 报告 Bug

请使用 GitHub Issues 报告 Bug，并包含：
- 问题描述
- 复现步骤
- 预期行为
- 实际行为
- 截图（如适用）
- 环境信息（操作系统、CoSurf 版本等）

### 功能请求

欢迎提出新功能建议！请在 Issue 中说明：
- 功能描述
- 使用场景
- 预期效果

## 📄 许可证

MIT License
