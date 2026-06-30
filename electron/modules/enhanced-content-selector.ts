/**
 * 增强版页面内容选择器
 * 
 * 支持：
 * 1. 文本选择（鼠标拖拽）
 * 2. 区域框选（Ctrl+拖拽）
 * 3. 高亮显示
 * 4. 右键菜单 AI 分析
 */

interface SelectionData {
  text: string;
  url: string;
  title: string;
  timestamp: number;
  selectionType: 'text' | 'area';
  areaX?: number;
  areaY?: number;
  areaWidth?: number;
  areaHeight?: number;
  highlightColor?: string;
}

// 状态变量
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
  
  const data: SelectionData = {
    text: selectedText,
    url: window.location.href,
    title: document.title,
    timestamp: Date.now(),
    selectionType: 'text',
    highlightColor: '#ffeb3b', // 黄色高亮
  };
  
  sendSelectionEvent(data);
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
  
  const data: SelectionData = {
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
  };
  
  sendSelectionEvent(data);
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
 * 发送选择事件到主进程
 */
function sendSelectionEvent(data: SelectionData) {
  try {
    // @ts-ignore - electron API is available in preload context
    window.electronAPI.send('webview:content-selected', data);
  } catch (err) {
    console.error('[ContentSelector] Failed to send selection event:', err);
  }
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

/**
 * 初始化内容选择器
 */
export function initEnhancedContentSelector(): void {
  console.log('[ContentSelector] 🚀 Initializing enhanced content selector...');
  
  // 监听鼠标按下事件
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
  
  // 监听右键菜单
  document.addEventListener('contextmenu', (e) => {
    const selection = window.getSelection();
    if (selection && selection.toString().trim().length > 0) {
      console.log('[ContentSelector] Right-click on selection detected');
      // TODO: 显示自定义右键菜单
    }
  });
  
  // ESC 键清除高亮
  document.addEventListener('keydown', (e) => {
    if (e.key === 'Escape') {
      clearHighlights();
      console.log('[ContentSelector] Highlights cleared');
    }
  });
  
  console.log('[ContentSelector] ✅ Enhanced content selector initialized');
}
