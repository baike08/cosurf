/**
 * CoSurf 网页内容提取器（简化版）
 * 
 * 由于 React 渲染的 <webview> 不支持 session.setPreloads()，
 * 我们采用以下方案：
 * 1. 监听页面加载完成事件
 * 2. 通过 IPC 通知前端执行提取脚本
 * 3. 前端执行 Readability 提取并返回结果
 * 4. 主进程接收结果并转换为 Markdown
 */

import { ipcMain } from 'electron';
import * as path from 'path';
import * as fs from 'fs';
import * as os from 'os';
import TurndownService from 'turndown';

// Native 模块（延迟加载）
let native: any = null;

function getNative(): any {
  if (!native) {
    try {
      native = require(path.join(__dirname, '../../native/cosurf-native.node'));
    } catch {
      console.warn('[PageExtractor] Native module not available');
      native = null;
    }
  }
  return native;
}

// 临时数据目录
const TEMP_DIR = path.join(os.tmpdir(), 'cosurf-page-cache');

// 确保临时目录存在
if (!fs.existsSync(TEMP_DIR)) {
  fs.mkdirSync(TEMP_DIR, { recursive: true });
}

interface PageExtractTask {
  url: string;
  tabId: string;
  timestamp: number;
  article?: {
    title: string;
    content: string;
    excerpt?: string;
  };
}

// 任务队列
const extractQueue: PageExtractTask[] = [];
let isProcessing = false;

// Turndown 服务实例（用于 HTML 转 Markdown）
const turndownService = new TurndownService({
  headingStyle: 'atx',       // 使用 # 标题
  codeBlockStyle: 'fenced',  // 使用 ``` 代码块
  bulletListMarker: '-',     // 列表标记
  emDelimiter: '*',          // 斜体标记
  strongDelimiter: '**',     // 粗体标记
});

// 自定义规则：移除不需要的元素
turndownService.remove(['script', 'style', 'noscript', 'iframe', 'nav', 'footer', 'header']);

/**
 * 生成唯一的文件路径
 */
function generateFilePath(url: string): string {
  // 将 URL 转换为安全的文件名
  const hash = Buffer.from(url).toString('base64').replace(/[^a-zA-Z0-9]/g, '_');
  const timestamp = Date.now();
  return path.join(TEMP_DIR, `${timestamp}_${hash}.md`);
}

/**
 * 清理过期的缓存文件（保留 3 天）
 */
function cleanupOldCache(): void {
  try {
    const now = Date.now();
    const threeDaysMs = 3 * 24 * 60 * 60 * 1000;
    
    const files = fs.readdirSync(TEMP_DIR);
    let deletedCount = 0;
    
    for (const file of files) {
      if (!file.endsWith('.md')) continue;
      
      const filePath = path.join(TEMP_DIR, file);
      const stats = fs.statSync(filePath);
      
      if (now - stats.mtimeMs > threeDaysMs) {
        fs.unlinkSync(filePath);
        deletedCount++;
      }
    }
    
    if (deletedCount > 0) {
      console.log(`[PageExtractor] 🧹 Cleaned up ${deletedCount} old cache files`);
    }
  } catch (err) {
    console.error('[PageExtractor] Failed to cleanup old cache:', err);
  }
}

/**
 * 每小时执行一次清理
 */
setInterval(cleanupOldCache, 60 * 60 * 1000);

/**
 * 将 Readability 提取的 HTML 转换为 Markdown
 * 
 * 使用 Turndown 进行高质量转换，保留结构（标题、列表、表格、代码等）
 */
function convertToMarkdown(article: {
  title: string;
  content: string;
  excerpt?: string;
}, url: string): string {
  try {
    let markdown = '';
    
    // 添加标题
    if (article.title) {
      markdown += `# ${article.title}\n\n`;
    }
    
    // 添加摘要
    if (article.excerpt) {
      markdown += `> ${article.excerpt}\n\n`;
    }
    
    // 添加元信息
    markdown += `**URL**: ${url}\n\n`;
    markdown += `**Extracted at**: ${new Date().toISOString()}\n\n`;
    markdown += `---\n\n`;
    
    // 使用 Turndown 转换主要内容
    const contentMarkdown = turndownService.turndown(article.content);
    markdown += contentMarkdown;
    
    // 限制长度（最多 50KB）
    if (markdown.length > 50000) {
      markdown = markdown.substring(0, 50000) + '\n\n... (content truncated)';
    }
    
    return markdown;
  } catch (err) {
    console.error('[PageExtractor] Failed to convert to markdown:', err);
    return `# Error\n\nFailed to convert content from ${url}`;
  }
}

/**
 * 处理单个提取任务
 */
async function processExtractTask(task: PageExtractTask): Promise<void> {
  const { url, tabId, timestamp, article } = task;
  
  console.log(`[PageExtractor] 📄 Extracting content: ${url}`);
  
  try {
    // 如果没有提供 article（从 Readability 提取），则跳过
    if (!article) {
      console.warn(`[PageExtractor] ⚠️  No article content provided for: ${url}`);
      return;
    }
    
    // 使用 Turndown 转换为 Markdown
    const markdown = convertToMarkdown(article, url);
    
    // 保存文件
    const filePath = generateFilePath(url);
    fs.writeFileSync(filePath, markdown, 'utf-8');
    
    console.log(`[PageExtractor] ✅ Content saved to: ${filePath}`);
    console.log(`[PageExtractor] 📊 Markdown length: ${markdown.length} chars`);
    
    // 记录到数据库
    const nat = getNative();
    if (nat) {
      const eventJson = JSON.stringify({
        id: `page-extract-${timestamp}-${Math.random().toString(36).substr(2, 9)}`,
        type: 'page_extract',
        timestamp: timestamp,
        url: url,
        tab_id: tabId,
        data: {
          file_path: filePath,
          file_size: fs.statSync(filePath).size,
          title: article.title,
          excerpt: article.excerpt || '',
          extracted_at: new Date().toISOString(),
        },
        created_at: timestamp,
      });
      
      nat.dbInsertUserEvent(eventJson);
      console.log(`[PageExtractor] 📊 Event recorded in database`);
    }
    
  } catch (err) {
    console.error(`[PageExtractor] ❌ Failed to extract content:`, err);
  }
}

/**
 * 处理任务队列
 */
async function processQueue(): Promise<void> {
  if (isProcessing || extractQueue.length === 0) {
    return;
  }
  
  isProcessing = true;
  
  while (extractQueue.length > 0) {
    const task = extractQueue.shift();
    if (task) {
      await processExtractTask(task);
      // 每个任务之间延迟 100ms，避免阻塞
      await new Promise(resolve => setTimeout(resolve, 100));
    }
  }
  
  isProcessing = false;
}

/**
 * 添加提取任务到队列
 */
export function addExtractTask(
  url: string,
  tabId: string,
  article?: {
    title: string;
    content: string;
    excerpt?: string;
  }
): void {
  // 过滤无效 URL
  if (!url || url.startsWith('about:') || url.startsWith('chrome://')) {
    return;
  }
  
  const task: PageExtractTask = {
    url,
    tabId,
    timestamp: Date.now(),
    article,
  };
  
  extractQueue.push(task);
  console.log(`[PageExtractor] 📥 Task added to queue (queue size: ${extractQueue.length})`);
  
  // 异步处理队列
  processQueue();
}

/**
 * 初始化网页内容提取器
 */
export function initPageExtractor(mainWindow: any): void {
  console.log('[PageExtractor] 🚀 Initializing page extractor...');
  
  // 监听来自前端的提取结果
  ipcMain.on('webview:content-extracted', (_event, data) => {
    const { url, tabId, title, content, excerpt } = data;
    console.log(`[PageExtractor] Received extracted content from frontend:`, {
      url,
      tabId,
      title,
      contentLength: content?.length || 0,
    });
    
    // 异步添加提取任务
    addExtractTask(url, tabId, {
      title,
      content,
      excerpt,
    });
  });
  
  // 初始清理
  cleanupOldCache();
  
  console.log('[PageExtractor] ✅ Page extractor initialized');
}

/**
 * 获取缓存文件路径
 */
export function getCachedFilePath(url: string): string | null {
  try {
    const files = fs.readdirSync(TEMP_DIR);
    
    // 查找匹配的缓存文件
    for (const file of files) {
      if (!file.endsWith('.md')) continue;
      
      const filePath = path.join(TEMP_DIR, file);
      const content = fs.readFileSync(filePath, 'utf-8');
      
      if (content.includes(url)) {
        return filePath;
      }
    }
    
    return null;
  } catch (err) {
    console.error('[PageExtractor] Failed to get cached file path:', err);
    return null;
  }
}
