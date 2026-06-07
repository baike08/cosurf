import { useState, useEffect } from "react";
import { useTabStore } from "@/stores/tabStore";
import { onPageContent, onPageContentError } from "@/lib/tools";
import { invoke } from "@tauri-apps/api/core";

/**
 * 工具调用演示组件
 * 展示如何在 AI 对话中集成浏览器工具
 */
export function ToolDemo() {
  const activeTabId = useTabStore((s) => s.activeTabId);
  const [toolResult, setToolResult] = useState<string>("");
  const [isLoading, setIsLoading] = useState(false);

  useEffect(() => {
    // 监听页面内容返回
    const unlistenContent = onPageContent((result) => {
      console.log("[ToolDemo] Page content received:", result.content.length, "chars");
      setIsLoading(false);
      setToolResult(`提取到 ${result.content.length} 字符的页面内容`);
    });

    // 监听错误
    const unlistenError = onPageContentError((error) => {
      console.error("[ToolDemo] Page content error:", error);
      setIsLoading(false);
      setToolResult(`提取失败: ${error.error}`);
    });

    return () => {
      unlistenContent.then(fn => fn());
      unlistenError.then(fn => fn());
    };
  }, []);

  /**
   * 测试智能总结功能
   */
  const handleSummarize = async () => {
    if (!activeTabId) {
      setToolResult("请先打开一个网页");
      return;
    }

    setIsLoading(true);
    setToolResult("正在提取页面内容...");

    try {
      // 调用后端总结功能
      const result = await invoke<string>("summarize_page", {
        tabId: activeTabId,
        maxLength: 500,
      });
      
      setToolResult(result);
    } catch (error) {
      console.error("Summarize failed:", error);
      setToolResult(`总结失败: ${String(error)}`);
      setIsLoading(false);
    }
  };

  /**
   * 测试网页操作 - 点击元素
   */
  const handleClickElement = async () => {
    if (!activeTabId) {
      setToolResult("请先打开一个网页");
      return;
    }

    setIsLoading(true);
    
    try {
      const result = await invoke<string>("execute_web_action", {
        tabId: activeTabId,
        action: "click",
        selector: "button:first-of-type", // 点击第一个按钮
      });
      
      setToolResult(result);
      setIsLoading(false);
    } catch (error) {
      console.error("Click failed:", error);
      setToolResult(`点击失败: ${String(error)}`);
      setIsLoading(false);
    }
  };

  /**
   * 测试网页操作 - 填写表单
   */
  const handleFillForm = async () => {
    if (!activeTabId) {
      setToolResult("请先打开一个网页");
      return;
    }

    setIsLoading(true);
    
    try {
      const result = await invoke<string>("execute_web_action", {
        tabId: activeTabId,
        action: "fill",
        selector: "input[type='text']:first-of-type",
        value: "测试文本",
      });
      
      setToolResult(result);
      setIsLoading(false);
    } catch (error) {
      console.error("Fill failed:", error);
      setToolResult(`填写失败: ${String(error)}`);
      setIsLoading(false);
    }
  };

  /**
   * 测试关闭弹窗
   */
  const handleClosePopup = async () => {
    if (!activeTabId) {
      setToolResult("请先打开一个网页");
      return;
    }

    setIsLoading(true);
    
    try {
      const result = await invoke<string>("execute_web_action", {
        tabId: activeTabId,
        action: "close_popup",
        selector: "",
      });
      
      setToolResult(result);
      setIsLoading(false);
    } catch (error) {
      console.error("Close popup failed:", error);
      setToolResult(`关闭弹窗失败: ${String(error)}`);
      setIsLoading(false);
    }
  };

  return (
    <div className="p-4 space-y-4">
      <h3 className="text-sm font-medium text-content">浏览器工具测试</h3>
      
      <div className="space-y-2">
        <button
          onClick={handleSummarize}
          disabled={isLoading || !activeTabId}
          className="w-full px-3 py-2 rounded-lg text-xs bg-brand-600 text-white hover:bg-brand-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
        >
          {isLoading ? "处理中..." : "📝 智能总结当前页面"}
        </button>

        <button
          onClick={handleClickElement}
          disabled={isLoading || !activeTabId}
          className="w-full px-3 py-2 rounded-lg text-xs bg-surface-secondary text-content hover:bg-surface-hover disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
        >
          👆 点击第一个按钮
        </button>

        <button
          onClick={handleFillForm}
          disabled={isLoading || !activeTabId}
          className="w-full px-3 py-2 rounded-lg text-xs bg-surface-secondary text-content hover:bg-surface-hover disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
        >
          ✏️ 填写第一个输入框
        </button>

        <button
          onClick={handleClosePopup}
          disabled={isLoading || !activeTabId}
          className="w-full px-3 py-2 rounded-lg text-xs bg-surface-secondary text-content hover:bg-surface-hover disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
        >
          ❌ 关闭弹窗
        </button>
      </div>

      {toolResult && (
        <div className="mt-4 p-3 rounded-lg bg-surface-secondary border border-border">
          <div className="text-2xs text-content-tertiary mb-1">执行结果:</div>
          <div className="text-xs text-content whitespace-pre-wrap break-words">
            {toolResult}
          </div>
        </div>
      )}

      {!activeTabId && (
        <div className="mt-2 text-2xs text-content-tertiary italic">
          提示: 请先打开一个网页进行测试
        </div>
      )}
    </div>
  );
}
