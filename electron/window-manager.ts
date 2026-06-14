/**
 * CoSurf 多标签页管理器
 * 
 * 使用 Electron 的 WebContentsView 实现真正的多标签页浏览。
 * 每个标签页拥有独立的渲染进程，不受 X-Frame-Options / CSP 限制。
 * 
 * 这是对比 Tauri iframe 方案的核心优势:
 * - 每个 tab 独立渲染进程，互不影响
 * - 可通过 preload 注入脚本到任何页面
 * - 可通过 webContents.executeJavaScript() 操作任何页面的 DOM
 * - 可通过 webContents.capturePage() 截取任何页面
 */

import { BrowserWindow } from 'electron';
import { EventEmitter } from 'events';

// ===== 标签页信息接口 =====
export interface TabInfo {
  id: string;
  url: string;
  title: string;
  isLoading: boolean;
  canGoBack: boolean;
  canGoForward: boolean;
  favicon?: string;
}

// ===== 标签页管理器 =====
export class TabManager extends EventEmitter {
  private mainWindow: BrowserWindow;
  private tabInfo: Map<string, TabInfo> = new Map();
  private activeTabId: string | null = null;
  private tabCounter = 0;

  constructor(mainWindow: BrowserWindow) {
    super();
    this.mainWindow = mainWindow;

    // 拦截 iframe 中的弹出窗口请求 (target="_blank")
    // 将其转为 CoSurf 新标签页
    this.setupPopupInterceptor();
  }

  /**
   * 拦截 iframe 中 target="_blank" 链接的弹出窗口请求
   * 当 iframe 内的链接尝试打开新窗口时，改为在 CoSurf 内创建新标签页
   */
  private setupPopupInterceptor(): void {
    this.mainWindow.webContents.setWindowOpenHandler(({ url }) => {
      console.log(`[TabManager] Intercepted popup request: ${url}`);
      
      // 忽略 about:blank (iframe 内部行为)
      if (url === 'about:blank') {
        return { action: 'deny' };
      }

      // 在 CoSurf 内创建新标签页
      const newTabId = this.generateTabId();
      // 使用 setImmediate 避免在事件处理中调用
      setImmediate(() => {
        this.createTab(newTabId, url, '加载中...');
        // 通知前端
        this.mainWindow.webContents.send('webview:create-tab', {
          requestId: newTabId,
          url,
          title: url,
        });
      });

      // 阻止 Electron 创建新窗口
      return { action: 'deny' };
    });
  }

  private generateTabId(): string {
    this.tabCounter++;
    return `tab-${Date.now()}-${this.tabCounter}`;
  }

  /**
   * 创建新标签页（仅跟踪元数据，网页渲染由 React <webview> tag 处理）
   */
  createTab(id: string, url: string, title: string = '新标签页'): TabInfo {
    if (this.tabInfo.has(id)) {
      console.warn(`[TabManager] Tab ${id} already exists`);
      return this.tabInfo.get(id)!;
    }

    // 初始化标签页信息
    const info: TabInfo = {
      id,
      url,
      title,
      isLoading: url !== 'about:blank',
      canGoBack: false,
      canGoForward: false,
    };

    this.tabInfo.set(id, info);
    this.switchTab(id);

    console.log(`[TabManager] Created tab: ${id} -> ${url}`);
    this.emit('tab-created', info);

    return info;
  }

  // WebContents 监听已移除 — 网页渲染由 React iframe 处理

  /**
   * 切换活跃标签页
   */
  switchTab(id: string): void {
    if (!this.tabInfo.has(id)) {
      console.warn(`[TabManager] Tab ${id} not found`);
      return;
    }

    this.activeTabId = id;

    // 通知前端
    const info = this.tabInfo.get(id);
    this.mainWindow.webContents.send('tab:switched', {
      tabId: id,
      url: info?.url || '',
      title: info?.title || '',
      canGoBack: false,
      canGoForward: false,
    });

    console.log(`[TabManager] Switched to tab: ${id}`);
  }

  /**
   * 关闭标签页
   */
  closeTab(id: string): void {
    if (!this.tabInfo.has(id)) {
      console.warn(`[TabManager] Tab ${id} not found for closing`);
      return;
    }

    this.tabInfo.delete(id);

    // 如果关闭的是活跃标签页，切换到相邻标签页
    if (this.activeTabId === id) {
      const remaining = Array.from(this.tabInfo.keys());
      if (remaining.length > 0) {
        this.switchTab(remaining[remaining.length - 1]!);
      } else {
        this.activeTabId = null;
      }
    }

    console.log(`[TabManager] Closed tab: ${id}`);
    this.emit('tab-closed', { tabId: id });
  }

  /**
   * 导航到指定 URL（更新元数据，实际导航由 React <webview> tag 处理）
   */
  navigate(id: string, url: string): void {
    const info = this.tabInfo.get(id);
    if (info) {
      info.url = url;
      info.isLoading = true;
    }
  }

  /**
   * 后退（由 React iframe 处理）
   */
  goBack(_id: string): boolean {
    return false;
  }

  /**
   * 前进（由 React iframe 处理）
   */
  goForward(_id: string): boolean {
    return false;
  }

  /**
   * 刷新页面（由 React iframe 处理）
   */
  reload(_id: string): void {
    // 由 React iframe 处理
  }

  /**
   * 获取标签页信息
   */
  getTabInfo(id: string): TabInfo | undefined {
    return this.tabInfo.get(id);
  }

  /**
   * 获取所有标签页信息
   */
  getAllTabInfo(): TabInfo[] {
    return Array.from(this.tabInfo.values());
  }

  /**
   * 获取活跃标签页 ID
   */
  getActiveTabId(): string | null {
    return this.activeTabId;
  }

  /**
   * 获取标签页的视图（已弃用 — 使用 React iframe 替代）
   */
  getTabView(_id: string): undefined {
    return undefined;
  }

  /**
   * 在指定标签页执行 JavaScript（已弃用 — 使用 React iframe 替代）
   */
  async executeJavaScript(_id: string, _code: string): Promise<any> {
    return null;
  }

  /**
   * 截取指定标签页的页面截图（已弃用）
   */
  async capturePage(_id: string): Promise<string> {
    return '';
  }

  /**
   * 获取标签页数量
   */
  get tabCount(): number {
    return this.tabInfo.size;
  }

  /**
   * 销毁所有标签页
   */
  destroyAll(): void {
    for (const [id] of this.tabInfo) {
      this.closeTab(id);
    }
  }
}
