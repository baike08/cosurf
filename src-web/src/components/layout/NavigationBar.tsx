import { useState, useRef, useEffect, useCallback } from "react";
import {
  ArrowLeft,
  ArrowRight,
  RotateCw,
  Home,
  Star,
  Download,
  Settings,
  Loader2,
  Globe,
  Lock,
  Bookmark,
  History,
  MessageSquare,
  MousePointer2,
  Wrench,
  Layers,
  Minus,
  Square,
  X,
} from "lucide-react";
import { useTabStore } from "@/stores/tabStore";
import { useUIStore } from "@/stores/uiStore";
import { useBookmarkStore } from "@/stores/bookmarkStore";
import { IconButton } from "@/components/ui/IconButton";
import { Tooltip } from "@/components/ui/Tooltip";
import { cn, getDomain } from "@/lib/utils";
import { isToolUrl, parseToolUrl } from "@/components/tools/ToolPage";
import { getCurrentWindow } from "@tauri-apps/api/window";

export function NavigationBar() {
  // 【关键修复】分别订阅 tabs 和 activeTabId，避免对象引用导致的无限循环
  const tabs = useTabStore((s) => s.tabs);
  const activeTabId = useTabStore((s) => s.activeTabId);
  const updateTab = useTabStore((s) => s.updateTab);
  const navigateTo = useTabStore((s) => s.navigateTo);
  const goBack = useTabStore((s) => s.goBack);
  const goForward = useTabStore((s) => s.goForward);
  const toggleAIPanel = useUIStore((s) => s.toggleAIPanel);
  const toggleBrowserActionPanel = useUIStore((s) => s.toggleBrowserActionPanel);
  const browserActionPanelOpen = useUIStore((s) => s.browserActionPanelOpen);
  const aiPanelOpen = useUIStore((s) => s.aiPanelOpen);
  const setSidebarPanel = useUIStore((s) => s.setSidebarPanel);
  const openSettings = useUIStore((s) => s.openSettings);
  const toggleToolbox = useUIStore((s) => s.toggleToolbox);
  const toolboxOpen = useUIStore((s) => s.toolboxOpen);

  const activeTab = tabs.find((t: any) => t.id === activeTabId);
  const [urlInput, setUrlInput] = useState(activeTab?.url ?? "");
  const [isFocused, setIsFocused] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  // 书签状态
  const bookmarks = useBookmarkStore((s) => s.bookmarks);
  const loadBookmarks = useBookmarkStore((s) => s.loadBookmarks);
  const addBookmark = useBookmarkStore((s) => s.addBookmark);
  const removeBookmarkByUrl = useBookmarkStore((s) => s.removeBookmarkByUrl);

  // 检查当前页面是否已收藏
  const isCurrentPageBookmarked = activeTab?.url
    ? bookmarks.some((b) => b.url === activeTab.url)
    : false;

  // 初始化加载书签
  useEffect(() => {
    loadBookmarks();
  }, [loadBookmarks]);

  // 切换书签
  const handleToggleBookmark = useCallback(async () => {
    if (!activeTab?.url || activeTab.url === "about:blank") return;
    if (isCurrentPageBookmarked) {
      await removeBookmarkByUrl(activeTab.url);
    } else {
      await addBookmark(
        activeTab.title || getDomain(activeTab.url),
        activeTab.url,
        undefined,
        undefined
      );
    }
  }, [activeTab, isCurrentPageBookmarked, addBookmark, removeBookmarkByUrl]);

  // 监听 Ctrl+L 聚焦地址栏事件
  useEffect(() => {
    const handleFocusAddressBar = () => {
      inputRef.current?.focus();
      inputRef.current?.select();
    };
    window.addEventListener('cosurf:focus-address-bar', handleFocusAddressBar);
    return () => window.removeEventListener('cosurf:focus-address-bar', handleFocusAddressBar);
  }, []);

  // 调试日志：检查导航栏的状态
  useEffect(() => {
    console.log('[NavigationBar] 📊 State:', {
      activeTabId,
      tabCount: tabs.length,
      activeTabUrl: activeTab?.url,
      activeTabTitle: activeTab?.title,
      urlInput,
      hasActiveTab: !!activeTab
    });
  }, [activeTabId, tabs, activeTab?.url, activeTab?.title, urlInput]);

  // 计算是否可以后退/前进
  const canGoBack = activeTabId ? useTabStore.getState().canGoBack(activeTabId) : false;
  const canGoForward = activeTabId ? useTabStore.getState().canGoForward(activeTabId) : false;

  useEffect(() => {
    // 只有在失焦且 URL 有效时才更新输入框
    if (!isFocused && activeTab?.url) {
      setUrlInput((prev) => {
        return prev !== activeTab.url ? activeTab.url : prev;
      });
    }
  }, [activeTab?.url, isFocused]);

  const handleGoBack = useCallback(() => {
    if (activeTabId) {
      goBack(activeTabId);
    }
  }, [activeTabId, goBack]);

  const handleGoForward = useCallback(() => {
    if (activeTabId) {
      goForward(activeTabId);
    }
  }, [activeTabId, goForward]);

  const handleReload = useCallback(() => {
    if (activeTabId && activeTab) {
      // 通过重新设置 URL 来刷新 iframe
      navigateTo(activeTabId, activeTab.url);
    }
  }, [activeTabId, activeTab, navigateTo]);

  const handleNavigate = useCallback(() => {
    if (!activeTabId || !urlInput.trim()) return;
    
    let url = urlInput.trim();
    
    // 处理特殊 URL
    if (url === "about:blank") {
      updateTab(activeTabId, { url, title: "新标签页", isLoading: false });
      return;
    }

    // 内置工具箱协议 cosurf://tools/xxx
    if (isToolUrl(url)) {
      const toolId = parseToolUrl(url);
      // 从工具箱定义中查找工具名称
      const toolNames: Record<string, string> = {
        "json-parser": "JSON 解析",
        "json-editor": "JSON 编辑",
        "json-validator": "JSON 检查",
        "regex-tester": "正则测试",
        "qrcode-generator": "二维码生成",
        "crypto": "加密解密",
        "text-diff": "文本对比",
      };
      const title = toolId ? (toolNames[toolId] || toolId) : "工具";
      navigateTo(activeTabId, url);
      updateTab(activeTabId, { title, isLoading: false });
      inputRef.current?.blur();
      return;
    }
    
    // 标准化 URL
    if (!url.startsWith("http://") && !url.startsWith("https://")) {
      if (url.includes(".") && !url.includes(" ")) {
        url = "https://" + url;
      } else {
        url = `https://www.baidu.com/s?wd=${encodeURIComponent(url)}`;
      }
    }
    
    // 直接使用前端导航
    navigateTo(activeTabId, url);
    updateTab(activeTabId, { title: getDomain(url), isLoading: true });
    
    // 失焦，让 useEffect 更新输入框显示
    inputRef.current?.blur();
  }, [activeTabId, urlInput, navigateTo, updateTab]);

  const handleHome = useCallback(() => {
    if (!activeTabId) return;
    navigateTo(activeTabId, "about:blank");
    updateTab(activeTabId, { title: "新标签页", isLoading: false });
  }, [activeTabId, navigateTo, updateTab]);

  const isLoading = activeTab?.isLoading ?? false;
  const isSecure = activeTab?.url?.startsWith("https://") ?? false;
  const domain = activeTab && activeTab.url !== "about:blank" ? getDomain(activeTab.url) : "";

  // 窗口控制
  const handleMinimize = useCallback(async () => {
    try { await getCurrentWindow().minimize(); } catch (e) { console.error('Failed to minimize:', e); }
  }, []);

  const handleMaximize = useCallback(async () => {
    try {
      const win = getCurrentWindow();
      if (await win.isMaximized()) { await win.unmaximize(); } 
      else { await win.maximize(); }
    } catch (e) { console.error('Failed to maximize:', e); }
  }, []);

  const handleClose = useCallback(async () => {
    try { await getCurrentWindow().close(); } catch (e) { console.error('Failed to close:', e); }
  }, []);

  return (
    <div className="h-nav-bar flex items-center gap-1 px-2 bg-surface border-b border-border select-none drag-region">
      {/* 导航按钮组 */}
      <div className="flex items-center gap-0.5 no-drag">
        <Tooltip label="后退">
          <IconButton size="sm" disabled={!canGoBack} onClick={handleGoBack}>
            <ArrowLeft />
          </IconButton>
        </Tooltip>
        <Tooltip label="前进">
          <IconButton size="sm" disabled={!canGoForward} onClick={handleGoForward}>
            <ArrowRight />
          </IconButton>
        </Tooltip>
        <Tooltip label="刷新">
          <IconButton size="sm" onClick={handleReload}>
            {isLoading ? (
              <Loader2 className="w-4 h-4 animate-spin" />
            ) : (
              <RotateCw />
            )}
          </IconButton>
        </Tooltip>
        <Tooltip label="主页">
          <IconButton size="sm" onClick={handleHome}>
            <Home />
          </IconButton>
        </Tooltip>
      </div>

      {/* 地址栏 */}
      <div
        className={cn(
          "flex-1 flex items-center h-8 rounded-lg px-3 gap-2 transition-all no-drag",
          "bg-surface-secondary border",
          isFocused
            ? "border-brand-500 ring-2 ring-brand-500/20"
            : "border-border hover:border-brand-500/50",
        )}
      >
        {/* 安全图标 */}
        {isLoading ? (
          <Loader2 className="w-3.5 h-3.5 text-brand-500 shrink-0 animate-spin" />
        ) : activeTab?.url === "about:blank" ? (
          <Globe className="w-3.5 h-3.5 text-content-tertiary shrink-0" />
        ) : isSecure ? (
          <Lock className="w-3.5 h-3.5 text-green-500 shrink-0" />
        ) : (
          <Globe className="w-3.5 h-3.5 text-amber-500 shrink-0" />
        )}

        {/* URL 输入 */}
        <input
          ref={inputRef}
          value={urlInput}
          onChange={(e) => setUrlInput(e.target.value)}
          onFocus={() => {
            setIsFocused(true);
            setTimeout(() => inputRef.current?.select(), 0);
          }}
          onBlur={() => setIsFocused(false)}
          onKeyDown={(e) => {
            if (e.key === "Enter") {
              e.preventDefault();
              handleNavigate();
            }
            if (e.key === "Escape") inputRef.current?.blur();
          }}
          className="flex-1 bg-transparent text-xs text-content outline-none placeholder:text-content-tertiary"
          placeholder="搜索或输入网址，Ctrl+L 聚焦"
        />

        {/* 书签按钮 */}
        {domain && (
          <Tooltip label={isCurrentPageBookmarked ? "取消收藏" : "添加书签"}>
            <IconButton
              size="sm"
              className={cn("shrink-0", isCurrentPageBookmarked && "text-amber-500")}
              onClick={handleToggleBookmark}
            >
              <Star className={cn("w-3.5 h-3.5", isCurrentPageBookmarked && "fill-current")} />
            </IconButton>
          </Tooltip>
        )}
      </div>

      {/* 右侧工具按钮 */}
      <div className="flex items-center gap-0.5 no-drag">
        <Tooltip label="标签管理">
          <IconButton
            size="sm"
            active={useUIStore.getState().sidebarPanel === "tabs" && useUIStore.getState().sidebarOpen}
            onClick={() => setSidebarPanel("tabs")}
          >
            <Layers />
          </IconButton>
        </Tooltip>
        <Tooltip label="工具箱">
          <IconButton
            size="sm"
            active={toolboxOpen}
            onClick={toggleToolbox}
          >
            <Wrench />
          </IconButton>
        </Tooltip>
        <Tooltip label="书签管理">
          <IconButton
            size="sm"
            active={useUIStore.getState().sidebarPanel === "bookmarks" && useUIStore.getState().sidebarOpen}
            onClick={() => setSidebarPanel("bookmarks")}
          >
            <Bookmark />
          </IconButton>
        </Tooltip>
        <Tooltip label="浏览历史">
          <IconButton
            size="sm"
            active={useUIStore.getState().sidebarPanel === "history" && useUIStore.getState().sidebarOpen}
            onClick={() => setSidebarPanel("history")}
          >
            <History />
          </IconButton>
        </Tooltip>
        <Tooltip label="AI 对话">
          <IconButton
            size="sm"
            active={aiPanelOpen}
            onClick={toggleAIPanel}
          >
            <MessageSquare />
          </IconButton>
        </Tooltip>
        <Tooltip label="浏览器操作">
          <IconButton
            size="sm"
            active={browserActionPanelOpen}
            onClick={toggleBrowserActionPanel}
          >
            <MousePointer2 />
          </IconButton>
        </Tooltip>
        <Tooltip label="下载管理">
          <IconButton
            size="sm"
            active={useUIStore.getState().sidebarPanel === "downloads" && useUIStore.getState().sidebarOpen}
            onClick={() => setSidebarPanel("downloads")}
          >
            <Download />
          </IconButton>
        </Tooltip>
        <Tooltip label="设置">
          <IconButton size="sm" onClick={() => openSettings()}>
            <Settings />
          </IconButton>
        </Tooltip>
      </div>

      {/* 窗口控制按钮 */}
      <div className="flex items-center gap-0.5 ml-1 no-drag">
        <button
          onClick={handleMinimize}
          className="w-8 h-8 flex items-center justify-center rounded hover:bg-surface-hover text-content-tertiary hover:text-content transition-colors"
          title="最小化"
        >
          <Minus className="w-3.5 h-3.5" />
        </button>
        <button
          onClick={handleMaximize}
          className="w-8 h-8 flex items-center justify-center rounded hover:bg-surface-hover text-content-tertiary hover:text-content transition-colors"
          title="最大化"
        >
          <Square className="w-3 h-3" />
        </button>
        <button
          onClick={handleClose}
          className="w-8 h-8 flex items-center justify-center rounded hover:bg-red-500 hover:text-white text-content-tertiary transition-colors"
          title="关闭"
        >
          <X className="w-3.5 h-3.5" />
        </button>
      </div>
    </div>
  );
}
