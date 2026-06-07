import { TabBar } from "./TabBar";
import { NavigationBar } from "./NavigationBar";
import { Sidebar } from "./Sidebar";
import { AIPanel } from "./AIPanel";
import { BrowserActionPanel } from "./BrowserActionPanel";
import { WebView2Container } from "./WebView2Container";
import { SettingsPage } from "@/components/settings/SettingsPage";
import { ScreenshotOverlay } from "@/components/ui/ScreenshotOverlay";
import { useUIStore } from "@/stores/uiStore";
import { useSettingsStore } from "@/stores/settingsStore";
import { useConversationStore } from "@/stores/conversationStore";
import { useTabStore } from "@/stores/tabStore";
import { useEffect, useRef, useCallback } from "react";
import { listen } from "@tauri-apps/api/event";

export function AppLayout() {
  const sidebarOpen = useUIStore((s) => s.sidebarOpen);
  const sidebarWidth = useUIStore((s) => s.sidebarWidth);
  const setSidebarWidth = useUIStore((s) => s.setSidebarWidth);
  const browserActionPanelOpen = useUIStore((s) => s.browserActionPanelOpen);
  const loadModels = useSettingsStore((s) => s.loadModels);
  const loadConversations = useConversationStore((s) => s.loadConversations);

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
    
    const unlisten = listen<{ requestId: string; url: string; title: string }>(
      'webview:create-tab',
      async (event) => {
        const { requestId, url, title } = event.payload;
        console.log('[AppLayout] 📥 Received create-tab request:', { requestId, url, title });
        
        // 检查是否是重复请求（相同的 URL 在 2 秒内）
        const now = Date.now();
        const lastRequestTime = recentRequests.current.get(url);
        if (lastRequestTime && now - lastRequestTime < 2000) {
          console.log('[AppLayout] ⚠️ Duplicate request detected, ignoring:', url, `(last: ${now - lastRequestTime}ms ago)`);
          return;
        }
        recentRequests.current.set(url, now);
        
        try {
          // 创建新标签页（addTab 内部已经设置为 active 并聚焦）
          const addTab = useTabStore.getState().addTab;
          const newTabId = addTab(url, title);
          console.log('[AppLayout] ✅ New tab created:', { newTabId, url, title });
          
          // 【移除重复调用】addTab 已经调用了 set_active_tab
          // 不需要再次调用 invoke('set_active_tab')
          
          // 通知后端新标签页 ID
          const { emit } = await import('@tauri-apps/api/event');
          await emit('cosurf:new-tab-response', {
            requestId,
            tabId: newTabId
          });
          console.log('[AppLayout] 📤 Sent new-tab-response');
        } catch (error) {
          console.error('[AppLayout] ❌ Failed to create tab:', error);
        }
      }
    );
    
    return () => {
      clearInterval(cleanupInterval);
      unlisten.then(fn => fn());
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

        <div className="flex-1 flex flex-col overflow-hidden">
          <div className="flex-1 overflow-hidden bg-surface-secondary relative">
            <WebView2Container />
          </div>
        </div>

        {browserActionPanelOpen && <BrowserActionPanel />}
        <AIPanel />
      </div>

      <SettingsPage />
      <ScreenshotOverlay />
    </div>
  );
}
