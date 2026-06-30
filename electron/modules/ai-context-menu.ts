/**
 * AI 分析右键菜单
 * 
 * 当用户选中内容后右键点击时，显示自定义菜单
 */

interface ContextMenuOptions {
  x: number;
  y: number;
  selectedText: string;
  url: string;
  title: string;
}

let contextMenuElement: HTMLDivElement | null = null;

/**
 * 显示 AI 分析右键菜单
 */
export function showAIContextMenu(options: ContextMenuOptions): void {
  const { x, y, selectedText, url, title } = options;
  
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
  menuItems.forEach((item) => {
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
        handleMenuItemClick(item.action!, selectedText, url, title);
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
  
  // 这里需要实现与 AI 面板的通信
  // 可以通过自定义事件或 IPC 实现
  const event = new CustomEvent('ai-panel-request', {
    detail: {
      action,
      prompt,
      selectedText,
      url,
      title,
    },
  });
  
  window.dispatchEvent(event);
}

/**
 * 初始化 AI 右键菜单
 */
export function initAIContextMenu(): void {
  console.log('[AIContextMenu] 🚀 Initializing AI context menu...');
  
  // 监听右键菜单事件
  document.addEventListener('contextmenu', (e) => {
    const selection = window.getSelection();
    
    if (selection && selection.toString().trim().length > 5) {
      e.preventDefault();
      
      const selectedText = selection.toString().trim();
      
      showAIContextMenu({
        x: e.clientX,
        y: e.clientY,
        selectedText,
        url: window.location.href,
        title: document.title,
      });
    }
  });
  
  console.log('[AIContextMenu] ✅ AI context menu initialized');
}
