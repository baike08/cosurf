import { useEffect, useRef, useCallback } from "react";
import { Sparkles, Globe } from "lucide-react";
import { useTabStore } from "@/stores/tabStore";
import { getDomain } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { listen, emit } from "@tauri-apps/api/event";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { LogicalPosition, LogicalSize } from "@tauri-apps/api/dpi";

// ─── 全局 Webview 管理器 ───────────────────────────────────────────────────
// 维护 tabId → label 的映射，在组件卸载时保留状态
const webviewManager = new Map<string, string>();

/**
 * WebContentView - 使用 Tauri 原生 Webview 加载网页
 * 每个标签页对应一个原生 Webview 实例，不受跨域限制
 */
export function WebContentView() {
  const tabs = useTabStore((s) => s.tabs);
  const activeTabId = useTabStore((s) => s.activeTabId);
  const addTab = useTabStore((s) => s.addTab);

  const activeTab = tabs.find((t: any) => t.id === activeTabId);

  // 用于定位原生 webview 的占位容器
  const contentRef = useRef<HTMLDivElement>(null);

  // ─── 计算 webview 位置 ─────────────────────────────────────────────────
  const calcWebviewRect = useCallback((): { x: number; y: number; width: number; height: number } | null => {
    if (!contentRef.current) return null;
    const rect = contentRef.current.getBoundingClientRect();
    return {
      x: Math.round(rect.left),
      y: Math.round(rect.top),
      width: Math.round(rect.width),
      height: Math.round(rect.height),
    };
  }, []);

  // ─── 创建/销毁/切换 Webview ─────────────────────────────────────────────
  const createWebview = useCallback(async (tabId: string, url: string): Promise<string | null> => {
    // 如果已存在，直接返回
    if (webviewManager.has(tabId)) {
      return webviewManager.get(tabId)!;
    }

    const rect = calcWebviewRect();
    if (!rect) {
      console.warn('[WebContentView] Cannot calculate webview rect');
      return null;
    }

    console.log(`[WebContentView] Creating webview via Rust for tab ${tabId}: ${url}`);

    try {
      const label = await invoke<string>('create_tab_webview', {
        tabId,
        url: normalizeUrl(url),
        x: rect.x,
        y: rect.y,
        width: rect.width,
        height: rect.height,
      });

      webviewManager.set(tabId, label);
      console.log(`[WebContentView] WebviewWindow created: ${label}`);
      return label;
    } catch (err) {
      console.error(`[WebContentView] Failed to create webview for tab ${tabId}:`, err);
      return null;
    }
  }, [calcWebviewRect]);

  // ─── 销毁 webview ──────────────────────────────────────────────────────
  const destroyWebview = useCallback(async (tabId: string) => {
    const label = webviewManager.get(tabId);
    if (label) {
      try {
        await invoke('close_tab_webview', { tabId });
        console.log(`[WebContentView] WebviewWindow closed: ${label}`);
      } catch (err) {
        console.warn(`[WebContentView] Failed to close webview: ${label}`, err);
      }
      webviewManager.delete(tabId);
    }
  }, []);

  // ─── 隐藏所有非活跃 webview，显示活跃 webview ───────────────────────────
  useEffect(() => {
    const syncWebviews = async () => {
      const rect = calcWebviewRect();

      for (const [tabId, label] of webviewManager.entries()) {
        const wv = await WebviewWindow.getByLabel(label);
        if (!wv) continue;

        if (tabId === activeTabId) {
          // 显示并聚焦活跃标签页的 webview
          try {
            await wv.show();
            if (rect) {
              await wv.setPosition(new LogicalPosition(rect.x, rect.y));
              await wv.setSize(new LogicalSize(rect.width, rect.height));
            }
            await wv.setFocus();
          } catch (err) {
            console.warn(`[WebContentView] Failed to show webview ${tabId}:`, err);
          }
        } else {
          // 隐藏其他 webview
          try {
            await wv.hide();
          } catch (err) {
            // 忽略
          }
        }
      }
    };

    syncWebviews();
  }, [activeTabId, tabs.length, calcWebviewRect]);

  // ─── 监听标签页变化，创建/销毁 webview ─────────────────────────────────
  useEffect(() => {
    const syncTabs = async () => {
      console.log('[WebContentView] syncTabs fired:', { tabCount: tabs.length, tabs: tabs.map((t: any) => ({ id: t.id, url: t.url })) });
      const currentTabIds = new Set(tabs.map((t: any) => t.id));

      // 销毁已关闭标签页的 webview
      for (const tabId of webviewManager.keys()) {
        if (!currentTabIds.has(tabId)) {
          console.log('[WebContentView] Destroying closed tab webview:', tabId);
          await destroyWebview(tabId);
        }
      }

      // 为非 about:blank 的标签页创建 webview
      for (const tab of tabs) {
        if (tab.url !== 'about:blank' && !webviewManager.has(tab.id)) {
          console.log('[WebContentView] Creating webview for tab:', { id: tab.id, url: tab.url });
          const label = await createWebview(tab.id, tab.url);
          console.log('[WebContentView] createWebview result:', { tabId: tab.id, label });
        } else if (tab.url === 'about:blank') {
          console.log('[WebContentView] Skipping about:blank tab:', tab.id);
        } else {
          console.log('[WebContentView] Tab already has webview:', { id: tab.id, label: webviewManager.get(tab.id) });
        }
      }
    };

    syncTabs();
  }, [tabs, createWebview, destroyWebview]);

  // ─── 监听标签页 URL 变化（导航） ──────────────────────────────────────
  useEffect(() => {
    const handleUrlChange = async () => {
      if (!activeTab || activeTab.url === 'about:blank') return;

      const label = webviewManager.get(activeTab.id);
      if (label) {
        // 关闭旧 webview 并创建新的来实现导航
        const rect = calcWebviewRect();
        if (!rect) return;

        try {
          await invoke('close_tab_webview', { tabId: activeTab.id });
          webviewManager.delete(activeTab.id);
          await createWebview(activeTab.id, activeTab.url);
        } catch (err) {
          console.warn('[WebContentView] Failed to navigate webview:', err);
        }
      }
    };

    handleUrlChange();
  }, [activeTab?.url, activeTab?.id, calcWebviewRect, createWebview]);

  // ─── 监听窗口大小变化，更新所有 webview 的位置 ─────────────────────────
  useEffect(() => {
    const handleResize = async () => {
      const rect = calcWebviewRect();
      if (!rect) return;

      for (const [, label] of webviewManager.entries()) {
        try {
          const wv = await WebviewWindow.getByLabel(label);
          if (wv) {
            await wv.setPosition(new LogicalPosition(rect.x, rect.y));
            await wv.setSize(new LogicalSize(rect.width, rect.height));
          }
        } catch (err) {
          // 忽略
        }
      }
    };

    window.addEventListener('resize', handleResize);
    return () => window.removeEventListener('resize', handleResize);
  }, [calcWebviewRect]);

  // ─── 监听后端获取标签页信息的请求 ──────────────────────────────────────
  useEffect(() => {
    const unlisten = listen<{ requestId: string; tabId: string }>(
      'webview:get-tab-info',
      async (event) => {
        const { requestId, tabId } = event.payload;
        const tab = tabs.find(t => t.id === tabId);
        if (tab) {
          try {
            await invoke('receive_page_content', {
              requestId,
              content: JSON.stringify({
                url: tab.url,
                title: tab.title,
                isLoading: tab.isLoading
              })
            });
          } catch (error) {
            console.error('[WebContentView] Failed to send tab info:', error);
          }
        }
      }
    );
    return () => { unlisten.then(fn => fn()); };
  }, [tabs]);

  // ─── 监听后端获取标签页 URL 的请求 ─────────────────────────────────────
  useEffect(() => {
    const unlisten = listen<{ tabId: string }>(
      'webview:get-tab-url',
      async (event) => {
        const { tabId } = event.payload;
        const tab = tabs.find(t => t.id === tabId);
        if (tab) {
          try {
            await emit('cosurf:tab-url-response', { tabId, url: tab.url });
          } catch (error) {
            console.error('[WebContentView] Failed to send URL response:', error);
          }
        }
      }
    );
    return () => { unlisten.then(fn => fn()); };
  }, [tabs]);

  // ─── 监听后端"打开新标签页"事件（链接拦截器触发） ──────────────────
  useEffect(() => {
    console.log('[WebContentView] Registering webview:create-tab event listener');
    const unlisten = listen<{ url: string; title: string }>(
      'webview:create-tab',
      (event) => {
        const { url, title } = event.payload;
        console.log('[WebContentView] 📌📌📌 Received create-tab event:', { url, title });
        addTab(url, title);
      }
    );
    return () => { unlisten.then(fn => fn()); };
  }, [addTab]);

  // ─── 渲染 ──────────────────────────────────────────────────────────────
  if (!activeTab || tabs.length === 0) {
    return <WelcomePage />;
  }

  if (!activeTab) {
    return <WelcomePage />;
  }

  // about:blank 标签页显示欢迎页（React 渲染），隐藏所有 webview
  const showWelcome = activeTab.url === 'about:blank';

  return (
    <div className="h-full w-full relative">
      {/* 占位容器：用于计算 webview 的位置和大小 */}
      <div
        ref={contentRef}
        className="absolute inset-0"
        style={{ zIndex: showWelcome ? 1 : 0 }}
      >
        {showWelcome && <WelcomePage />}
      </div>
    </div>
  );
}

// ─── WelcomePage 组件（保持不变） ─────────────────────────────────────────
function WelcomePage() {
  const activeTabId = useTabStore((s) => s.activeTabId);
  const updateTab = useTabStore((s) => s.updateTab);

  const navigateTo = (url: string) => {
    if (!activeTabId) return;
    updateTab(activeTabId, { url, title: getDomain(url), isLoading: true });
  };

  const quickLinks = [
    { name: "Google", url: "https://google.com", color: "bg-blue-500" },
    { name: "GitHub", url: "https://github.com", color: "bg-gray-700" },
    { name: "YouTube", url: "https://youtube.com", color: "bg-red-500" },
    { name: "Bilibili", url: "https://bilibili.com", color: "bg-pink-500" },
    { name: "知乎", url: "https://zhihu.com", color: "bg-blue-600" },
    { name: "baidu", url: "https://baidu.com", color: "bg-gray-600" },
    { name: "Stack Overflow", url: "https://stackoverflow.com", color: "bg-amber-600" },
    { name: "MDN", url: "https://developer.mozilla.org", color: "bg-indigo-600" },
  ];

  return (
    <div className="h-full flex flex-col items-center justify-center gap-6 overflow-y-auto">
      <div className="flex flex-col items-center gap-2">
        <div className="w-20 h-20 rounded-3xl bg-gradient-to-br from-brand-500 to-brand-700 flex items-center justify-center shadow-lg shadow-brand-500/20">
          <Sparkles className="w-10 h-10 text-white" />
        </div>
        <h1 className="text-2xl font-bold text-content mt-2">
          欢迎使用 CoSurf
        </h1>
        <p className="text-sm text-content-secondary">
          AI 原生的智能桌面浏览器
        </p>
      </div>

      <div className="flex flex-col items-center gap-3 w-full max-w-lg px-6">
        <div className="w-full relative">
          <div className="flex items-center gap-2 h-10 rounded-xl px-4 bg-surface-secondary border border-border hover:border-brand-500/50 transition-colors cursor-text">
            <Globe className="w-4 h-4 text-content-tertiary shrink-0" />
            <span className="text-xs text-content-tertiary">搜索或输入网址</span>
          </div>
        </div>

        <div className="text-xs text-content-tertiary flex items-center gap-2">
          <span className="w-8 h-px bg-border" />
          快捷链接
          <span className="w-8 h-px bg-border" />
        </div>
      </div>

      <div className="grid grid-cols-4 gap-3 px-6">
        {quickLinks.map((site) => (
          <div
            key={site.name}
            onClick={() => navigateTo(site.url)}
            className="flex flex-col items-center gap-2 p-4 rounded-2xl hover:bg-surface-hover cursor-pointer transition-colors group"
          >
            <div
              className={`w-12 h-12 rounded-xl ${site.color} flex items-center justify-center text-white text-lg font-bold shadow-md group-hover:scale-105 transition-transform`}
            >
              {site.name.charAt(0)}
            </div>
            <span className="text-xs text-content-secondary group-hover:text-content transition-colors">
              {site.name}
            </span>
          </div>
        ))}
      </div>

      <div className="mt-4 px-6">
        <div className="flex items-center gap-2 text-2xs text-content-tertiary">
          <Sparkles className="w-3 h-3" />
          <span>按 Ctrl+L 聚焦地址栏 · AI 对话面板随时唤醒</span>
        </div>
      </div>
    </div>
  );
}

// ─── URL 标准化函数 ──────────────────────────────────────────────────────
function normalizeUrl(input: string): string {
  if (input.startsWith("http://") || input.startsWith("https://")) {
    return input;
  }
  if (input.includes(".") && !input.includes(" ")) {
    return "https://" + input;
  }
  return "https://www.baidu.com/search?q=" + encodeURIComponent(input);
}
