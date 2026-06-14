/**
 * 统一事件适配层
 *
 * 替代 @tauri-apps/api/event 的 listen/emit，
 * 封装 Electron IPC 事件通信。
 *
 * 关键差异:
 *   - Tauri: `listen(event, (e) => handler(e.payload))` — payload 在 event.payload
 *   - Electron preload: `on(event, (payload) => handler(payload))` — 直接传递 payload
 *
 * 本模块抹平差异，提供与 Tauri 兼容的 API 签名。
 */

// ===== 事件名称常量 =====
export const Events = {
  // AI 流式事件
  AI_STREAM_CHUNK: 'ai:stream-chunk',
  AI_STREAM_ERROR: 'ai:stream-error',
  AI_TOOL_CALL_START: 'ai:tool-call-start',
  AI_TOOL_CALL_RESULT: 'ai:tool-call-result',

  // 标签页事件
  TAB_CREATE: 'tab:create',
  TAB_NAVIGATE: 'tab:navigate',
  TAB_TITLE_UPDATED: 'tab:title-updated',
  TAB_LOADING: 'tab:loading',
  TAB_LOADED: 'tab:loaded',
  TAB_SWITCHED: 'tab:switched',

  // 系统事件
  SHORTCUT_SCREENSHOT: 'shortcut:screenshot',
  UPDATER_UPDATE_AVAILABLE: 'updater:update-available',
  WEBVIEW_CREATE_TAB: 'webview:create-tab',
  COSURF_NEW_TAB_RESPONSE: 'cosurf:new-tab-response',
} as const;

// ===== 监听事件 (替代 Tauri listen) =====
/**
 * 持续监听主进程事件。
 * 返回取消订阅函数。
 *
 * @example
 * ```ts
 * const unsub = on('ai:stream-chunk', (payload) => {
 *   console.log(payload.delta);
 * });
 * // 取消:
 * unsub();
 * ```
 */
export function on<T = any>(event: string, callback: (payload: T) => void): () => void {
  if (!window.electronAPI) {
    console.warn('[events] electronAPI not available');
    return () => {};
  }
  return window.electronAPI.on(event, callback);
}

// ===== 一次性监听 (替代 Tauri once) =====
export function once<T = any>(event: string, callback: (payload: T) => void): void {
  if (!window.electronAPI) {
    console.warn('[events] electronAPI not available');
    return;
  }
  window.electronAPI.once(event, callback);
}

// ===== 取消监听 =====
export function off(unsubscribe: () => void): void {
  if (typeof unsubscribe === 'function') {
    unsubscribe();
  }
}

// ===== 移除通道全部监听 =====
export function removeAllListeners(event: string): void {
  if (!window.electronAPI) return;
  window.electronAPI.removeAllListeners(event);
}

// ===== 向后兼容: listen 别名 =====
export const listen = on;
