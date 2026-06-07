export type ThemeMode = "light" | "dark" | "system";

export type Language = "zh-CN" | "en-US";

export interface AppSettings {
  theme: ThemeMode;
  language: Language;
  fontSize: number;
  userName: string; // 用户显示名称
  panelDefaultHeight: number;
  panelOverlayMode: boolean;
  privacyMode: boolean;
  aiDataPrivacy: boolean;
  shortcuts: ShortcutConfig;
  userDataPath: string; // 用户数据路径（用于存储页面缓存等）
  // 注意: iqsApiKey 是独立配置，通过 store 顶层字段管理，不在 AppSettings 中
}

export interface ShortcutConfig {
  togglePanel: string;
  newTab: string;
  closeTab: string;
  focusAddressBar: string;
  newConversation: string;
  screenshot: string; // 截图快捷键
}

export const DEFAULT_SETTINGS: AppSettings = {
  theme: "system",
  language: "zh-CN",
  fontSize: 14,
  userName: "CoCo", // 默认用户名称
  panelDefaultHeight: 300,
  panelOverlayMode: true,
  privacyMode: false,
  aiDataPrivacy: false,
  shortcuts: {
    togglePanel: "Ctrl+J",
    newTab: "Ctrl+T",
    closeTab: "Ctrl+W",
    focusAddressBar: "Ctrl+L",
    newConversation: "Ctrl+Shift+N",
    screenshot: "Ctrl+Shift+X", // 截图快捷键
  },
  userDataPath: "", // 空字符串表示使用默认路径（系统临时目录/cosurf/data/pages）
};
