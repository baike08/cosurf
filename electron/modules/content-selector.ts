/**
 * 页面内容选择器
 * 
 * 注入到每个浏览的页面中，监听用户的文本选择行为
 */

interface SelectionData {
  text: string;
  url: string;
  title: string;
  timestamp: number;
}

// 防抖定时器
let selectionTimer: NodeJS.Timeout | null = null;

/**
 * 处理文本选择事件
 */
function handleSelection() {
  const selection = window.getSelection();
  
  if (!selection || selection.toString().trim().length === 0) {
    return; // 没有选择内容
  }
  
  const selectedText = selection.toString().trim();
  
  // 过滤太短的选择（少于 5 个字符）
  if (selectedText.length < 5) {
    return;
  }
  
  const data: SelectionData = {
    text: selectedText,
    url: window.location.href,
    title: document.title,
    timestamp: Date.now(),
  };
  
  console.log('[ContentSelector] Text selected:', selectedText.substring(0, 50) + '...');
  
  // 通过 IPC 发送到主进程
  try {
    // @ts-ignore - electron API is available in preload context
    window.electronAPI.send('webview:content-selected', data);
  } catch (err) {
    console.error('[ContentSelector] Failed to send selection event:', err);
  }
}

/**
 * 初始化内容选择器
 */
export function initContentSelector(): void {
  console.log('[ContentSelector] 🚀 Initializing content selector...');
  
  // 监听鼠标松开事件（带防抖）
  document.addEventListener('mouseup', () => {
    if (selectionTimer) {
      clearTimeout(selectionTimer);
    }
    
    // 100ms 防抖，等待用户完成选择
    selectionTimer = setTimeout(handleSelection, 100);
  });
  
  // 监听键盘事件（Shift+方向键选择文本）
  document.addEventListener('keyup', (e) => {
    if (e.key === 'Shift') {
      if (selectionTimer) {
        clearTimeout(selectionTimer);
      }
      selectionTimer = setTimeout(handleSelection, 100);
    }
  });
  
  console.log('[ContentSelector] ✅ Content selector initialized');
}
