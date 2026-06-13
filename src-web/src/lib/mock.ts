import type {
  Tab,
  Conversation,
  Message,
  Bookmark,
  HistoryEntry,
  ModelConfig,
  ToolInstance,
} from "@cosurf/shared";

const now = new Date().toISOString();

export const mockTabs: Tab[] = [
  {
    id: "tab-1",
    title: "新标签页",
    url: "about:blank",
    favicon: "",
    isLoading: false,
    isMuted: false,
    isPinned: false,
    isDiscarded: false,
    isActive: true,
    order: 0,
    navigationHistory: ["about:blank"],
    navigationIndex: 0,
  },
];

export const mockConversations: Conversation[] = [
  {
    id: "conv-1",
    title: "帮我总结这篇文章的要点",
    isPinned: true,
    modelId: "model-1",
    messageCount: 6,
    createdAt: now,
    updatedAt: now,
  },
  {
    id: "conv-2",
    title: "如何使用 Rust 编写 Tauri 插件",
    isPinned: false,
    modelId: "model-1",
    messageCount: 12,
    createdAt: now,
    updatedAt: now,
  },
  {
    id: "conv-3",
    title: "分析这个页面的性能问题",
    isPinned: false,
    modelId: "model-2",
    messageCount: 4,
    createdAt: now,
    updatedAt: now,
  },
];

export const mockMessages: Message[] = [
  {
    id: "msg-1",
    conversationId: "conv-1",
    role: "user",
    content: "请帮我总结一下当前页面的主要内容，用中文回答。",
    thinkingContent: "",
    status: "complete",
    attachments: [
      {
        id: "att-1",
        type: "webpage",
        name: "CoSurf - AI Smart Browser",
        content: "CoSurf 是一款内置 AI 助手的智能浏览器...",
      },
    ],
    createdAt: now,
    updatedAt: now,
    feedback: "",
  },
  {
    id: "msg-2",
    conversationId: "conv-1",
    role: "assistant",
    content:
      "## 页面总结\n\n这是 **CoSurf** 的官方网站，一款内置 AI 助手的智能浏览器。主要特点包括：\n\n1. **内置 AI 对话** — 无需切换窗口，直接在浏览器中与 AI 对话\n2. **网页智能总结** — 一键提取网页核心内容\n3. **多模型支持** — 支持 OpenAI、Claude、本地模型等\n4. **工具系统** — 可扩展的 AI 工具生态\n\n> 这是一个开源项目，基于 Tauri + React 构建。",
    thinkingContent: "",
    status: "complete",
    attachments: [],
    createdAt: now,
    updatedAt: now,
    feedback: "",
  },
  {
    id: "msg-3",
    conversationId: "conv-1",
    role: "user",
    content: "它和普通浏览器相比有什么优势？",
    thinkingContent: "",
    status: "complete",
    attachments: [],
    createdAt: now,
    updatedAt: now,
    feedback: "",
  },
  {
    id: "msg-4",
    conversationId: "conv-1",
    role: "assistant",
    content:
      "CoSurf 相比普通浏览器的核心优势：\n\n### 🧠 AI 原生集成\n- 浏览网页时可随时调用 AI 分析、总结、翻译\n- AI 能理解当前页面上下文，提供更精准的回答\n\n### 🛠️ 自动化操作\n- 通过网页操作 Agent，AI 可以自动点击、填表、滚动\n- 支持自定义工作流\n\n### 🔒 隐私保护\n- 支持本地模型（Ollama），数据不出本机\n- 可控的数据分享设置",
    thinkingContent: "",
    status: "streaming",
    attachments: [],
    createdAt: now,
    updatedAt: now,
    feedback: "",
  },
];

export const mockBookmarks: Bookmark[] = [
  {
    id: "bm-1",
    title: "CoSurf 官网",
    url: "https://cosurf.dev",
    order: 0,
    createdAt: now,
  },
  {
    id: "bm-2",
    title: "React 官方文档",
    url: "https://react.dev",
    order: 1,
    createdAt: now,
  },
  {
    id: "bm-3",
    title: "Tauri 开发文档",
    url: "https://tauri.app",
    order: 2,
    createdAt: now,
  },
  {
    id: "bm-4",
    title: "Tailwind CSS",
    url: "https://tailwindcss.com",
    order: 3,
    createdAt: now,
  },
  {
    id: "bm-5",
    title: "GitHub",
    url: "https://github.com",
    order: 4,
    createdAt: now,
  },
];

export const mockHistory: HistoryEntry[] = [
  { id: "h-1", title: "CoSurf - AI 智能浏览器", url: "https://cosurf.dev", visitedAt: now },
  { id: "h-2", title: "React 官方文档", url: "https://react.dev/learn", visitedAt: now },
  { id: "h-3", title: "Tauri 应用框架", url: "https://tauri.app/start/", visitedAt: now },
  { id: "h-4", title: "Zustand 状态管理", url: "https://github.com/pmndrs/zustand", visitedAt: now },
  { id: "h-5", title: "Tailwind CSS 文档", url: "https://tailwindcss.com/docs", visitedAt: now },
];

export const mockModels: ModelConfig[] = [
  {
    id: "model-1",
    name: "GPT-4o",
    provider: "openai",
    modelId: "gpt-4o",
    baseUrl: "https://api.openai.com/v1",
    temperature: 0.7,
    topP: 1,
    maxTokens: 4096,
    isLocal: false,
    isActive: true,
  },
  {
    id: "model-2",
    name: "Claude Sonnet 4",
    provider: "anthropic",
    modelId: "claude-sonnet-4-20250514",
    baseUrl: "https://api.anthropic.com/v1",
    temperature: 0.7,
    topP: 1,
    maxTokens: 4096,
    isLocal: false,
    isActive: false,
  },
  {
    id: "model-3",
    name: "Ollama Llama3 (本地)",
    provider: "ollama",
    modelId: "llama3",
    baseUrl: "http://localhost:11434/v1",
    temperature: 0.7,
    topP: 0.9,
    maxTokens: 2048,
    isLocal: true,
    isActive: false,
  },
];

export const mockToolInstances: ToolInstance[] = [
  { toolId: "webpage-summarize", enabled: true, config: {} },
  { toolId: "webpage-agent", enabled: true, config: {} },
  { toolId: "webpage-screenshot", enabled: true, config: {} },
  { toolId: "export-markdown", enabled: true, config: {} },
  { toolId: "web-search", enabled: false, config: { engine: "serpapi" } },
];
