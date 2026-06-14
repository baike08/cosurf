/**
 * CoSurf Electron 主进程入口
 * 
 * 架构: Electron Main Process
 * 职责:
 *   1. 创建主窗口 (BrowserWindow)
 *   2. 管理多标签页 (TabManager / WebContentsView)
 *   3. 注册 IPC 处理器 (桥接前端 <-> Native 模块)
 *   4. 全局快捷键注册
 *   5. 应用生命周期管理
 */

import { app, BrowserWindow, globalShortcut, protocol, session } from 'electron';
import path from 'path';
import { TabManager } from './window-manager';
import { registerIpcHandlers } from './ipc-handlers';
import { setupNetworkInterception } from './network-interceptor';

// ===== 全局变量 =====
let mainWindow: BrowserWindow | null = null;
let tabManager: TabManager | null = null;

// ===== 开发/生产环境检测 =====
const isDev = !app.isPackaged;

// ===== 应用数据目录 =====
function getAppDataDir(): string {
  return path.join(app.getPath('userData'), 'cosurf-data');
}

// ===== 创建主窗口 =====
async function createMainWindow(): Promise<BrowserWindow> {
  const win = new BrowserWindow({
    width: 1400,
    height: 900,
    minWidth: 1024,
    minHeight: 680,
    frame: false,          // 无边框窗口 (与 Tauri decorations:false 一致)
    titleBarStyle: 'hidden',
    title: 'CoSurf',
    show: false,
    webPreferences: {
      preload: path.join(__dirname, '../preload/preload.js'),
      contextIsolation: true,
      nodeIntegration: false,
      sandbox: false,       // 允许 preload 中 require
      webviewTag: true,     // 启用 <webview> tag
      webSecurity: false,   // 禁用所有 Web 安全策略（包括 CSP）
    },
  });

  // 窗口准备好后再显示，防止闪烁
  win.once('ready-to-show', () => {
    win.show();
    win.focus();
  });

  // 拦截所有 webContents 的响应头，移除 CSP 和 X-Frame-Options
  win.webContents.session.webRequest.onHeadersReceived(
    { urls: ['*://*/*'] },
    (details, callback) => {
      const headers: Record<string, string | string[]> = details.responseHeaders || {};
      
      // 移除所有 CSP 相关头（不区分大小写）
      Object.keys(headers).forEach(key => {
        const lowerKey = key.toLowerCase();
        if (lowerKey === 'content-security-policy' || 
            lowerKey === 'x-frame-options' ||
            lowerKey === 'x-content-security-policy') {
          delete headers[key];
        }
      });
      
      callback({ responseHeaders: headers });
    }
  );

  console.log('[CoSurf] CSP/X-Frame-Options interception enabled');

  // 加载前端
  if (isDev && process.env.ELECTRON_RENDERER_URL) {
    // 开发模式: 加载 Vite dev server
    await win.loadURL(process.env.ELECTRON_RENDERER_URL);
    win.webContents.openDevTools({ mode: 'detach' });
  } else {
    // 生产模式: 加载打包后的 HTML
    await win.loadFile(path.join(__dirname, '../renderer/index.html'));
  }

  return win;
}

// ===== 初始化原生模块 =====
function initNativeModule(): void {
  try {
    // 尝试加载编译好的 .node 原生模块
    const nativeModulePath = path.join(__dirname, '../../native/cosurf-native.node');
    const nativeModule = require(nativeModulePath);

    // 调用 native_init (Rust #[napi] pub fn native_init -> JS: nativeInit)
    const appDataDir = getAppDataDir();
    
    // 不传递 skills_dir 参数，让 native 模块自动从数据库读取配置
    if (typeof nativeModule.nativeInit === 'function') {
      nativeModule.nativeInit(appDataDir, null); // 第二个参数为 null，让 Rust 自动从数据库读取
      console.log('[CoSurf] Native module initialized, data dir:', appDataDir);
    } else {
      console.warn('[CoSurf] nativeInit not found in native module, available:', Object.keys(nativeModule));
    }
  } catch (err) {
    console.warn('[CoSurf] Native module not available, running in UI-only mode:', err);
  }
}

// ===== 配置 Webview Session（移除 CSP/X-Frame-Options）=====
function configureWebviewSession(): void {
  const webviewSession = session.fromPartition('persist:cosurf-webview');
  
  // 设置 preload 脚本
  const preloadPath = path.join(__dirname, '../preload/content-preload.js');
  webviewSession.setPreloads([preloadPath]);
  console.log('[CoSurf] Webview preload configured:', preloadPath);
  
  webviewSession.webRequest.onHeadersReceived(
    { urls: ['*://*/*'] },
    (details, callback) => {
      const headers: Record<string, string | string[]> = details.responseHeaders || {};
      
      // 移除所有 CSP 相关头（不区分大小写）
      Object.keys(headers).forEach(key => {
        const lowerKey = key.toLowerCase();
        if (lowerKey === 'content-security-policy' || 
            lowerKey === 'x-frame-options' ||
            lowerKey === 'x-content-security-policy') {
          delete headers[key];
        }
      });
      
      callback({ responseHeaders: headers });
    }
  );
  
  console.log('[CoSurf] Webview session CSP interception enabled');
}

// ===== 注册全局快捷键 =====
function registerGlobalShortcuts(): void {
  // Ctrl+Shift+X: 全屏截图
  const screenshotShortcut = process.platform === 'darwin' ? 'Command+Shift+X' : 'Control+Shift+X';
  globalShortcut.register(screenshotShortcut, () => {
    if (mainWindow) {
      mainWindow.webContents.send('shortcut:screenshot');
    }
  });

  console.log('[CoSurf] Global shortcuts registered');
}

// ===== 自定义协议 (cosurf://) — 必须在 app.whenReady() 之前调用 =====
function registerCustomProtocol(): void {
  protocol.registerSchemesAsPrivileged([
    {
      scheme: 'cosurf',
      privileges: {
        standard: true,
        secure: true,
        supportFetchAPI: true,
        corsEnabled: true,
      },
    },
  ]);
}

// 协议注册必须在 app ready 之前
registerCustomProtocol();

// ===== 应用启动 =====
app.whenReady().then(async () => {
  console.log('[CoSurf] === Application Starting ===');
  console.log('[CoSurf] App data directory:', getAppDataDir());
  console.log('[CoSurf] Dev mode:', isDev);

  // 1. 初始化 Native 模块 (数据库)
  initNativeModule();

  // 2. 配置 Webview Session（移除 CSP/X-Frame-Options）
  configureWebviewSession();

  // 3. 设置网络请求拦截
  setupNetworkInterception();

  // 3. 创建主窗口
  mainWindow = await createMainWindow();

  // 4. 创建标签页管理器
  tabManager = new TabManager(mainWindow);

  // 5. 注册 IPC 处理器
  registerIpcHandlers(tabManager, mainWindow);

  // 6. 注册全局快捷键
  registerGlobalShortcuts();

  // 7. 创建一个默认标签页
  tabManager.createTab('tab-initial', 'about:blank', '新标签页');

  console.log('[CoSurf] === Application Started ===');
});

// ===== 应用退出 =====
app.on('window-all-closed', () => {
  // macOS 惯例: 关闭所有窗口后不退出应用
  if (process.platform !== 'darwin') {
    app.quit();
  }
});

app.on('activate', async () => {
  // macOS: 点击 Dock 图标时重新创建窗口
  if (BrowserWindow.getAllWindows().length === 0) {
    mainWindow = await createMainWindow();
    tabManager = new TabManager(mainWindow);
    registerIpcHandlers(tabManager, mainWindow);
    tabManager.createTab('tab-initial', 'about:blank', '新标签页');
  }
});

app.on('will-quit', () => {
  // 注销所有全局快捷键
  globalShortcut.unregisterAll();
});
