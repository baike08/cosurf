/**
 * CoSurf 网页内容提取器
 * 
 * 负责：
 * 1. 拦截网页加载
 * 2. 提取内容并转换为 Markdown
 * 3. 保存到临时目录
 * 4. 记录元信息到 SQLite
 */

import { ipcMain, WebContentsView, session, BrowserWindow } from 'electron';
import * as path from 'path';
import * as fs from 'fs';
import * as os from 'os';
import TurndownService from 'turndown';
import { createHash } from 'crypto';

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

// ===== 优化策略 =====

// 频率限制：记录每个域名的提取次数
const extractCountByDomain = new Map<string, { count: number; resetTime: number }>();

// 内容去重：记录已提取的内容哈希
const extractedHashes = new Set<string>();

/**
 * 检查是否应该提取该页面
 */
function shouldExtract(url: string, title: string): boolean {
  // 1. 跳过无效 URL
  if (!url || url.startsWith('about:') || url.startsWith('chrome://') || url.startsWith('file://')) {
    return false;
  }
  
  // 2. 跳过非文章页面
  const skipPatterns = ['/login', '/signin', '/signup', '/search', '/cart', '/checkout'];
  if (skipPatterns.some(pattern => url.includes(pattern))) {
    return false;
  }
  
  // 3. 只提取包含文章特征的页面
  const articleKeywords = ['blog', 'article', 'post', 'news', 'docs', 'guide', 'tutorial', 'learn'];
  const hasArticleKeyword = articleKeywords.some(keyword => 
    url.toLowerCase().includes(keyword) || title.toLowerCase().includes(keyword)
  );
  
  // 如果没有文章关键词，但有较长的标题，也尝试提取
  if (!hasArticleKeyword && title.length < 10) {
    return false;
  }
  
  return true;
}

/**
 * 检查频率限制（同域名每分钟最多 3 次）
 */
function canExtractByRateLimit(url: string): boolean {
  try {
    const domain = new URL(url).hostname;
    const now = Date.now();
    const windowMs = 60 * 1000; // 1 分钟
    const maxCount = 3;
    
    const record = extractCountByDomain.get(domain);
    
    if (!record || now > record.resetTime) {
      // 新窗口或窗口已过期
      extractCountByDomain.set(domain, { count: 1, resetTime: now + windowMs });
      return true;
    }
    
    if (record.count >= maxCount) {
      console.log(`[PageExtractor] ⏱️  Rate limit exceeded for ${domain}`);
      return false;
    }
    
    record.count++;
    return true;
  } catch (err) {
    return true; // URL 解析失败时允许提取
  }
}

/**
 * 计算内容哈希（用于去重）
 */
function getContentHash(content: string): string {
  return createHash('md5').update(content.substring(0, 1000)).digest('hex');
}

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
    
    // 内容去重检查
    const contentHash = getContentHash(article.content);
    if (extractedHashes.has(contentHash)) {
      console.log(`[PageExtractor] 🔄 Content already extracted (hash: ${contentHash.substring(0, 8)}), skipping`);
      return;
    }
    extractedHashes.add(contentHash);
    
    // 限制已提取内容的缓存大小（最多保留 100 个）
    if (extractedHashes.size > 100) {
      const firstHash = Array.from(extractedHashes)[0];
      extractedHashes.delete(firstHash);
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
export function initPageExtractor(mainWindow: BrowserWindow): void {
  console.log('[PageExtractor] 🚀 Initializing page extractor...');
  
  // 监听 webContentsView 的 did-finish-load 事件（主动注入）
  // 注意：由于 React 渲染的 <webview> 无法直接获取 WebContents，
  // 我们采用折中方案：通过前端发送页面加载完成事件
  ipcMain.on('webview:did-finish-load', (_event, { url, title, tabId }) => {
    console.log(`[PageExtractor] 📥 Page finished loading: ${url}`);
    
    // 1. 智能触发条件检查
    if (!shouldExtract(url, title)) {
      console.log(`[PageExtractor] ⏭️  Skipping non-article page: ${url}`);
      return;
    }
    
    // 2. 频率限制检查
    if (!canExtractByRateLimit(url)) {
      console.log(`[PageExtractor] ⏱️  Rate limit exceeded, skipping: ${url}`);
      return;
    }
    
    // 3. 延迟执行，等待动态内容加载
    setTimeout(() => {
      // 4. 通过 IPC 通知前端执行 Readability 提取
      mainWindow.webContents.send('webview:extract-content', { tabId, url });
      console.log(`[PageExtractor] 📤 Sent extract request to frontend`);
    }, 2000); // 等待 2 秒
  });
  
  // 监听前端返回的提取结果
  ipcMain.on('webview:content-extracted', (_event, data) => {
    const { url, tabId, title, content, excerpt } = data;
    console.log(`[PageExtractor] ✅ Received extracted content from frontend:`, {
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

/**
 * 读取缓存的 Markdown 内容
 */
export function readCachedMarkdown(filePath: string): string | null {
  try {
    if (!fs.existsSync(filePath)) {
      return null;
    }
    
    return fs.readFileSync(filePath, 'utf-8');
  } catch (err) {
    console.error('[PageExtractor] Failed to read cached markdown:', err);
    return null;
  }
}
