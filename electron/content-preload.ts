/**
 * CoSurf Content Preload Script
 * 
 * 注入到每个标签页 (WebContentsView) 的渲染进程中。
 * 提供:
 * 1. 链接点击拦截 (替代 Tauri iframe 方案的 postMessage)
 * 2. 页面内容提取 API
 * 3. 安全的 IPC 通信通道
 * 
 * 运行环境: 标签页渲染进程 (contextIsolation: true)
 */

import { contextBridge, ipcRenderer } from 'electron';

// ===== 暴露安全的 API 给标签页内容 =====
contextBridge.exposeInMainWorld('cosurfContent', {
  /**
   * 获取页面纯文本内容
   */
  getPageText: (): string => {
    const clone = document.body.cloneNode(true) as HTMLElement;
    clone.querySelectorAll('script, style, noscript, iframe').forEach(el => el.remove());
    return clone.innerText.trim().substring(0, 15000);
  },

  /**
   * 获取页面 HTML (清理后)
   */
  getPageHtml: (): string => {
    const clone = document.body.cloneNode(true) as HTMLElement;
    clone.querySelectorAll('script, style, noscript').forEach(el => el.remove());
    return clone.innerHTML.substring(0, 50000);
  },

  /**
   * 获取页面元数据
   */
  getPageMeta: () => {
    return {
      url: window.location.href,
      title: document.title,
      description: document.querySelector('meta[name="description"]')?.getAttribute('content') || '',
      ogTitle: document.querySelector('meta[property="og:title"]')?.getAttribute('content') || '',
      ogImage: document.querySelector('meta[property="og:image"]')?.getAttribute('content') || '',
      charset: document.characterSet,
      viewport: document.querySelector('meta[name="viewport"]')?.getAttribute('content') || '',
    };
  },

  /**
   * 获取页面所有链接
   */
  getPageLinks: (): Array<{ href: string; text: string; isExternal: boolean }> => {
    const links = Array.from(document.querySelectorAll('a[href]'));
    return links.slice(0, 200).map(a => ({
      href: (a as HTMLAnchorElement).href,
      text: a.textContent?.trim().substring(0, 100) || '',
      isExternal: (a as HTMLAnchorElement).hostname !== window.location.hostname,
    }));
  },
});

// ===== 链接点击拦截 =====
// 只拦截 target="_blank" 的链接，改为在 CoSurf 内新建标签页
document.addEventListener('click', (e: Event) => {
  const target = e.target as HTMLElement;
  const link = target.closest('a') as HTMLAnchorElement | null;

  if (!link || !link.href) return;

  // 只拦截 target="_blank" 的链接
  if (link.target === '_blank') {
    e.preventDefault();
    e.stopPropagation();
    console.log('[CoSurf Content] Intercepted target=_blank link:', link.href);
    ipcRenderer.send('open-new-tab', link.href);
  }
}, true);

// ===== 覆盖 window.open =====
const originalOpen = window.open;
window.open = function (url?: string | URL, target?: string, _features?: string): WindowProxy | null {
  if (url) {
    const urlString = typeof url === 'string' ? url : url.toString();
    console.log('[CoSurf Content] Intercepted window.open:', urlString);
    ipcRenderer.send('open-new-tab', urlString);
    return null;
  }
  return originalOpen.call(window, url, target, _features);
};

// ===== 页面加载完成通知 =====
window.addEventListener('DOMContentLoaded', () => {
  ipcRenderer.send('page:dom-loaded', {
    url: window.location.href,
    title: document.title,
  });
});

console.log('[CoSurf] Content preload script injected');
