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
        name: "CoSurf - AI 阅读伴侣",
        content: "CoSurf 是你的 AI 阅读伴侣和思考搭档...",
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
      "## 页面总结\n\n这是 **CoSurf** 的官方网站，你的 AI 阅读伴侣和思考搭档。主要特点包括：\n\n1. **读懂** — AI 帮你深度理解网页内容，提取关键信息\n2. **记住** — 自动标注要点，生成记忆卡片，让知识不流失\n3. **想起** — 跨文章关联召回，快速找回之前读过的内容\n4. **决策** — 基于阅读历史提供建议，辅助判断\n\n> 读过的，都算数。这不是一个工具，而是你阅读时的思考伙伴。",
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
      "CoSurf 相比普通浏览器的核心价值：\n\n### 🧠 思考搭档，不是工具\n- 在你看内容时，它帮你理解、记录、关联\n- 把阅读行为变成可沉淀的个人知识\n\n### 📚 读 → 记 → 想 → 决\n- **读懂**：智能摘要、术语解释、长文拆解\n- **记住**：自动生成记忆卡片，不用动手\n- **想起**：\"我之前读过一篇...\" 时快速召回\n- **决策**：基于阅读历史，多源信息对比\n\n### 💡 读过的，都算数\n- 解决 \"读了白读，看完就忘\" 的痛点\n- 每一次阅读都不白费",
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
  { id: "h-1", title: "CoSurf - AI 阅读伴侣", url: "https://cosurf.dev", visitedAt: now },
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
