export type ModelProvider =
  | "openai"
  | "anthropic"
  | "google"
  | "zhipu"
  | "moonshot"
  | "deepseek"
  | "doubao"
  | "qwen"
  | "ollama"
  | "custom";

export interface ModelConfig {
  id: string;
  name: string;
  provider: ModelProvider;
  modelId: string;
  apiKey?: string;
  baseUrl?: string;
  temperature: number;
  topP: number;
  maxTokens: number;
  isLocal: boolean;
  isActive: boolean;
}

export interface ModelProviderPreset {
  provider: ModelProvider;
  name: string;
  defaultBaseUrl: string;
  models: string[];
  isLocal: boolean;
}

export const MODEL_PROVIDER_PRESETS: ModelProviderPreset[] = [
  {
    provider: "openai",
    name: "OpenAI",
    defaultBaseUrl: "https://api.openai.com/v1",
    models: ["gpt-4o", "gpt-4o-mini", "gpt-4-turbo", "o1-preview"],
    isLocal: false,
  },
  {
    provider: "anthropic",
    name: "Anthropic (Claude)",
    defaultBaseUrl: "https://api.anthropic.com/v1",
    models: [
      "claude-sonnet-4-20250514",
      "claude-3-5-haiku-20241022",
      "claude-3-opus-20240229",
    ],
    isLocal: false,
  },
  {
    provider: "google",
    name: "Google (Gemini)",
    defaultBaseUrl: "https://generativelanguage.googleapis.com/v1beta",
    models: ["gemini-2.0-flash", "gemini-1.5-pro", "gemini-1.5-flash"],
    isLocal: false,
  },
  {
    provider: "zhipu",
    name: "智谱 AI",
    defaultBaseUrl: "https://open.bigmodel.cn/api/paas/v4",
    models: ["glm-4-plus", "glm-4-flash", "glm-4-long"],
    isLocal: false,
  },
  {
    provider: "moonshot",
    name: "月之暗面 (Kimi)",
    defaultBaseUrl: "https://api.moonshot.cn/v1",
    models: ["moonshot-v1-128k", "moonshot-v1-32k", "moonshot-v1-8k"],
    isLocal: false,
  },
  {
    provider: "deepseek",
    name: "DeepSeek",
    defaultBaseUrl: "https://api.deepseek.com",
    models: ["deepseek-chat", "deepseek-reasoner"],
    isLocal: false,
  },
  {
    provider: "doubao",
    name: "豆包 (字节跳动)",
    defaultBaseUrl: "https://ark.cn-beijing.volces.com/api/v3",
    models: ["doubao-pro-32k", "doubao-lite-32k", "doubao-pro-128k"],
    isLocal: false,
  },
  {
    provider: "qwen",
    name: "通义千问 (阿里云)",
    defaultBaseUrl: "https://dashscope.aliyuncs.com/compatible-mode/v1",
    models: ["qwen-max", "qwen-plus", "qwen-turbo", "qwen-long"],
    isLocal: false,
  },
  {
    provider: "ollama",
    name: "Ollama (本地)",
    defaultBaseUrl: "http://localhost:11434/v1",
    models: ["llama3", "qwen2", "deepseek-coder", "mistral"],
    isLocal: true,
  },
];
