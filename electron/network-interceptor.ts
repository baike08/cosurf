/**
 * CoSurf 网络请求拦截器
 * 
 * 利用 Electron session.webRequest API 实现:
 * 1. 拦截 API 响应（如电商商品接口）
 * 2. 移除追踪脚本和广告请求
 * 3. 修改 CSP 头以允许脚本注入
 * 4. Cookie 管理
 */

import { session, ipcMain } from 'electron';

// ===== 追踪域名黑名单 =====
const TRACKING_DOMAINS = [
  'google-analytics.com',
  'googletagmanager.com',
  'doubleclick.net',
  'facebook.com/tr',
  'analytics.google.com',
  'hotjar.com',
  'mixpanel.com',
  'segment.io',
];

// ===== 拦截的请求记录（用于 AI 分析）=====
interface InterceptedRequest {
  url: string;
  method: string;
  timestamp: number;
  responseBody?: string;
  statusCode?: number;
}

const interceptedRequests: InterceptedRequest[] = [];
const MAX_INTERCEPTED = 100;

/**
 * 设置网络请求拦截
 */
export function setupNetworkInterception(): void {
  const ses = session.defaultSession;

  // ===== 1. 阻止追踪请求 =====
  ses.webRequest.onBeforeRequest(
    { urls: TRACKING_DOMAINS.map(d => `*://*.${d}/*`) },
    (_details, callback) => {
      // 取消追踪请求
      callback({ cancel: true });
    }
  );

  // ===== 2. 修改响应头（移除限制性 CSP）=====
  ses.webRequest.onHeadersReceived((details, callback) => {
    const headers = details.responseHeaders || {};

    // 修改 CSP 以允许 CoSurf 注入脚本
    if (headers['content-security-policy']) {
      // 移除 CSP 头，让我们的 preload 脚本可以正常注入
      delete headers['content-security-policy'];
    }
    if (headers['Content-Security-Policy']) {
      delete headers['Content-Security-Policy'];
    }

    // 移除 X-Frame-Options（允许在 BrowserView 中加载）
    if (headers['x-frame-options']) {
      delete headers['x-frame-options'];
    }
    if (headers['X-Frame-Options']) {
      delete headers['X-Frame-Options'];
    }

    callback({ responseHeaders: headers });
  });

  // ===== 3. 监听 API 请求（电商数据抓取）=====
  // 淘宝 H5 API
  ses.webRequest.onCompleted(
    { urls: ['*://h5api.m.taobao.com/*', '*://acs.m.taobao.com/*'] },
    (details) => {
      const record: InterceptedRequest = {
        url: details.url,
        method: details.method,
        timestamp: Date.now(),
        statusCode: details.statusCode,
      };

      interceptedRequests.push(record);

      // 保持最多 100 条记录
      if (interceptedRequests.length > MAX_INTERCEPTED) {
        interceptedRequests.shift();
      }

      // 通知主窗口
      try {
        const win = require('electron').BrowserWindow.getAllWindows()[0];
        if (win && !win.isDestroyed()) {
          win.webContents.send('network:api-intercepted', {
            url: details.url,
            method: details.method,
            statusCode: details.statusCode,
          });
        }
      } catch {
        // 忽略：窗口可能已关闭
      }
    }
  );

  console.log('[CoSurf] Network interception configured');
}

/**
 * 获取拦截的请求记录（供 AI 分析使用）
 */
export function getInterceptedRequests(): InterceptedRequest[] {
  return [...interceptedRequests];
}

/**
 * 清空拦截记录
 */
export function clearInterceptedRequests(): void {
  interceptedRequests.length = 0;
}

/**
 * 注册网络拦截相关的 IPC 处理器
 */
export function registerNetworkIpcHandlers(): void {
  ipcMain.handle('network:get_intercepted', () => {
    return getInterceptedRequests();
  });

  ipcMain.handle('network:clear_intercepted', () => {
    clearInterceptedRequests();
  });

  // Cookie 管理
  ipcMain.handle('cookies:get', async (_event, url: string) => {
    const cookies = await session.defaultSession.cookies.get({ url });
    return cookies;
  });

  ipcMain.handle('cookies:set', async (_event, cookie: Electron.CookiesSetDetails) => {
    await session.defaultSession.cookies.set(cookie);
  });

  ipcMain.handle('cookies:remove', async (_event, url: string, name: string) => {
    await session.defaultSession.cookies.remove(url, name);
  });

  ipcMain.handle('cookies:flush', async () => {
    await session.defaultSession.cookies.flushStore();
  });
}
