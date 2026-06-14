/**
 * 全局事件管理器
 * 用于处理后端和前端之间的双向通信
 */

import { on } from "@/lib/events";

type EventHandler = (payload: any) => void;

interface PendingRequest {
  resolve: (value: any) => void;
  reject: (error: any) => void;
  timeout: ReturnType<typeof setTimeout>;
}

class EventManager {
  private static instance: EventManager;
  private pendingRequests: Map<string, PendingRequest> = new Map();
  private requestIdCounter: number = 0;

  private constructor() {}

  static getInstance(): EventManager {
    if (!EventManager.instance) {
      EventManager.instance = new EventManager();
    }
    return EventManager.instance;
  }

  /**
   * 生成唯一的请求 ID
   */
  private generateRequestId(): string {
    return `req_${++this.requestIdCounter}_${Date.now()}`;
  }

  /**
   * 发送请求并等待响应
   */
  async sendRequest<T = any>(
    eventName: string,
    payload: any,
    responseEventName: string,
    timeoutMs: number = 10000
  ): Promise<T> {
    const requestId = this.generateRequestId();
    
    return new Promise<T>((resolve, reject) => {
      // 设置超时
      const timeout = setTimeout(() => {
        this.pendingRequests.delete(requestId);
        reject(new Error(`Request timeout: ${eventName}`));
      }, timeoutMs);

      // 存储请求
      this.pendingRequests.set(requestId, { resolve, reject, timeout });

      // 监听响应事件
      const unsubscribe = on(responseEventName, (payload: any) => {
        const { id, data, error } = payload;
        
        if (id === requestId) {
          clearTimeout(timeout);
          this.pendingRequests.delete(requestId);
          unsubscribe();

          if (error) {
            reject(new Error(error));
          } else {
            resolve(data);
          }
        }
      });

      // 发送请求
      if (window.electronAPI) {
        window.electronAPI.send(eventName, { ...payload, requestId });
      } else {
        reject(new Error('Electron API not available'));
      }
    });
  }

  /**
   * 注册事件处理器
   */
  registerHandler(eventName: string, handler: EventHandler): () => void {
    const unsubscribe = on(eventName, (payload: any) => {
      handler(payload);
    });

    return unsubscribe;
  }

  /**
   * 清理所有待处理的请求
   */
  cleanup() {
    for (const [, request] of this.pendingRequests) {
      clearTimeout(request.timeout);
      request.reject(new Error("Cleanup"));
    }
    this.pendingRequests.clear();
  }
}

export const eventManager = EventManager.getInstance();
