export interface Tab {
  id: string;
  title: string;
  url: string;
  favicon?: string;
  isLoading: boolean;
  isMuted: boolean;
  isPinned: boolean;
  isDiscarded: boolean;
  isActive: boolean;
  groupId?: string;
  order: number;
  // 导航历史
  navigationHistory: string[];
  navigationIndex: number;
}

export interface TabGroup {
  id: string;
  name: string;
  color: string;
  tabIds: string[];
}

export interface NavigationState {
  canGoBack: boolean;
  canGoForward: boolean;
  isLoading: boolean;
  url: string;
  title: string;
}
