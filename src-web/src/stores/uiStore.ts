import { create } from "zustand";

export type SidebarPanel = "bookmarks" | "history" | "conversations" | "downloads" | "none";
export type SettingsView = "general" | "models" | "tools" | "skills" | "mcp" | "shortcuts";

interface UIState {
  sidebarOpen: boolean;
  sidebarPanel: SidebarPanel;
  sidebarWidth: number; // 侧边栏宽度
  aiPanelOpen: boolean;
  aiPanelWidth: number; // AI面板宽度
  browserActionPanelOpen: boolean;
  settingsOpen: boolean;
  settingsView: SettingsView;

  toggleSidebar: () => void;
  setSidebarPanel: (panel: SidebarPanel) => void;
  setSidebarWidth: (width: number) => void;
  toggleAIPanel: () => void;
  setAIPanelWidth: (width: number) => void;
  toggleBrowserActionPanel: () => void;
  openSettings: (view?: SettingsView) => void;
  closeSettings: () => void;
  setSettingsView: (view: SettingsView) => void;
}

export const useUIStore = create<UIState>((set) => ({
  sidebarOpen: false,
  sidebarPanel: "none",
  sidebarWidth: 280, // 默认宽度
  aiPanelOpen: true,
  aiPanelWidth: 400, // 默认宽度
  browserActionPanelOpen: false,
  settingsOpen: false,
  settingsView: "general",

  toggleSidebar: () => {
    set((state) => ({ sidebarOpen: !state.sidebarOpen }));
  },

  setSidebarPanel: (panel) => {
    set((state) => ({
      sidebarOpen: state.sidebarPanel === panel && state.sidebarOpen ? false : true,
      sidebarPanel: panel,
    }));
  },

  setSidebarWidth: (width) => {
    // 最小200px，最大窗口宽度的50%
    const maxWidth = Math.floor(window.innerWidth * 0.5);
    const clampedWidth = Math.max(200, Math.min(width, maxWidth));
    set({ sidebarWidth: clampedWidth });
  },

  toggleAIPanel: () => {
    set((state) => ({ aiPanelOpen: !state.aiPanelOpen }));
  },

  setAIPanelWidth: (width) => {
    // 最小300px，最大窗口宽度的60%
    const maxWidth = Math.floor(window.innerWidth * 0.6);
    const clampedWidth = Math.max(300, Math.min(width, maxWidth));
    set({ aiPanelWidth: clampedWidth });
  },

  toggleBrowserActionPanel: () => {
    set((state) => ({ browserActionPanelOpen: !state.browserActionPanelOpen }));
  },

  openSettings: (view = "general") => {
    set({ settingsOpen: true, settingsView: view });
  },

  closeSettings: () => {
    set({ settingsOpen: false });
  },

  setSettingsView: (view) => {
    set({ settingsView: view });
  },
}));
