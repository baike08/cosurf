/**
 * Electron 通信桥接层
 * 
 * 替代 @tauri-apps/api 的 invoke/listen/emit，
 * 提供统一的 API 接口给前端 React 组件使用。
 * 
 * 迁移策略:
 *   - 将所有 `invoke('command', args)` 替换为 `electronBridge.invoke('command', args)`
 *   - 将所有 `listen('event', handler)` 替换为 `electronBridge.on('event', handler)`
 *   - 将所有 `emit('event', payload)` 替换为 `electronBridge.send('event', payload)`
 */

// ===== 类型声明 =====
declare global {
  interface Window {
    electronAPI: {
      invoke(channel: string, ...args: any[]): Promise<any>;
      on(channel: string, callback: (payload: any) => void): () => void;
      send(channel: string, ...args: any[]): void;
      once(channel: string, callback: (payload: any) => void): void;
      removeAllListeners(channel: string): void;
    };
    windowControls: {
      minimize(): Promise<void>;
      maximize(): Promise<void>;
      close(): Promise<void>;
      isMaximized(): Promise<boolean>;
    };
  }
}

// ===== invoke: 发送请求并等待回复 (替代 Tauri invoke) =====
export async function invoke<T = any>(channel: string, args?: Record<string, any>): Promise<T> {
  if (!window.electronAPI) {
    console.warn('[electronBridge] electronAPI not available, running in web-only mode');
    throw new Error('Electron API not available');
  }

  // 将 Tauri 风格的命名参数转换为 Electron IPC 的位置参数
  if (args && typeof args === 'object') {
    const values = Object.values(args);
    return window.electronAPI.invoke(channel, ...values);
  }

  return window.electronAPI.invoke(channel);
}

// ===== on: 监听主进程事件 (替代 Tauri listen) =====
export function on<T = any>(event: string, callback: (payload: T) => void): () => void {
  if (!window.electronAPI) {
    console.warn('[electronBridge] electronAPI not available');
    return () => {};
  }

  return window.electronAPI.on(event, callback);
}

// ===== send: 向主进程发送消息 (替代 Tauri emit) =====
export function send(event: string, payload?: any): void {
  if (!window.electronAPI) {
    console.warn('[electronBridge] electronAPI not available');
    return;
  }

  window.electronAPI.send(event, payload);
}

// ===== once: 一次性监听 =====
export function once<T = any>(event: string, callback: (payload: T) => void): void {
  if (!window.electronAPI) {
    console.warn('[electronBridge] electronAPI not available');
    return;
  }

  window.electronAPI.once(event, callback);
}

// ===== removeAllListeners: 移除所有监听器 =====
export function removeAllListeners(event: string): void {
  if (!window.electronAPI) return;
  window.electronAPI.removeAllListeners(event);
}

// ===== 窗口控制快捷方法 =====
export const windowControls = {
  minimize: () => window.windowControls?.minimize(),
  maximize: () => window.windowControls?.maximize(),
  close: () => window.windowControls?.close(),
  isMaximized: () => window.windowControls?.isMaximized(),
};

// ===== 检测是否在 Electron 环境中运行 =====
export function isElectron(): boolean {
  return typeof window !== 'undefined' && !!window.electronAPI;
}

// ===== 向后兼容: 导出 listen/emit 别名 =====
export const listen = on;
export const emit = send;
