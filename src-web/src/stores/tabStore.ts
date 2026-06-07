import { create } from "zustand";
import type { Tab } from "@cosurf/shared";
import { generateId } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";

interface TabState {
  tabs: Tab[];
  activeTabId: string | null;
  setActiveTab: (id: string) => void;
  addTab: (url?: string, title?: string) => string;
  closeTab: (id: string) => void;
  updateTab: (id: string, updates: Partial<Tab>) => void;
  reorderTabs: (fromIndex: number, toIndex: number) => void;
  // 导航操作
  navigateTo: (id: string, url: string) => void;
  goBack: (id: string) => boolean;
  goForward: (id: string) => boolean;
  canGoBack: (id: string) => boolean;
  canGoForward: (id: string) => boolean;
}

function createTab(url: string, title: string, id?: string): Tab {
  return {
    id: id || generateId(),
    title,
    url,
    isLoading: false,
    isMuted: false,
    isPinned: false,
    isDiscarded: false,
    isActive: false,
    order: 0,
    navigationHistory: [url],
    navigationIndex: 0,
  };
}

export const useTabStore = create<TabState>((set, get) => ({
  tabs: [
    {
      id: "tab-initial",
      title: "新标签页",
      url: "about:blank",
      isLoading: false,
      isMuted: false,
      isPinned: false,
      isDiscarded: false,
      isActive: true,
      order: 0,
      navigationHistory: ["about:blank"],
      navigationIndex: 0,
    },
  ],
  activeTabId: "tab-initial",

  setActiveTab: (id) => {
    console.log('[TabStore] 🔄 Switching to tab:', id);
    console.trace('[TabStore] 📍 setActiveTab called from:');
    
    set((state) => ({
      activeTabId: id,
      tabs: state.tabs.map((t) => ({
        ...t,
        isActive: t.id === id,
      })),
    }));
    
    // 通知后端设置活跃标签页
    invoke('set_active_tab', { tabId: id }).catch(err => {
      console.error('[TabStore] Failed to set active tab:', err);
    });
    
    // 【核心改进】强制聚焦到主窗口，确保应用在前台
    // 使用 requestAnimationFrame 确保 DOM 更新后执行
    requestAnimationFrame(() => {
      console.log('[TabStore] 🎯 Forcing window focus after tab switch');
      window.focus();
      
      // 延迟一小段时间让 window.focus() 生效
      setTimeout(() => {
        // 尝试聚焦到标签页容器
        const container = document.getElementById(`tab-container-${id}`);
        if (container) {
          container.focus();
          console.log('[TabStore] ✅ Focused tab container:', id);
        }
      }, 50);
    });
  },

  addTab: (url = "about:blank", title = "新标签页") => {
    const id = generateId();
    const state = get();
    // 【关键修复】传递 id 给 createTab，避免生成两个不同的 ID
    const newTab = createTab(url, title, id);
    newTab.isActive = true;
    newTab.order = state.tabs.length;

    console.log('[TabStore] ➕ Adding new tab:', { id, url, title, currentTabCount: state.tabs.length });

    set({
      activeTabId: id,
      tabs: [
        ...state.tabs.map((t) => ({ ...t, isActive: false })),
        newTab,
      ],
    });
    
    // 验证更新是否成功
    const newState = get();
    const foundTab = newState.tabs.find(t => t.id === id);
    console.log('[TabStore] ✅ Tab added, verification:', {
      newActiveTabId: newState.activeTabId,
      newTabCount: newState.tabs.length,
      tabFound: !!foundTab,
      foundTabUrl: foundTab?.url
    });
    
    // 通知后端设置活跃标签页
    invoke('set_active_tab', { tabId: id }).catch(err => {
      console.error('[TabStore] Failed to set active tab:', err);
    });
    
    // 【核心改进】强制聚焦到主窗口和新标签页
    // 【关键修复】使用多次重试机制，等待 DOM 更新
    let retryCount = 0;
    const maxRetries = 10;
    const focusWithRetry = () => {
      retryCount++;
      console.log(`[TabStore] 🎯 Focus attempt ${retryCount}/${maxRetries} for tab:`, id);
      
      // 首先确保浏览器窗口获得焦点
      window.focus();
      
      // 尝试聚焦到标签页容器
      const container = document.getElementById(`tab-container-${id}`);
      if (container) {
        container.focus();
        console.log('[TabStore] ✅ Focused new tab container:', id);
        return; // 成功，停止重试
      }
      
      // 如果还没找到，继续重试
      if (retryCount < maxRetries) {
        setTimeout(focusWithRetry, 100); // 每 100ms 重试一次
      } else {
        console.warn('[TabStore] ⚠️ Failed to focus tab container after', maxRetries, 'attempts');
      }
    };
    
    // 开始第一次聚焦尝试
    setTimeout(focusWithRetry, 50);

    return id;
  },

  closeTab: (id) => {
    const { tabs, activeTabId } = get();
    if (tabs.length <= 1) {
      // 最后一个标签页关闭时创建一个新标签页
      set({
        tabs: [createTab("about:blank", "新标签页")],
        activeTabId: null,
      });
      return;
    }

    const idx = tabs.findIndex((t) => t.id === id);
    const newTabs = tabs.filter((t) => t.id !== id);

    let newActiveId = activeTabId;
    if (activeTabId === id) {
      const fallback = newTabs[Math.min(idx, newTabs.length - 1)];
      newActiveId = fallback?.id ?? null;
    }

    set({
      tabs: newTabs.map((t, i) => ({
        ...t,
        order: i,
        isActive: t.id === newActiveId,
      })),
      activeTabId: newActiveId,
    });
  },

  updateTab: (id, updates) => {
    set((state) => ({
      tabs: state.tabs.map((t) =>
        t.id === id ? { ...t, ...updates } : t,
      ),
    }));
  },

  reorderTabs: (fromIndex, toIndex) => {
    set((state) => {
      const newTabs = [...state.tabs];
      const [moved] = newTabs.splice(fromIndex, 1);
      if (!moved) return state;
      newTabs.splice(toIndex, 0, moved);
      return {
        tabs: newTabs.map((t, i) => ({ ...t, order: i })),
      };
    });
  },

  // 导航到新 URL
  navigateTo: (id, url) => {
    set((state) => ({
      tabs: state.tabs.map((t) => {
        if (t.id !== id) return t;

        const newHistory = [
          ...t.navigationHistory.slice(0, t.navigationIndex + 1),
          url,
        ];

        return {
          ...t,
          url,
          isLoading: true,
          navigationHistory: newHistory,
          navigationIndex: newHistory.length - 1,
        };
      }),
    }));
  },

  // 后退
  goBack: (id) => {
    const state = get();
    const tab = state.tabs.find((t) => t.id === id);
    if (!tab || tab.navigationIndex <= 0) return false;

    const newIndex = tab.navigationIndex - 1;
    const newUrl = tab.navigationHistory[newIndex];
    if (!newUrl) return false;

    set((s) => ({
      tabs: s.tabs.map((t) =>
        t.id === id
          ? { ...t, url: newUrl, navigationIndex: newIndex, isLoading: true }
          : t,
      ),
    }));

    return true;
  },

  // 前进
  goForward: (id) => {
    const state = get();
    const tab = state.tabs.find((t) => t.id === id);
    if (!tab || tab.navigationIndex >= tab.navigationHistory.length - 1)
      return false;

    const newIndex = tab.navigationIndex + 1;
    const newUrl = tab.navigationHistory[newIndex];
    if (!newUrl) return false;

    set((s) => ({
      tabs: s.tabs.map((t) =>
        t.id === id
          ? { ...t, url: newUrl, navigationIndex: newIndex, isLoading: true }
          : t,
      ),
    }));

    return true;
  },

  // 是否可以后退
  canGoBack: (id) => {
    const tab = get().tabs.find((t) => t.id === id);
    return tab ? tab.navigationIndex > 0 : false;
  },

  // 是否可以前进
  canGoForward: (id) => {
    const tab = get().tabs.find((t) => t.id === id);
    return tab
      ? tab.navigationIndex < tab.navigationHistory.length - 1
      : false;
  },
}));
