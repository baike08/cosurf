import { useState, useCallback, useEffect } from "react";
import { MousePointer2, Hand, Type, Image as ImageIcon, Scroll, Trash2, FileText } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { useTabStore } from "@/stores/tabStore";

/**
 * BrowserActionPanel - 浏览器操作面板
 * 提供页面元素选择和操作功能
 */
export function BrowserActionPanel() {
  const activeTabId = useTabStore((s) => s.activeTabId);
  const [isSelectMode, setIsSelectMode] = useState(false);
  const [selectedElement, setSelectedElement] = useState<string | null>(null);
  const [textInput, setTextInput] = useState("");
  const [actionHistory, setActionHistory] = useState<string[]>([]);

  // 监听元素选择事件
  useEffect(() => {
    const handleElementSelected = (e: Event) => {
      const customEvent = e as CustomEvent;
      const { selector } = customEvent.detail;
      setSelectedElement(selector);
    };

    window.addEventListener('cosurf:element-selected', handleElementSelected);
    return () => {
      window.removeEventListener('cosurf:element-selected', handleElementSelected);
    };
  }, []);

  // 切换元素选择模式
  const toggleSelectMode = useCallback(async () => {
    if (!activeTabId) return;

    const newMode = !isSelectMode;
    setIsSelectMode(newMode);

    try {
      await invoke("browser_toggle_select_mode", {
        tabId: activeTabId,
        enabled: newMode,
      });
    } catch (err) {
      console.error("[BrowserActionPanel] Toggle select mode failed:", err);
    }
  }, [activeTabId, isSelectMode]);

  // 点击选中的元素
  const handleClickElement = useCallback(async () => {
    if (!activeTabId || !selectedElement) return;

    try {
      await invoke("browser_click_element", {
        tabId: activeTabId,
        selector: selectedElement,
      });
      
      setActionHistory(prev => [...prev, `Clicked: ${selectedElement}`]);
    } catch (err) {
      console.error("[BrowserActionPanel] Click failed:", err);
    }
  }, [activeTabId, selectedElement]);

  // 在选中的元素中输入文本
  const handleInputText = useCallback(async () => {
    if (!activeTabId || !selectedElement || !textInput) return;

    try {
      await invoke("browser_input_text", {
        tabId: activeTabId,
        selector: selectedElement,
        text: textInput,
      });
      
      setActionHistory(prev => [...prev, `Input "${textInput}" into: ${selectedElement}`]);
      setTextInput("");
    } catch (err) {
      console.error("[BrowserActionPanel] Input failed:", err);
    }
  }, [activeTabId, selectedElement, textInput]);

  // 滚动页面
  const handleScroll = useCallback(async (direction: "up" | "down" | "left" | "right") => {
    if (!activeTabId) return;

    try {
      await invoke("browser_scroll", {
        tabId: activeTabId,
        direction,
      });
      
      setActionHistory(prev => [...prev, `Scrolled ${direction}`]);
    } catch (err) {
      console.error("[BrowserActionPanel] Scroll failed:", err);
    }
  }, [activeTabId]);

  // 截图
  const handleScreenshot = useCallback(async () => {
    if (!activeTabId) return;

    try {
      const result = await invoke("browser_screenshot", {
        tabId: activeTabId,
        fullPage: false,
      });
      
      setActionHistory(prev => [...prev, "Screenshot taken"]);
      console.log("[BrowserActionPanel] Screenshot result:", result);
    } catch (err) {
      console.error("[BrowserActionPanel] Screenshot failed:", err);
    }
  }, [activeTabId]);

  // 提取页面内容
  const handleExtractContent = useCallback(async () => {
    if (!activeTabId) return;

    try {
      const script = `
        (function() {
          const clone = document.body.cloneNode(true);
          clone.querySelectorAll('script, style, noscript').forEach(el => el.remove());
          return clone.innerText.trim().substring(0, 5000);
        })()
      `;

      const result = await invoke("browser_execute_script", {
        tabId: activeTabId,
        script,
      });
      
      setActionHistory(prev => [...prev, "Content extracted"]);
      console.log("[BrowserActionPanel] Extracted content:", result);
      
      // TODO: 显示提取的内容
    } catch (err) {
      console.error("[BrowserActionPanel] Extract content failed:", err);
    }
  }, [activeTabId]);

  // 清除历史记录
  const clearHistory = useCallback(() => {
    setActionHistory([]);
  }, []);

  return (
    <div className="w-64 h-full bg-surface border-l border-border flex flex-col">
      {/* 标题 */}
      <div className="h-10 flex items-center px-3 border-b border-border">
        <span className="text-xs font-medium text-content">浏览器操作</span>
      </div>

      {/* 操作工具栏 */}
      <div className="p-3 space-y-3">
        {/* 元素选择模式 */}
        <div className="space-y-2">
          <button
            onClick={toggleSelectMode}
            className={`w-full flex items-center gap-2 px-3 py-2 rounded-lg text-xs transition-colors ${
              isSelectMode
                ? "bg-brand-600 text-white"
                : "bg-surface-secondary hover:bg-surface-hover text-content"
            }`}
          >
            <MousePointer2 className="w-4 h-4" />
            {isSelectMode ? "退出选择模式" : "选择元素"}
          </button>

          {selectedElement && (
            <div className="px-2 py-1.5 bg-surface-secondary rounded text-2xs text-content-tertiary truncate">
              {selectedElement}
            </div>
          )}
        </div>

        {/* 元素操作 */}
        <div className="space-y-2">
          <div className="text-2xs text-content-tertiary">元素操作</div>
          
          <button
            onClick={handleClickElement}
            disabled={!selectedElement}
            className="w-full flex items-center gap-2 px-3 py-2 rounded-lg text-xs bg-surface-secondary hover:bg-surface-hover text-content disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
          >
            <Hand className="w-4 h-4" />
            点击元素
          </button>

          <div className="space-y-1">
            <input
              type="text"
              value={textInput}
              onChange={(e) => setTextInput(e.target.value)}
              placeholder="输入文本..."
              disabled={!selectedElement}
              className="w-full px-3 py-2 rounded-lg text-xs bg-surface-secondary border border-border focus:border-brand-500 outline-none text-content placeholder:text-content-tertiary disabled:opacity-50"
            />
            <button
              onClick={handleInputText}
              disabled={!selectedElement || !textInput}
              className="w-full flex items-center gap-2 px-3 py-2 rounded-lg text-xs bg-surface-secondary hover:bg-surface-hover text-content disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
            >
              <Type className="w-4 h-4" />
              输入文本
            </button>
          </div>
        </div>

        {/* 页面操作 */}
        <div className="space-y-2">
          <div className="text-2xs text-content-tertiary">页面操作</div>
          
          <div className="grid grid-cols-2 gap-2">
            <button
              onClick={() => handleScroll("up")}
              className="flex items-center justify-center gap-1 px-2 py-2 rounded-lg text-xs bg-surface-secondary hover:bg-surface-hover text-content transition-colors"
            >
              <Scroll className="w-3.5 h-3.5" />
              向上
            </button>
            <button
              onClick={() => handleScroll("down")}
              className="flex items-center justify-center gap-1 px-2 py-2 rounded-lg text-xs bg-surface-secondary hover:bg-surface-hover text-content transition-colors"
            >
              <Scroll className="w-3.5 h-3.5 rotate-180" />
              向下
            </button>
          </div>

          <div className="grid grid-cols-2 gap-2">
            <button
              onClick={handleScreenshot}
              className="flex items-center justify-center gap-1 px-2 py-2 rounded-lg text-xs bg-surface-secondary hover:bg-surface-hover text-content transition-colors"
            >
              <ImageIcon className="w-3.5 h-3.5" />
              截图
            </button>
            <button
              onClick={handleExtractContent}
              className="flex items-center justify-center gap-1 px-2 py-2 rounded-lg text-xs bg-surface-secondary hover:bg-surface-hover text-content transition-colors"
            >
              <FileText className="w-3.5 h-3.5" />
              提取内容
            </button>
          </div>
        </div>

        {/* 操作历史 */}
        <div className="space-y-2">
          <div className="flex items-center justify-between">
            <div className="text-2xs text-content-tertiary">操作历史</div>
            {actionHistory.length > 0 && (
              <button
                onClick={clearHistory}
                className="text-2xs text-content-tertiary hover:text-content transition-colors"
              >
                <Trash2 className="w-3 h-3" />
              </button>
            )}
          </div>
          
          <div className="max-h-32 overflow-y-auto space-y-1">
            {actionHistory.length === 0 ? (
              <div className="text-2xs text-content-tertiary italic">暂无操作</div>
            ) : (
              actionHistory.map((action, index) => (
                <div key={index} className="text-2xs text-content-secondary px-2 py-1 bg-surface-secondary rounded">
                  {action}
                </div>
              ))
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
