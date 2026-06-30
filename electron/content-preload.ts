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

// ===== 调试日志：确认 preload 脚本已加载 =====
console.log('[ContentPreload] ✅ Script loaded successfully!');
console.log('[ContentPreload] Current URL:', window.location.href);
console.log('[ContentPreload] Document ready state:', document.readyState);

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

// ===== 页面加载完成通知 + 内容提取 =====
window.addEventListener('DOMContentLoaded', () => {
  ipcRenderer.send('page:dom-loaded', {
    url: window.location.href,
    title: document.title,
  });
  
  // 延迟触发内容提取（等待动态内容加载）
  setTimeout(async () => {
    try {
      const article = await extractWithReadability();
      
      if (article) {
        console.log('[ContentExtractor] Successfully extracted:', {
          title: article.title,
          contentLength: article.content?.length || 0,
        });
        
        // 发送到主进程转换为 Markdown
        ipcRenderer.send('webview:content-extracted', {
          url: window.location.href,
          title: article.title,
          content: article.content,
          excerpt: article.excerpt || '',
        });
      } else {
        console.log('[ContentExtractor] No readable content found, skipping');
      }
    } catch (err) {
      console.error('[ContentExtractor] Failed to extract:', err);
    }
  }, 2000); // 等待 2 秒，确保动态内容加载完成
});

// ===== 内容选择器（增强版）=====
let isSelecting = false;
let selectionStart = { x: 0, y: 0 };
let selectionBox: HTMLDivElement | null = null;
let highlights: Array<{ element: HTMLElement; range: Range }> = [];

// 防抖定时器
let selectionTimer: NodeJS.Timeout | null = null;

/**
 * 处理文本选择事件
 */
function handleTextSelection() {
  const selection = window.getSelection();
  
  if (!selection || selection.toString().trim().length === 0) {
    return;
  }
  
  const selectedText = selection.toString().trim();
  
  // 过滤太短的选择（少于 5 个字符）
  if (selectedText.length < 5) {
    return;
  }
  
  console.log('[ContentSelector] Text selected:', selectedText.substring(0, 50) + '...');
  
  // 添加高亮
  addHighlight(selection);
  
  // 通过 IPC 发送到主进程
  ipcRenderer.send('webview:content-selected', {
    text: selectedText,
    url: window.location.href,
    title: document.title,
    timestamp: Date.now(),
    selectionType: 'text',
    highlightColor: '#ffeb3b', // 黄色高亮
  });
}

/**
 * 处理区域框选事件
 */
function handleAreaSelection(endX: number, endY: number) {
  const startX = selectionStart.x;
  const startY = selectionStart.y;
  
  const x = Math.min(startX, endX);
  const y = Math.min(startY, endY);
  const width = Math.abs(endX - startX);
  const height = Math.abs(endY - startY);
  
  // 过滤太小的区域
  if (width < 50 || height < 50) {
    return;
  }
  
  console.log(`[ContentSelector] Area selected: ${width}x${height} at (${x}, ${y})`);
  
  // 提取区域内的文本
  const elements = getElementsInArea(x, y, width, height);
  const text = extractTextFromElements(elements).trim();
  
  if (text.length < 5) {
    return;
  }
  
  // 添加区域高亮
  addAreaHighlight(x, y, width, height);
  
  // 通过 IPC 发送到主进程
  ipcRenderer.send('webview:content-selected', {
    text: text.substring(0, 5000), // 限制长度
    url: window.location.href,
    title: document.title,
    timestamp: Date.now(),
    selectionType: 'area',
    areaX: x,
    areaY: y,
    areaWidth: width,
    areaHeight: height,
    highlightColor: '#4caf50', // 绿色高亮
  });
}

/**
 * 获取区域内的元素
 */
function getElementsInArea(x: number, y: number, width: number, height: number): HTMLElement[] {
  const elements: HTMLElement[] = [];
  const allElements = document.querySelectorAll('*');
  
  allElements.forEach((el) => {
    if (!(el instanceof HTMLElement)) return;
    
    const rect = el.getBoundingClientRect();
    
    // 检查元素是否与选择区域有交集
    if (
      rect.left < x + width &&
      rect.right > x &&
      rect.top < y + height &&
      rect.bottom > y
    ) {
      elements.push(el);
    }
  });
  
  return elements;
}

/**
 * 从元素中提取文本
 */
function extractTextFromElements(elements: HTMLElement[]): string {
  // 过滤掉脚本、样式等元素
  const filtered = elements.filter(el => {
    const tag = el.tagName.toLowerCase();
    return !['script', 'style', 'noscript', 'iframe'].includes(tag);
  });
  
  // 按深度排序，优先选择深层元素
  filtered.sort((a, b) => {
    const depthA = getElementDepth(a);
    const depthB = getElementDepth(b);
    return depthB - depthA;
  });
  
  // 去重（只保留最深层的元素）
  const unique: HTMLElement[] = [];
  for (const el of filtered) {
    let isChild = false;
    for (const other of unique) {
      if (other.contains(el)) {
        isChild = true;
        break;
      }
    }
    if (!isChild) {
      unique.push(el);
    }
  }
  
  return unique.map(el => el.innerText).filter(t => t.trim()).join('\n\n');
}

/**
 * 获取元素深度
 */
function getElementDepth(el: HTMLElement): number {
  let depth = 0;
  let parent = el.parentElement;
  while (parent) {
    depth++;
    parent = parent.parentElement;
  }
  return depth;
}

/**
 * 添加文本高亮
 */
function addHighlight(selection: Selection) {
  try {
    const range = selection.getRangeAt(0);
    const span = document.createElement('span');
    span.style.backgroundColor = '#ffeb3b';
    span.style.borderRadius = '2px';
    span.style.padding = '0 2px';
    
    range.surroundContents(span);
    highlights.push({ element: span, range });
    
    console.log('[ContentSelector] Highlight added');
  } catch (err) {
    console.warn('[ContentSelector] Failed to add highlight:', err);
  }
}

/**
 * 添加区域高亮
 */
function addAreaHighlight(x: number, y: number, width: number, height: number) {
  const overlay = document.createElement('div');
  overlay.style.position = 'fixed';
  overlay.style.left = `${x}px`;
  overlay.style.top = `${y}px`;
  overlay.style.width = `${width}px`;
  overlay.style.height = `${height}px`;
  overlay.style.border = '2px solid #4caf50';
  overlay.style.backgroundColor = 'rgba(76, 175, 80, 0.2)';
  overlay.style.pointerEvents = 'none';
  overlay.style.zIndex = '99999';
  overlay.id = `highlight-${Date.now()}`;
  
  document.body.appendChild(overlay);
  
  // 5秒后自动移除
  setTimeout(() => {
    overlay.remove();
  }, 5000);
}

/**
 * 清除所有高亮
 */
function clearHighlights() {
  highlights.forEach(({ element }) => {
    const parent = element.parentNode;
    if (parent) {
      parent.replaceChild(document.createTextNode(element.textContent || ''), element);
    }
  });
  highlights = [];
  
  // 移除所有区域高亮
  const overlays = document.querySelectorAll('[id^="highlight-"]');
  overlays.forEach(el => el.remove());
}

// 监听鼠标按下事件（区域框选）
document.addEventListener('mousedown', (e) => {
  // Ctrl+鼠标按下：开始区域框选
  if (e.ctrlKey && e.button === 0) {
    e.preventDefault();
    isSelecting = true;
    selectionStart = { x: e.clientX, y: e.clientY };
    
    // 创建选择框
    selectionBox = document.createElement('div');
    selectionBox.style.position = 'fixed';
    selectionBox.style.border = '2px dashed #2196f3';
    selectionBox.style.backgroundColor = 'rgba(33, 150, 243, 0.1)';
    selectionBox.style.pointerEvents = 'none';
    selectionBox.style.zIndex = '99999';
    selectionBox.style.left = `${e.clientX}px`;
    selectionBox.style.top = `${e.clientY}px`;
    selectionBox.style.width = '0px';
    selectionBox.style.height = '0px';
    
    document.body.appendChild(selectionBox);
    
    console.log('[ContentSelector] Area selection started');
  }
});

// 监听鼠标移动事件（区域框选）
document.addEventListener('mousemove', (e) => {
  if (isSelecting && selectionBox) {
    const currentX = e.clientX;
    const currentY = e.clientY;
    
    const x = Math.min(selectionStart.x, currentX);
    const y = Math.min(selectionStart.y, currentY);
    const width = Math.abs(currentX - selectionStart.x);
    const height = Math.abs(currentY - selectionStart.y);
    
    selectionBox.style.left = `${x}px`;
    selectionBox.style.top = `${y}px`;
    selectionBox.style.width = `${width}px`;
    selectionBox.style.height = `${height}px`;
  }
});

// 监听鼠标松开事件
document.addEventListener('mouseup', (e) => {
  // 结束区域框选
  if (isSelecting && selectionBox) {
    isSelecting = false;
    
    const endX = e.clientX;
    const endY = e.clientY;
    
    // 移除选择框
    selectionBox.remove();
    selectionBox = null;
    
    // 处理区域选择
    handleAreaSelection(endX, endY);
    return;
  }
  
  // 普通文本选择（带防抖）
  if (selectionTimer) {
    clearTimeout(selectionTimer);
  }
  
  selectionTimer = setTimeout(handleTextSelection, 100);
});

// 监听键盘事件（Shift+方向键选择文本）
document.addEventListener('keyup', (e) => {
  if (e.key === 'Shift') {
    if (selectionTimer) {
      clearTimeout(selectionTimer);
    }
    selectionTimer = setTimeout(handleTextSelection, 100);
  }
});

// ESC 键清除高亮
document.addEventListener('keydown', (e) => {
  if (e.key === 'Escape') {
    clearHighlights();
    console.log('[ContentSelector] Highlights cleared');
  }
});

// ===== AI 右键菜单 =====
let contextMenuElement: HTMLDivElement | null = null;

/**
 * 显示 AI 分析右键菜单
 */
function showAIContextMenu(x: number, y: number, selectedText: string, url: string, title: string): void {
  // 移除旧菜单
  if (contextMenuElement) {
    contextMenuElement.remove();
  }
  
  // 创建菜单容器
  const menu = document.createElement('div');
  menu.style.position = 'fixed';
  menu.style.left = `${x}px`;
  menu.style.top = `${y}px`;
  menu.style.backgroundColor = '#ffffff';
  menu.style.border = '1px solid #e0e0e0';
  menu.style.borderRadius = '8px';
  menu.style.boxShadow = '0 4px 12px rgba(0, 0, 0, 0.15)';
  menu.style.padding = '8px 0';
  menu.style.minWidth = '200px';
  menu.style.zIndex = '100000';
  menu.id = 'ai-context-menu';
  
  // 菜单项配置
  const menuItems = [
    { label: '🤖 用 AI 总结', action: 'summarize' },
    { label: '🔍 用 AI 解释', action: 'explain' },
    { label: '🌐 翻译成中文', action: 'translate' },
    { label: '📝 提取关键点', action: 'extract' },
    { type: 'divider' },
    { label: '❌ 取消', action: 'cancel' },
  ];
  
  // 添加菜单项
  menuItems.forEach((item: any) => {
    if (item.type === 'divider') {
      const divider = document.createElement('div');
      divider.style.height = '1px';
      divider.style.backgroundColor = '#e0e0e0';
      divider.style.margin = '4px 0';
      menu.appendChild(divider);
    } else {
      const menuItem = document.createElement('div');
      menuItem.textContent = item.label || '';
      menuItem.style.padding = '8px 16px';
      menuItem.style.cursor = 'pointer';
      menuItem.style.fontSize = '14px';
      menuItem.style.color = '#333';
      menuItem.style.transition = 'background-color 0.2s';
      
      menuItem.addEventListener('mouseenter', () => {
        menuItem.style.backgroundColor = '#f5f5f5';
      });
      
      menuItem.addEventListener('mouseleave', () => {
        menuItem.style.backgroundColor = '#ffffff';
      });
      
      menuItem.addEventListener('click', () => {
        handleMenuItemClick(item.action, selectedText, url, title);
        menu.remove();
      });
      
      menu.appendChild(menuItem);
    }
  });
  
  document.body.appendChild(menu);
  contextMenuElement = menu;
  
  console.log('[AIContextMenu] Menu displayed');
  
  // 点击其他地方关闭菜单
  setTimeout(() => {
    document.addEventListener('click', closeContextMenu, { once: true });
  }, 100);
}

/**
 * 关闭右键菜单
 */
function closeContextMenu(): void {
  if (contextMenuElement) {
    contextMenuElement.remove();
    contextMenuElement = null;
    console.log('[AIContextMenu] Menu closed');
  }
}

/**
 * 处理菜单项点击
 */
function handleMenuItemClick(action: string, selectedText: string, url: string, title: string): void {
  console.log(`[AIContextMenu] Action: ${action}`);
  
  let prompt = '';
  
  switch (action) {
    case 'summarize':
      prompt = `请总结以下内容：\n\n${selectedText}\n\n来源：${title} (${url})`;
      break;
    case 'explain':
      prompt = `请解释以下内容：\n\n${selectedText}\n\n来源：${title} (${url})`;
      break;
    case 'translate':
      prompt = `请将以下内容翻译成中文：\n\n${selectedText}\n\n来源：${title} (${url})`;
      break;
    case 'extract':
      prompt = `请提取以下内容的重点：\n\n${selectedText}\n\n来源：${title} (${url})`;
      break;
    default:
      return;
  }
  
  // TODO: 通过 IPC 发送到主进程，打开 AI 面板并发送消息
  console.log('[AIContextMenu] Sending to AI panel:', prompt.substring(0, 100) + '...');
  
  // 通过 IPC 发送到主进程
  ipcRenderer.send('ai-panel:request', {
    action,
    prompt,
    selectedText,
    url,
    title,
  });
}

// 监听右键菜单事件（有选中内容时）
document.addEventListener('contextmenu', (e) => {
  const selection = window.getSelection();
  
  if (selection && selection.toString().trim().length > 5) {
    e.preventDefault();
    
    const selectedText = selection.toString().trim();
    
    showAIContextMenu(
      e.clientX,
      e.clientY,
      selectedText,
      window.location.href,
      document.title
    );
  }
});

// ===== 高亮持久化 =====
const HIGHLIGHTS_STORAGE_KEY = 'cosurf-highlights';

/**
 * 保存高亮到 localStorage
 */
function saveHighlights() {
  try {
    const highlightsData = highlights.map(({ range }) => ({
      startContainer: getRangePath(range.startContainer),
      startOffset: range.startOffset,
      endContainer: getRangePath(range.endContainer),
      endOffset: range.endOffset,
      color: '#ffeb3b',
    }));
    
    localStorage.setItem(HIGHLIGHTS_STORAGE_KEY, JSON.stringify(highlightsData));
    console.log('[ContentSelector] Highlights saved:', highlightsData.length);
  } catch (err) {
    console.warn('[ContentSelector] Failed to save highlights:', err);
  }
}

/**
 * 获取节点的路径（用于持久化）
 */
function getRangePath(node: Node): string {
  const path: number[] = [];
  let current: Node | null = node;
  
  while (current && current !== document.body) {
    const parent = current.parentNode as Node | null;
    if (!parent) break;
    
    let index = 0;
    let sibling = parent.firstChild;
    while (sibling && sibling !== current) {
      index++;
      sibling = sibling.nextSibling;
    }
    
    path.unshift(index);
    current = parent;
  }
  
  return path.join('/');
}

/**
 * 从 localStorage 恢复高亮
 */
function restoreHighlights() {
  try {
    const saved = localStorage.getItem(HIGHLIGHTS_STORAGE_KEY);
    if (!saved) return;
    
    const highlightsData = JSON.parse(saved);
    console.log('[ContentSelector] Restoring highlights:', highlightsData.length);
    
    // TODO: 实现复杂的路径解析和高亮恢复
    // 由于 DOM 结构可能变化，这里简化处理，仅记录日志
    // 完整实现需要 XPath 或 CSS Selector 定位
  } catch (err) {
    console.warn('[ContentSelector] Failed to restore highlights:', err);
  }
}

// ===== Readability 内容提取 =====

/**
 * 使用 Readability.js 智能提取页面主要内容
 * 
 * @returns 提取的文章对象 { title, content, excerpt } 或 null
 */
async function extractWithReadability(): Promise<{
  title: string;
  content: string;
  excerpt?: string;
} | null> {
  try {
    // 1. 动态加载 Readability.js（从 CDN）
    if (!(window as any).Readability) {
      await loadReadabilityScript();
    }
    
    // 2. 克隆文档（避免修改原页面）
    const documentClone = document.cloneNode(true) as Document;
    
    // 3. 执行 Readability 提取
    const reader = new (window as any).Readability(documentClone, {
      charThreshold: 500, // 最小字符数
      keepClasses: false,
      nbTopCandidates: 5,
    });
    
    const article = reader.parse();
    
    if (!article || !article.content) {
      return null;
    }
    
    return {
      title: article.title || document.title,
      content: article.content,
      excerpt: article.excerpt || '',
    };
  } catch (err) {
    console.error('[ReadabilityExtractor] Extraction failed:', err);
    return null;
  }
}

/**
 * 动态加载 Readability.js 脚本
 */
function loadReadabilityScript(): Promise<void> {
  return new Promise((resolve, reject) => {
    const script = document.createElement('script');
    // 使用 jsDelivr CDN
    script.src = 'https://cdn.jsdelivr.net/npm/@mozilla/readability@0.4.4/Readability.min.js';
    script.async = true;
    
    script.onload = () => {
      console.log('[ReadabilityExtractor] Script loaded successfully');
      resolve();
    };
    
    script.onerror = (err) => {
      console.error('[ReadabilityExtractor] Failed to load script:', err);
      reject(err);
    };
    
    document.head.appendChild(script);
  });
}

// 页面加载时恢复高亮
window.addEventListener('DOMContentLoaded', () => {
  setTimeout(restoreHighlights, 1000); // 延迟 1 秒，等待 DOM 稳定
});

// 每次添加高亮后保存
const originalAddHighlight = addHighlight;
function addHighlightWithSave(selection: Selection) {
  originalAddHighlight(selection);
  setTimeout(saveHighlights, 100); // 防抖保存
}
// 替换原函数
(window as any).addHighlight = addHighlightWithSave;

console.log('[CoSurf] Content selector initialized');

console.log('[CoSurf] Content preload script injected');
