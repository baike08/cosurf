import { TabBar } from "./TabBar";
import { NavigationBar } from "./NavigationBar";
import { Sidebar } from "./Sidebar";
import { AIPanel } from "./AIPanel";
import { BrowserActionPanel } from "./BrowserActionPanel";
import { WebView2Container } from "./WebView2Container";
import { SettingsPage } from "@/components/settings/SettingsPage";
import { ScreenshotOverlay } from "@/components/ui/ScreenshotOverlay";
import { ToolboxPanel } from "./ToolboxPanel";
import { useUIStore } from "@/stores/uiStore";
import { useSettingsStore } from "@/stores/settingsStore";
import { useConversationStore } from "@/stores/conversationStore";
import { useTabStore } from "@/stores/tabStore";
import { useEffect, useRef, useCallback } from "react";
import { on } from "@/lib/events";

export function AppLayout() {
  const sidebarOpen = useUIStore((s) => s.sidebarOpen);
  const sidebarWidth = useUIStore((s) => s.sidebarWidth);
  const setSidebarWidth = useUIStore((s) => s.setSidebarWidth);
  const browserActionPanelOpen = useUIStore((s) => s.browserActionPanelOpen);
  const loadModels = useSettingsStore((s) => s.loadModels);
  const loadConversations = useConversationStore((s) => s.loadConversations);

  // ========== 全局快捷键 ==========
  const toggleAIPanel = useUIStore((s) => s.toggleAIPanel);
  const addTab = useTabStore((s) => s.addTab);
  const closeTab = useTabStore((s) => s.closeTab);
  const createConversation = useConversationStore((s) => s.createConversation);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      const ctrl = e.ctrlKey;
      const shift = e.shiftKey;
      const key = e.key.toLowerCase();

      // 忽略输入框内的快捷键（除了 Ctrl 组合键）
      const target = e.target as HTMLElement;
      const isInput = target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable;
      if (isInput && !ctrl) return;

      // Ctrl+J — 切换 AI 面板
      if (ctrl && !shift && key === 'j') {
        e.preventDefault();
        toggleAIPanel();
        return;
      }

      // Ctrl+T — 新建标签页
      if (ctrl && !shift && key === 't') {
        e.preventDefault();
        addTab();
        return;
      }

      // Ctrl+W — 关闭当前标签页
      if (ctrl && !shift && key === 'w') {
        e.preventDefault();
        const { activeTabId } = useTabStore.getState();
        if (activeTabId) closeTab(activeTabId);
        return;
      }

      // Ctrl+L — 聚焦地址栏
      if (ctrl && !shift && key === 'l') {
        e.preventDefault();
        window.dispatchEvent(new CustomEvent('cosurf:focus-address-bar'));
        return;
      }

      // Ctrl+Shift+N — 新建对话
      if (ctrl && shift && key === 'n') {
        e.preventDefault();
        // 确保 AI 面板打开
        const { aiPanelOpen } = useUIStore.getState();
        if (!aiPanelOpen) toggleAIPanel();
        createConversation();
        return;
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [toggleAIPanel, addTab, closeTab, createConversation]);

  // 应用启动时加载模型和对话列表
  useEffect(() => {
    loadModels();
    loadConversations();
    
    // 【移除】不需要在这里设置初始活跃标签页
    // setActiveTab 已经会通知后端，避免重复调用导致状态不一致
  }, [loadModels, loadConversations]);

  // 用于跟踪最近创建的标签页，防止重复（使用 useRef 保持引用）
  const recentRequests = useRef(new Map<string, number>());

  // 监听创建新标签页请求（来自 AI 工具调用）
  useEffect(() => {
    // 定期清理过期的请求记录（每 30 秒清理一次）
    const cleanupInterval = setInterval(() => {
      const now = Date.now();
      const expiredUrls: string[] = [];
      
      recentRequests.current.forEach((timestamp, url) => {
        if (now - timestamp > 5000) { // 超过 5 秒的记录视为过期
          expiredUrls.push(url);
        }
      });
      
      expiredUrls.forEach(url => recentRequests.current.delete(url));
      
      if (expiredUrls.length > 0) {
        console.log('[AppLayout] 🧹 Cleaned up expired request records:', expiredUrls.length);
      }
    }, 30000);
    
    const unlisten = on<{ requestId: string; url: string; title: string }>(
      'webview:create-tab',
      async (payload) => {
        const { requestId, url, title } = payload;
        
        // 检查是否是重复请求（相同的 URL 在 2 秒内）
        const now = Date.now();
        const lastRequestTime = recentRequests.current.get(url);
        if (lastRequestTime && now - lastRequestTime < 2000) {
          return;
        }
        recentRequests.current.set(url, now);
        
        try {
          const addTab = useTabStore.getState().addTab;
          const newTabId = addTab(url, title);
          
          // 通知后端新标签页 ID
          window.electronAPI?.send('cosurf:new-tab-response', {
            requestId,
            tabId: newTabId
          });
        } catch (error) {
          console.error('[AppLayout] Failed to create tab:', error);
        }
      }
    );
    
    return () => {
      clearInterval(cleanupInterval);
      unlisten();
    };
  }, []);

  // 拖拽调整侧边栏宽度
  const isDragging = useRef(false);

  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    e.preventDefault();
    isDragging.current = true;
    const startX = e.clientX;
    const startWidth = sidebarWidth;

    const handleMouseMove = (moveEvent: MouseEvent) => {
      if (!isDragging.current) return;
      const delta = moveEvent.clientX - startX;
      setSidebarWidth(startWidth + delta);
    };

    const handleMouseUp = () => {
      isDragging.current = false;
      document.removeEventListener("mousemove", handleMouseMove);
      document.removeEventListener("mouseup", handleMouseUp);
      document.body.style.cursor = "";
      document.body.style.userSelect = "";
    };

    document.addEventListener("mousemove", handleMouseMove);
    document.addEventListener("mouseup", handleMouseUp);
    document.body.style.cursor = "col-resize";
    document.body.style.userSelect = "none";
  }, [sidebarWidth, setSidebarWidth]);

  return (
    <div className="h-full flex flex-col bg-surface">
      <NavigationBar />
      <TabBar />

      <div className="flex-1 flex overflow-hidden">
        {sidebarOpen && <Sidebar />}
        {sidebarOpen && (
          <div
            className="w-1 bg-transparent hover:bg-brand-500/20 cursor-col-resize shrink-0 transition-colors"
            onMouseDown={handleMouseDown}
          />
        )}

        <div className="flex-1 overflow-hidden bg-surface-secondary flex flex-col min-h-0">
          <WebView2Container />
        </div>

        {browserActionPanelOpen && <BrowserActionPanel />}
        <AIPanel />
      </div>

      <SettingsPage />
      <ToolboxPanel />
      <ScreenshotOverlay />
    </div>
  );
}
