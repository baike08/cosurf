/// <reference types="vite/client" />

// Electron preload 暴露的 API 类型
interface ElectronAPI {
  invoke(channel: string, ...args: any[]): Promise<any>;
  on(channel: string, callback: (payload: any) => void): () => void;
  send(channel: string, ...args: any[]): void;
  once(channel: string, callback: (payload: any) => void): void;
  removeAllListeners(channel: string): void;
}

interface WindowControls {
  minimize(): Promise<void>;
  maximize(): Promise<void>;
  close(): Promise<void>;
  isMaximized(): Promise<boolean>;
}

declare global {
  interface Window {
    electronAPI?: ElectronAPI;
    windowControls?: WindowControls;
  }
}
