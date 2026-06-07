/**
 * Browser Automation Engine
 * 提供完整的浏览器自动化操作能力
 */

export interface BrowserAction {
  type: 'click' | 'input' | 'select' | 'scroll' | 'wait' | 'screenshot' | 'extract';
  selector?: string;
  value?: string;
  options?: Record<string, any>;
}

export interface ActionResult {
  success: boolean;
  message: string;
  data?: any;
}

/**
 * 生成智能选择器
 */
export function generateSmartSelector(element: HTMLElement): string {
  // 优先使用 ID
  if (element.id) {
    return `#${element.id}`;
  }

  // 使用 CSS 类
  if (element.className && typeof element.className === 'string') {
    const classes = element.className.trim().split(/\s+/).filter(c => c);
    if (classes.length > 0) {
      return `${element.tagName.toLowerCase()}.${classes.join('.')}`;
    }
  }

  // 使用属性
  const attrs = ['name', 'data-testid', 'aria-label', 'placeholder'];
  for (const attr of attrs) {
    const value = element.getAttribute(attr);
    if (value) {
      return `${element.tagName.toLowerCase()}[${attr}="${value}"]`;
    }
  }

  // 使用 XPath 风格的层级
  return getElementPath(element);
}

/**
 * 获取元素的路径
 */
function getElementPath(element: HTMLElement): string {
  const path: string[] = [];
  let current: HTMLElement | null = element;

  while (current && current !== document.body) {
    let selector = current.tagName.toLowerCase();
    
    if (current.id) {
      selector = `#${current.id}`;
      path.unshift(selector);
      break;
    } else {
      // 计算同级元素中的位置
      const siblings = Array.from(current.parentElement?.children || []);
      const index = siblings.indexOf(current);
      if (siblings.filter(s => s.tagName === current!.tagName).length > 1) {
        selector += `:nth-child(${index + 1})`;
      }
    }
    
    path.unshift(selector);
    current = current.parentElement as HTMLElement;
  }

  return path.join(' > ');
}

/**
 * 等待元素出现
 */
export async function waitForElement(
  selector: string,
  timeout: number = 5000
): Promise<HTMLElement | null> {
  return new Promise((resolve) => {
    const element = document.querySelector(selector) as HTMLElement;
    if (element) {
      resolve(element);
      return;
    }

    const observer = new MutationObserver(() => {
      const element = document.querySelector(selector) as HTMLElement;
      if (element) {
        observer.disconnect();
        resolve(element);
      }
    });

    observer.observe(document.body, {
      childList: true,
      subtree: true,
    });

    setTimeout(() => {
      observer.disconnect();
      resolve(null);
    }, timeout);
  });
}

/**
 * 点击元素(支持多种点击方式)
 */
export function clickElement(
  selector: string,
  options: { button?: 'left' | 'right' | 'middle'; double?: boolean } = {}
): ActionResult {
  try {
    const element = document.querySelector(selector) as HTMLElement;
    if (!element) {
      return { success: false, message: `Element not found: ${selector}` };
    }

    // 滚动到元素可见
    element.scrollIntoView({ behavior: 'smooth', block: 'center' });

    // 触发鼠标事件
    const eventType = options.double ? 'dblclick' : 'click';
    const event = new MouseEvent(eventType, {
      bubbles: true,
      cancelable: true,
      view: window,
      button: options.button === 'right' ? 2 : 0,
    });

    element.dispatchEvent(event);

    // 如果是链接,也触发默认的点击行为
    if (element.tagName === 'A' && (element as HTMLAnchorElement).href) {
      (element as HTMLAnchorElement).click();
    }

    return { success: true, message: `Clicked: ${selector}` };
  } catch (error) {
    return { success: false, message: `Click failed: ${error}` };
  }
}

/**
 * 输入文本(支持多种输入框类型)
 */
export function inputText(
  selector: string,
  text: string,
  options: { clear?: boolean; submit?: boolean } = { clear: true }
): ActionResult {
  try {
    const element = document.querySelector(selector) as 
      | HTMLInputElement
      | HTMLTextAreaElement
      | HTMLSelectElement;

    if (!element) {
      return { success: false, message: `Input element not found: ${selector}` };
    }

    // 聚焦元素
    element.focus();

    // 清空现有值
    if (options.clear) {
      if ('value' in element) {
        element.value = '';
      }
    }

    // 设置新值
    if ('value' in element) {
      element.value = text;
    }

    // 触发输入事件
    const events = ['input', 'change', 'keyup', 'keydown'];
    events.forEach(eventName => {
      const event = new Event(eventName, { bubbles: true });
      element.dispatchEvent(event);
    });

    // 如果需要提交表单
    if (options.submit && element.form) {
      element.form.submit();
    }

    return { success: true, message: `Input "${text}" into: ${selector}` };
  } catch (error) {
    return { success: false, message: `Input failed: ${error}` };
  }
}

/**
 * 选择下拉选项
 */
export function selectOption(
  selector: string,
  value: string | string[]
): ActionResult {
  try {
    const select = document.querySelector(selector) as HTMLSelectElement;
    if (!select) {
      return { success: false, message: `Select element not found: ${selector}` };
    }

    if (Array.isArray(value)) {
      // 多选
      Array.from(select.options).forEach(option => {
        option.selected = value.includes(option.value);
      });
    } else {
      // 单选
      select.value = value;
    }

    // 触发变化事件
    const event = new Event('change', { bubbles: true });
    select.dispatchEvent(event);

    return { success: true, message: `Selected ${value} in: ${selector}` };
  } catch (error) {
    return { success: false, message: `Select failed: ${error}` };
  }
}

/**
 * 滚动页面
 */
export function scrollPage(
  direction: 'up' | 'down' | 'left' | 'right' | 'top' | 'bottom',
  amount?: number
): ActionResult {
  try {
    const defaultAmount = 300;
    const scrollAmount = amount || defaultAmount;

    switch (direction) {
      case 'up':
        window.scrollBy({ top: -scrollAmount, behavior: 'smooth' });
        break;
      case 'down':
        window.scrollBy({ top: scrollAmount, behavior: 'smooth' });
        break;
      case 'left':
        window.scrollBy({ left: -scrollAmount, behavior: 'smooth' });
        break;
      case 'right':
        window.scrollBy({ left: scrollAmount, behavior: 'smooth' });
        break;
      case 'top':
        window.scrollTo({ top: 0, behavior: 'smooth' });
        break;
      case 'bottom':
        window.scrollTo({ top: document.body.scrollHeight, behavior: 'smooth' });
        break;
    }

    return { success: true, message: `Scrolled ${direction}` };
  } catch (error) {
    return { success: false, message: `Scroll failed: ${error}` };
  }
}

/**
 * 提取页面内容
 */
export function extractPageContent(options: {
  selector?: string;
  format?: 'text' | 'html' | 'markdown';
} = {}): ActionResult {
  try {
    const { selector, format = 'text' } = options;
    let content = '';

    if (selector) {
      const element = document.querySelector(selector);
      if (!element) {
        return { success: false, message: `Element not found: ${selector}` };
      }
      content = format === 'html' ? element.outerHTML : element.textContent || '';
    } else {
      // 提取整个页面
      if (format === 'html') {
        content = document.documentElement.outerHTML;
      } else if (format === 'markdown') {
        content = convertToMarkdown();
      } else {
        // 清理后的纯文本
        const clone = document.body.cloneNode(true) as HTMLElement;
        clone.querySelectorAll('script, style, noscript').forEach(el => el.remove());
        content = clone.innerText.trim();
      }
    }

    return {
      success: true,
      message: 'Content extracted successfully',
      data: { content, length: content.length },
    };
  } catch (error) {
    return { success: false, message: `Extract failed: ${error}` };
  }
}

/**
 * 转换为 Markdown 格式(简化版)
 */
function convertToMarkdown(): string {
  const elements = document.body.querySelectorAll('h1, h2, h3, p, a, img, ul, ol');
  let markdown = '';

  elements.forEach(el => {
    const tag = el.tagName.toLowerCase();
    
    switch (tag) {
      case 'h1':
        markdown += `# ${el.textContent}\n\n`;
        break;
      case 'h2':
        markdown += `## ${el.textContent}\n\n`;
        break;
      case 'h3':
        markdown += `### ${el.textContent}\n\n`;
        break;
      case 'p':
        markdown += `${el.textContent}\n\n`;
        break;
      case 'a':
        const href = (el as HTMLAnchorElement).href;
        markdown += `[${el.textContent}](${href})\n`;
        break;
      case 'img':
        const src = (el as HTMLImageElement).src;
        const alt = (el as HTMLImageElement).alt || 'image';
        markdown += `![${alt}](${src})\n`;
        break;
    }
  });

  return markdown;
}

/**
 * 获取表单字段列表
 */
export function getFormFields(formSelector?: string): ActionResult {
  try {
    const form = formSelector
      ? (document.querySelector(formSelector) as HTMLFormElement)
      : document.querySelector('form');

    if (!form) {
      return { success: false, message: 'No form found' };
    }

    const fields = Array.from(form.elements).map(el => {
      const element = el as HTMLInputElement | HTMLSelectElement | HTMLTextAreaElement;
      return {
        name: element.name || '',
        type: element.type || element.tagName.toLowerCase(),
        placeholder: 'placeholder' in element ? (element as HTMLInputElement).placeholder || '' : '',
        required: element.hasAttribute('required'),
        value: element.value || '',
        selector: generateSmartSelector(element),
      };
    });

    return {
      success: true,
      message: `Found ${fields.length} form fields`,
      data: { form: form.action, fields },
    };
  } catch (error) {
    return { success: false, message: `Get form fields failed: ${error}` };
  }
}

/**
 * 自动填充表单
 */
export function autoFillForm(
  formData: Record<string, string>,
  formSelector?: string
): ActionResult {
  try {
    const form = formSelector
      ? (document.querySelector(formSelector) as HTMLFormElement)
      : document.querySelector('form');

    if (!form) {
      return { success: false, message: 'No form found' };
    }

    const results: string[] = [];

    Object.entries(formData).forEach(([fieldName, value]) => {
      // 尝试通过 name、id 或 placeholder 查找字段
      const element = form.querySelector(`[name="${fieldName}"], #${fieldName}, [placeholder*="${fieldName}"]`) as
        | HTMLInputElement
        | HTMLTextAreaElement;

      if (element) {
        element.focus();
        element.value = value;
        element.dispatchEvent(new Event('input', { bubbles: true }));
        element.dispatchEvent(new Event('change', { bubbles: true }));
        results.push(`Filled ${fieldName}`);
      }
    });

    return {
      success: true,
      message: `Auto-filled ${results.length} fields`,
      data: { filled: results },
    };
  } catch (error) {
    return { success: false, message: `Auto-fill failed: ${error}` };
  }
}

/**
 * 提交表单
 */
export function submitForm(formSelector?: string): ActionResult {
  try {
    const form = formSelector
      ? (document.querySelector(formSelector) as HTMLFormElement)
      : document.querySelector('form');

    if (!form) {
      return { success: false, message: 'No form found' };
    }

    // 触发验证
    if (!form.checkValidity()) {
      form.reportValidity();
      return { success: false, message: 'Form validation failed' };
    }

    // 提交表单
    form.submit();

    return { success: true, message: 'Form submitted' };
  } catch (error) {
    return { success: false, message: `Submit failed: ${error}` };
  }
}

/**
 * 执行自定义 JavaScript
 */
export function executeScript(script: string): ActionResult {
  try {
    // 使用 Function 构造器执行脚本,可以返回值
    const result = new Function(script)();
    return {
      success: true,
      message: 'Script executed successfully',
      data: { result },
    };
  } catch (error) {
    return { success: false, message: `Script execution failed: ${error}` };
  }
}

/**
 * 高亮元素(用于可视化调试)
 */
export function highlightElement(selector: string, duration: number = 2000): ActionResult {
  try {
    const element = document.querySelector(selector) as HTMLElement;
    if (!element) {
      return { success: false, message: `Element not found: ${selector}` };
    }

    // 添加高亮样式
    const originalOutline = element.style.outline;
    element.style.outline = '3px solid #ff0000';
    element.style.outlineOffset = '2px';
    element.style.transition = 'outline 0.3s';

    // 创建提示标签
    const tooltip = document.createElement('div');
    tooltip.textContent = selector;
    tooltip.style.cssText = `
      position: fixed;
      background: #ff0000;
      color: white;
      padding: 4px 8px;
      border-radius: 4px;
      font-size: 12px;
      z-index: 999999;
      pointer-events: none;
    `;

    const rect = element.getBoundingClientRect();
    tooltip.style.left = `${rect.left}px`;
    tooltip.style.top = `${rect.top - 30}px`;
    document.body.appendChild(tooltip);

    // 移除高亮
    setTimeout(() => {
      element.style.outline = originalOutline;
      tooltip.remove();
    }, duration);

    return { success: true, message: `Highlighted: ${selector}` };
  } catch (error) {
    return { success: false, message: `Highlight failed: ${error}` };
  }
}
