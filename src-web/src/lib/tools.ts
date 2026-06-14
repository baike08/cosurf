import { page as pageApi } from "@/lib/api";
import { on } from "@/lib/events";

/**
 * 页面内容提取结果
 */
export interface PageContentResult {
  tabId: string;
  content: string;
}

/**
 * 网页操作结果
 */
export interface WebActionResult {
  success: boolean;
  message: string;
}

/**
 * 监听页面内容返回事件
 */
export function onPageContent(callback: (result: PageContentResult) => void) {
  return on<PageContentResult>("cosurf:page-content", (payload) => {
    callback(payload);
  });
}

/**
 * 监听页面内容提取错误事件
 */
export function onPageContentError(callback: (error: { tabId: string; error: string }) => void) {
  return on<{ tabId: string; error: string }>("cosurf:page-content-error", (payload) => {
    callback(payload);
  });
}

/**
 * 智能总结当前页面
 */
export async function summarizeCurrentPage(tabId: string, _maxLength?: number): Promise<string> {
  try {
    const result = await pageApi.summarize(tabId);
    return result;
  } catch (error) {
    console.error("[Tool] Summarize page failed:", error);
    throw error;
  }
}

/**
 * 执行网页操作
 */
export async function executeWebAction(
  tabId: string,
  action: "click" | "fill" | "close_popup",
  selector: string,
  value?: string
): Promise<WebActionResult> {
  try {
    const result = await pageApi.executeAction(tabId, action, selector, value);
    
    return {
      success: true,
      message: result,
    };
  } catch (error) {
    console.error("[Tool] Web action failed:", error);
    return {
      success: false,
      message: String(error),
    };
  }
}

/**
 * 工具调用处理器 - 在 AI 对话中调用
 */
export class ToolExecutor {
  private activeTabId: string;

  constructor(activeTabId: string) {
    this.activeTabId = activeTabId;
  }

  /**
   * 更新活跃标签页 ID
   */
  updateActiveTab(tabId: string) {
    this.activeTabId = tabId;
  }

  /**
   * 执行工具调用
   */
  async executeTool(toolName: string, args: any): Promise<any> {
    switch (toolName) {
      case "summarize_page":
        return await this.handleSummarizePage(args);
      
      case "web_agent":
        return await this.handleWebAgent(args);
      
      default:
        throw new Error(`Unknown tool: ${toolName}`);
    }
  }

  /**
   * 处理页面总结工具
   */
  private async handleSummarizePage(args: any): Promise<string> {
    const maxLength = args.max_length || 500;
    return await summarizeCurrentPage(this.activeTabId, maxLength);
  }

  /**
   * 处理网页操作工具
   */
  private async handleWebAgent(args: any): Promise<WebActionResult> {
    const { action, selector, value } = args;
    return await executeWebAction(this.activeTabId, action, selector, value);
  }
}
