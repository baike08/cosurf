export type ToolCategory = "webpage" | "knowledge" | "search" | "custom";

export interface ToolDefinition {
  id: string;
  name: string;
  description: string;
  category: ToolCategory;
  icon: string;
  enabled: boolean;
  configSchema?: Record<string, ToolConfigField>;
}

export interface ToolConfigField {
  type: "string" | "number" | "boolean" | "select";
  label: string;
  description?: string;
  defaultValue?: unknown;
  options?: { label: string; value: string }[];
  required?: boolean;
  secret?: boolean;
}

export interface ToolInstance {
  toolId: string;
  enabled: boolean;
  config: Record<string, unknown>;
}

export const BUILT_IN_TOOLS: ToolDefinition[] = [
  {
    id: "webpage-summarize",
    name: "智能总结",
    description: "一键提取并总结当前网页的核心内容",
    category: "webpage",
    icon: "file-text",
    enabled: true,
  },
  {
    id: "webpage-agent",
    name: "网页操作 Agent",
    description: "AI 自动在网页上执行点击、填表、滚动等操作",
    category: "webpage",
    icon: "mouse-pointer",
    enabled: true,
  },
  {
    id: "webpage-screenshot",
    name: "截图与视觉理解",
    description: "对网页截图并发送给多模态大模型进行分析",
    category: "webpage",
    icon: "camera",
    enabled: true,
  },
  {
    id: "export-markdown",
    name: "导出 Markdown",
    description: "将对话或网页内容导出为 Markdown 文件",
    category: "knowledge",
    icon: "download",
    enabled: true,
  },
  {
    id: "web-search",
    name: "联网搜索",
    description: "让 AI 具备联网搜索能力（需配置搜索 API）",
    category: "search",
    icon: "search",
    enabled: false,
    configSchema: {
      apiKey: {
        type: "string",
        label: "搜索 API Key",
        description: "SerpAPI 或其他搜索服务的 API Key",
        secret: true,
      },
      engine: {
        type: "select",
        label: "搜索引擎",
        defaultValue: "serpapi",
        options: [
          { label: "SerpAPI", value: "serpapi" },
          { label: "Tavily", value: "tavily" },
        ],
      },
    },
  },
];
