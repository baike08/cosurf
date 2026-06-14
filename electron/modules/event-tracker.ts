/**
 * CoSurf 用户行为事件追踪器
 * 
 * 负责监听用户在浏览区的动作并持久化到 SQLite
 */

import { ipcMain, BrowserWindow } from 'electron';
import { TabManager } from '../window-manager';

// Native 模块（延迟加载）
let native: any = null;

function getNative(): any {
  if (!native) {
    try {
      const path = require('path');
      native = require(path.join(__dirname, '../../native/cosurf-native.node'));
    } catch {
      console.warn('[EventTracker] Native module not available');
      native = null;
    }
  }
  return native;
}

/**
 * 插入用户行为事件
 */
async function insertUserEvent(event: {
  id: string;
  type: string;
  timestamp: number;
  url?: string;
  tab_id?: string;
  window_id?: number;
  data: any; // JSON object (not string)
  created_at?: number; // 可选，默认使用当前时间
}) {
  try {
    const nat = getNative();
    if (!nat) {
      console.warn('[EventTracker] Native module not available, skipping event');
      return;
    }

    // 添加 created_at 字段（如果未提供）
    const fullEvent = {
      ...event,
      created_at: event.created_at || Date.now()
    };

    const eventJson = JSON.stringify(fullEvent);
    nat.dbInsertUserEvent(eventJson);
    console.log(`[EventTracker] ✅ Event inserted: ${event.type}`);
  } catch (err: any) {
    console.error('[EventTracker] ❌ Failed to insert event:', err.message);
  }
}

/**
 * 初始化事件追踪器
 */
export function initEventTracker(tabManager: TabManager, mainWindow: BrowserWindow): void {
  console.log('[EventTracker] 🚀 Initializing event tracker...');

  // 1. 监听标签页生命周期事件
  tabManager.on('tab-created', (data: any) => {
    const { id: tabId, url, title } = data;
    
    // 跳过初始标签页
    if (tabId === 'tab-initial') {
      console.log('[EventTracker] Skipping initial tab');
      return;
    }
    
    console.log(`[EventTracker] Tab created: ${tabId}`);
    insertUserEvent({
      id: `tab-open-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
      type: 'tab_open',
      timestamp: Date.now(),
      url: url || 'about:blank',
      tab_id: tabId,
      window_id: mainWindow.id,
      data: { 
        title: title || 'New Tab',
        is_initial: url === 'about:blank'
      }
    });
  });

  tabManager.on('tab-closed', (data: any) => {
    const { tabId } = data;
    console.log(`[EventTracker] Tab closed: ${tabId}`);
    insertUserEvent({
      id: `tab-close-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
      type: 'tab_close',
      timestamp: Date.now(),
      tab_id: tabId,
      window_id: mainWindow.id,
      data: {} // 空对象
    });
  });

  tabManager.on('tab-switched', (data: any) => {
    const { tabId, url } = data;
    console.log(`[EventTracker] Tab switched: ${tabId}`);
    insertUserEvent({
      id: `tab-switch-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
      type: 'tab_switch',
      timestamp: Date.now(),
      url: url,
      tab_id: tabId,
      window_id: mainWindow.id,
      data: {} // 空对象
    });
  });

  // 2. 监听窗口事件
  mainWindow.on('resize', () => {
    const [width, height] = mainWindow.getSize();
    insertUserEvent({
      id: `window-resize-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
      type: 'window_resize',
      timestamp: Date.now(),
      window_id: mainWindow.id,
      data: { width, height }
    });
  });

  // 3. 监听导航事件（通过 IPC）
  ipcMain.on('webview:navigated', (_event, { url, tabId }) => {
    console.log(`[EventTracker] Navigation: ${url}`);
    insertUserEvent({
      id: `url-change-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
      type: 'url_change',
      timestamp: Date.now(),
      url: url,
      tab_id: tabId,
      window_id: mainWindow.id,
      data: {} // 空对象
    });
  });

  // 4. 监听页面点击事件（通过 IPC）
  ipcMain.on('webview:page-click', (_event, { x, y, url, tabId, title }) => {
    insertUserEvent({
      id: `page-click-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
      type: 'page_click',
      timestamp: Date.now(),
      url: url,
      tab_id: tabId,
      window_id: mainWindow.id,
      data: { click_x: x, click_y: y, title }
    });
  });

  // 5. 监听页面停留时间（通过 IPC）
  ipcMain.on('webview:page-stay', (_event, { url, tabId, duration, title }) => {
    insertUserEvent({
      id: `page-stay-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
      type: 'page_stay',
      timestamp: Date.now(),
      url: url,
      tab_id: tabId,
      window_id: mainWindow.id,
      data: { duration, title }
    });
  });

  // 6. 定期清理旧数据（每小时执行一次）
  setInterval(() => {
    try {
      const nat = getNative();
      if (nat) {
        const count = nat.dbCleanupOldUserEvents();
        console.log(`[EventTracker] 🧹 Cleaned up ${count} old events`);
      }
    } catch (err: any) {
      console.error('[EventTracker] Failed to cleanup old events:', err.message);
    }
  }, 60 * 60 * 1000); // 1 hour

  console.log('[EventTracker] ✅ Event tracker initialized');
}
