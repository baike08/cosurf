/**
 * CoSurf IPC 处理器注册
 * 
 * 替代 Tauri 的 invoke_handler，所有前端 <-> 后端通信通过 Electron IPC 实现。
 * 桥接前端 React 应用和 Rust Native 模块。
 */

import { ipcMain, BrowserWindow, dialog, shell } from 'electron';
import { TabManager } from './window-manager';

// Native 模块类型（延迟加载）
let native: any = null;

function getNative(): any {
  if (!native) {
    try {
      const path = require('path');
      native = require(path.join(__dirname, '../../native/cosurf-native.node'));
    } catch {
      console.warn('[IPC] Native module not available');
      native = null;
    }
  }
  return native;
}
/**
 * 安全调用 Native 方法
 */
function nativeCall<T>(method: string, ...args: any[]): T {
  const nat = getNative();
  if (!nat || typeof nat[method] !== 'function') {
    throw new Error(`Native method '${method}' not available`);
  }
  return nat[method](...args);
}

/**
 * 将 snake_case 转为 camelCase (napi-rs 导出规范)
 * 例: 'list_bookmarks' → 'listBookmarks'
 */
function toCamelCase(str: string): string {
  return str.replace(/_([a-z])/g, (_, c) => c.toUpperCase());
}

/**
 * 注册所有 IPC 处理器
 */
export function registerIpcHandlers(tabManager: TabManager, mainWindow: BrowserWindow): void {

  // ===================================================
  // 窗口控制
  // ===================================================
  ipcMain.handle('window:minimize', () => {
    mainWindow.minimize();
  });

  ipcMain.handle('window:maximize', () => {
    if (mainWindow.isMaximized()) {
      mainWindow.unmaximize();
    } else {
      mainWindow.maximize();
    }
  });

  ipcMain.handle('window:close', () => {
    mainWindow.close();
  });

  ipcMain.handle('window:is_maximized', () => {
    return mainWindow.isMaximized();
  });

  // ===================================================
  // 标签页管理
  // ===================================================
  ipcMain.handle('tab:create', (_event, url: string, title: string) => {
    const id = `tab-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
    const info = tabManager.createTab(id, url || 'about:blank', title || '新标签页');
    return { id, ...info };
  });

  ipcMain.handle('tab:switch', (_event, id: string) => {
    tabManager.switchTab(id);
  });

  ipcMain.handle('tab:close', (_event, id: string) => {
    tabManager.closeTab(id);
  });

  ipcMain.handle('tab:navigate', (_event, id: string, url: string) => {
    tabManager.navigate(id, url);
  });

  ipcMain.handle('tab:back', (_event, id: string) => {
    return tabManager.goBack(id);
  });

  ipcMain.handle('tab:forward', (_event, id: string) => {
    return tabManager.goForward(id);
  });

  ipcMain.handle('tab:get_state', (_event, id: string) => {
    const info = tabManager.getTabInfo(id);
    if (!info) return null;
    return {
      ...info,
      canGoBack: false,
      canGoForward: false,
    };
  });

  ipcMain.handle('tab:get_title', (_event, id: string) => {
    return tabManager.getTabInfo(id)?.title ?? '';
  });

  ipcMain.handle('tab:set_active', (_event, id: string) => {
    tabManager.switchTab(id);
  });

  // ===================================================
  // 页面操作
  // ===================================================
  ipcMain.handle('page:get_content', async (_event, tabId: string) => {
    return tabManager.executeJavaScript(tabId, `
      (function() {
        const clone = document.body.cloneNode(true);
        clone.querySelectorAll('script, style, noscript').forEach(el => el.remove());
        return clone.innerText.trim();
      })()
    `);
  });

  ipcMain.handle('page:screenshot', async (_event, tabId: string) => {
    return tabManager.capturePage(tabId);
  });

  ipcMain.handle('page:execute_script', async (_event, tabId: string, script: string) => {
    return tabManager.executeJavaScript(tabId, script);
  });

  ipcMain.handle('page:inject_context', async (_event, tabId: string) => {
    // 获取页面上下文用于 AI 分析
    return tabManager.executeJavaScript(tabId, `
      (function() {
        return JSON.stringify({
          url: window.location.href,
          title: document.title,
          text: document.body.innerText.substring(0, 10000),
          html_length: document.body.innerHTML.length,
        });
      })()
    `);
  });

  ipcMain.handle('page:summarize', async (_event, tabId: string) => {
    const content = await tabManager.executeJavaScript(tabId, `
      document.body.innerText.substring(0, 15000)
    `);
    return content;
  });

  ipcMain.handle('page:execute_action', async (_event, tabId: string, action: string, selector: string, value?: string) => {
    const script = `
      (function() {
        const el = document.querySelector(${JSON.stringify(selector)});
        if (!el) return JSON.stringify({ success: false, message: 'Element not found' });
        
        switch (${JSON.stringify(action)}) {
          case 'click':
            el.click();
            return JSON.stringify({ success: true, message: 'Clicked' });
          case 'fill':
            el.value = ${JSON.stringify(value || '')};
            el.dispatchEvent(new Event('input', { bubbles: true }));
            el.dispatchEvent(new Event('change', { bubbles: true }));
            return JSON.stringify({ success: true, message: 'Filled' });
          case 'scroll':
            el.scrollIntoView({ behavior: 'smooth', block: 'center' });
            return JSON.stringify({ success: true, message: 'Scrolled' });
          default:
            return JSON.stringify({ success: false, message: 'Unknown action' });
        }
      })()
    `;
    return tabManager.executeJavaScript(tabId, script);
  });

  // ===================================================
  // 数据库操作 — 桥接到 Native 模块
  // ===================================================
  const dbMethods = [
    'list_conversations', 'get_conversation', 'create_conversation',
    'update_conversation', 'delete_conversation', 'get_conversation_with_messages',
    'list_messages', 'get_message', 'create_message', 'update_message',
    'delete_message', 'append_message_content', 'complete_message',
    'set_message_feedback',
    'list_bookmarks', 'create_bookmark', 'delete_bookmark',
    'list_bookmark_folders', 'create_bookmark_folder', 'delete_bookmark_folder',
    'get_settings', 'get_setting', 'set_setting',
    'list_model_configs', 'get_model_config', 'get_active_model',
    'create_model_config', 'update_model_config', 'set_active_model', 'delete_model_config',
    'get_skills_directory', 'set_skills_directory',
    'get_iqs_api_key', 'set_iqs_api_key',
    'list_mcp_servers', 'get_mcp_server', 'create_mcp_server',
    'update_mcp_server', 'delete_mcp_server', 'test_mcp_server',
    'import_mcp_servers_from_json',
    'list_history', 'search_history', 'add_history', 'clear_history', 'delete_history_entry',
    'list_agent_prompts', 'get_agent_prompt', 'set_agent_prompt', 'toggle_agent_prompt',
  ];

  for (const method of dbMethods) {
    const channel = `db:${method}`;
    // napi-rs 将 Rust snake_case 转为 JS camelCase: db_list_bookmarks → dbListBookmarks
    const nativeMethod = 'db' + toCamelCase(method).charAt(0).toUpperCase() + toCamelCase(method).slice(1);
    ipcMain.handle(channel, async (_event, ...args: any[]) => {
      try {
        console.log(`[IPC] ${channel} -> calling native ${nativeMethod} with args:`, args);
        const result = nativeCall<any>(nativeMethod, ...args);
        console.log(`[IPC] ${channel} result:`, typeof result === 'string' ? result.substring(0, 50) : result);
        return result;
      } catch (err: any) {
        console.error(`[IPC] ${channel} error:`, err);
        throw new Error(err?.message || String(err));
      }
    });
  }

  // ===================================================
  // AI 对话 — 桥接到 Native 模块 (流式回调)
  // ===================================================
  ipcMain.handle('ai:send_chat', async (event, params: {
    config: string;
    messages: string;
    conversationId: string;
    messageId: string;
  }) => {
    try {
      const nat = getNative();
      if (!nat) throw new Error('Native module not available');

      const sender = event.sender;

      await nat.streamChat(
        params.config,
        params.messages,
        params.conversationId,
        params.messageId,
        // 流式 chunk 回调
        (chunk: any) => {
          console.log('[IPC] streamChat onChunk called:', JSON.stringify(chunk).substring(0, 100));
          if (!sender.isDestroyed()) {
            sender.send('ai:stream-chunk', chunk);
          } else {
            console.warn('[IPC] sender destroyed, dropping chunk');
          }
        },
        // 工具调用回调
        (toolCall: any) => {
          console.log('[IPC] streamChat onToolCall called:', toolCall?.tool_name);
          if (!sender.isDestroyed()) {
            sender.send('ai:tool-call-start', toolCall);
          }
        },
        // 工具结果回调
        (toolResult: any) => {
          console.log('[IPC] streamChat onToolResult called:', toolResult?.tool_name);
          if (!sender.isDestroyed()) {
            sender.send('ai:tool-call-result', toolResult);
          }
        },
        // Electron 桥接回调（需要操作浏览器界面的工具）
        async (bridgeEvent: any) => {
          console.log('[IPC] streamChat onElectronBridge called:', bridgeEvent);
          try {
            // 兼容多种字段名和格式
            let toolCall = null;
            
            if (bridgeEvent.toolCall) {
              // napi-rs 可能已经将 toolCallJson 字符串解析为 toolCall 对象
              toolCall = bridgeEvent.toolCall;
            } else if (typeof bridgeEvent.toolCallJson === 'string') {
              // 如果是字符串，需要解析
              toolCall = JSON.parse(bridgeEvent.toolCallJson);
            } else if (bridgeEvent.tool_call) {
              // 兼容蛇形命名
              toolCall = bridgeEvent.tool_call;
            }
            
            if (!toolCall || !toolCall.name) {
              console.error('[IPC] Invalid tool call:', toolCall);
              return;
            }
            
            console.log('[IPC] Executing bridge tool:', toolCall.name);
            const result = await executeElectronBridgeTool(toolCall, tabManager, sender);
            // TODO: 将结果返回给 Native 模块
            console.log('[IPC] Electron bridge tool executed:', result);
          } catch (err) {
            console.error('[IPC] Electron bridge tool failed:', err);
          }
        },
        // 错误回调
        (error: string) => {
          console.error('[IPC] streamChat onError:', error);
          if (!sender.isDestroyed()) {
            sender.send('ai:stream-error', { conversationId: params.conversationId, error });
          }
        },
      );
      console.log('[IPC] streamChat call returned (streaming in background)');
    } catch (err: any) {
      console.error('[IPC] ai:send_chat error:', err);
      throw new Error(err?.message || String(err));
    }
  });

  ipcMain.handle('ai:stop_generation', () => {
    try {
      nativeCall('aiStopGeneration');
    } catch {
      // 忽略：Native 模块可能未加载
    }
  });

  ipcMain.handle('ai:generate_title', async (_event, content: string, config?: string) => {
    try {
      return nativeCall<string>('aiGenerateTitle', content, config || '');
    } catch (err: any) {
      console.error('[IPC] ai:generate_title error:', err);
      return 'New Conversation';
    }
  });

  // ===================================================
  // Agent 操作
  // ===================================================
  ipcMain.handle('agent:execute', async (event, params: any) => {
    try {
      const nat = getNative();
      if (!nat) throw new Error('Native module not available');

      const sender = event.sender;
      await nat.agentExecute(
        JSON.stringify(params),
        (chunk: any) => {
          if (!sender.isDestroyed()) sender.send('ai:stream-chunk', chunk);
        },
        (toolCall: any) => {
          if (!sender.isDestroyed()) sender.send('ai:tool-call-start', toolCall);
        },
        (toolResult: any) => {
          if (!sender.isDestroyed()) sender.send('ai:tool-call-result', toolResult);
        },
        async (bridgeEvent: any) => {
          console.log('[IPC] agentExecute onElectronBridge called:', bridgeEvent?.tool_call?.name);
          try {
            const toolCall = bridgeEvent.tool_call;
            const result = await executeElectronBridgeTool(toolCall, tabManager, sender);
            console.log('[IPC] Electron bridge tool executed:', result);
          } catch (err) {
            console.error('[IPC] Electron bridge tool failed:', err);
          }
        },
        (error: string) => {
          if (!sender.isDestroyed()) {
            sender.send('ai:stream-error', { error });
          }
        },
      );
    } catch (err: any) {
      throw new Error(err?.message || String(err));
    }
  });

  ipcMain.handle('agent:configure_qwen', async (_event, config: any) => {
    try {
      return nativeCall('agentConfigureQwen', JSON.stringify(config));
    } catch (err: any) {
      throw new Error(err?.message || String(err));
    }
  });

  ipcMain.handle('agent:summarize_page', async (event, params: any) => {
    try {
      const nat = getNative();
      if (!nat) throw new Error('Native module not available');
      const sender = event.sender;
      await nat.agentSummarizePage(
        JSON.stringify(params),
        (chunk: any) => {
          if (!sender.isDestroyed()) sender.send('ai:stream-chunk', chunk);
        },
        (error: string) => {
          if (!sender.isDestroyed()) sender.send('ai:stream-error', { error });
        },
      );
    } catch (err: any) {
      throw new Error(err?.message || String(err));
    }
  });

  ipcMain.handle('agent:extract_memory', async (_event, params: any) => {
    try {
      return nativeCall('agentExtractMemory', JSON.stringify(params));
    } catch (err: any) {
      throw new Error(err?.message || String(err));
    }
  });

  // ===================================================
  // Skills 管理
  // ===================================================
  const skillMethods = [
    'list', 
    'delete_skill', 
    'toggle_skill', 
    'import_skill_from_markdown',
    'import_skill_from_directory', 
    'list_skill_files', 
    'get_skill_content',
  ];

  for (const method of skillMethods) {
    const channel = `skills:${method}`;
    // napi-rs: skills_list → skillsList, skills_import_skill_from_directory → skillsImportSkillFromDirectory
    const camelCase = toCamelCase(method);
    const nativeMethod = 'skills' + camelCase.charAt(0).toUpperCase() + camelCase.slice(1);
    console.log(`[IPC] Registering skills handler: ${channel} -> ${nativeMethod} (camelCase: ${camelCase})`);
    ipcMain.handle(channel, async (_event, ...args: any[]) => {
      try {
        return nativeCall<any>(nativeMethod, ...args);
      } catch (err: any) {
        throw new Error(err?.message || String(err));
      }
    });
  }

  // ===================================================
  // 截图
  // ===================================================
  ipcMain.handle('screenshot:capture_full', async () => {
    console.log('[IPC] screenshot:capture_full called');
    try {
      const result = await nativeCall<string>('screenshotCaptureFull');
      console.log('[IPC] screenshot:capture_full succeeded, result length:', result.length);
      return result;
    } catch (err: any) {
      console.error('[IPC] screenshot:capture_full failed:', err);
      throw new Error(err?.message || String(err));
    }
  });

  ipcMain.handle('screenshot:capture_region', async (_event, base64Data: string, x: number, y: number, width: number, height: number, screenWidth: number, screenHeight: number) => {
    try {
      return nativeCall<string>('screenshotCrop', base64Data, x, y, width, height, screenWidth, screenHeight);
    } catch (err: any) {
      throw new Error(err?.message || String(err));
    }
  });

  ipcMain.handle('screenshot:save', async (_event, base64Data: string, filePath: string) => {
    try {
      return nativeCall<boolean>('screenshotSave', base64Data, filePath);
    } catch (err: any) {
      throw new Error(err?.message || String(err));
    }
  });

  ipcMain.handle('screenshot:copy_to_clipboard', async (_event, base64Data: string) => {
    try {
      return nativeCall<boolean>('screenshotCopyToClipboard', base64Data);
    } catch (err: any) {
      throw new Error(err?.message || String(err));
    }
  });

  // ===================================================
  // 页面缓存
  // ===================================================
  ipcMain.handle('cache:save', async (_event, key: string, data: string) => {
    try {
      return nativeCall('cacheSave', key, data);
    } catch (err: any) {
      throw new Error(err?.message || String(err));
    }
  });

  ipcMain.handle('cache:load', async (_event, key: string) => {
    try {
      return nativeCall<string>('cacheLoad', key);
    } catch (err: any) {
      return null;
    }
  });

  ipcMain.handle('cache:cleanup', async () => {
    try {
      return nativeCall('cacheCleanup');
    } catch (err: any) {
      throw new Error(err?.message || String(err));
    }
  });

  // ===================================================
  // 对话框 (替代 Tauri dialog plugin)
  // ===================================================
  ipcMain.handle('dialog:open_file', async (_event, options: any) => {
    const result = await dialog.showOpenDialog(mainWindow, options);
    return result;
  });

  ipcMain.handle('dialog:save_file', async (_event, options: any) => {
    const result = await dialog.showSaveDialog(mainWindow, options);
    return result;
  });

  // ===================================================
  // Shell (替代 Tauri shell plugin)
  // ===================================================
  ipcMain.handle('shell:open_url', async (_event, url: string) => {
    await shell.openExternal(url);
  });

  console.log('[CoSurf] IPC handlers registered');

  // ===================================================
  // 内容预加载的链接拦截 (open-new-tab)
  // ===================================================
  ipcMain.on('open-new-tab', (_event, url: string) => {
    const id = `tab-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
    tabManager.createTab(id, url, url);
    // 通知前端新标签页已创建
    mainWindow.webContents.send('webview:create-tab', {
      requestId: id,
      url,
      title: url,
    });
  });
}

/**
 * 执行需要 Electron 主进程桥接的工具
 * 这些工具需要操作浏览器界面，无法在 Native 模块中直接执行
 */
async function executeElectronBridgeTool(
  toolCall: any,
  tabManager: TabManager,
  sender: Electron.WebContents
): Promise<any> {
  const toolName = toolCall.name;
  const args = toolCall.arguments || {};

  console.log(`[ElectronBridge] Executing tool: ${toolName}`, args);

  // 从前端获取当前活动标签页 ID
  const getActiveTabId = async (): Promise<string | null> => {
    try {
      // 向前端查询当前活动标签页
      const result = await sender.executeJavaScript(`
        (function() {
          // 从 window.__cosurf_tabStore 或者通过 IPC 获取
          // 这里简化处理，假设可以通过全局变量访问
          if (window.__cosurf_activeTabId) {
            return window.__cosurf_activeTabId;
          }
          return null;
        })()
      `);
      return result || null;
    } catch (err) {
      console.error('[ElectronBridge] Failed to get active tab ID:', err);
      return null;
    }
  };

  switch (toolName) {
    case 'open_url': {
      // 打开新的 URL
      const url = args.url;
      if (!url) {
        return { success: false, error: 'Missing url parameter' };
      }
      
      // 获取当前活动标签页 ID
      const currentTabId = await getActiveTabId();
      if (currentTabId) {
        console.log(`[ElectronBridge] Navigating tab ${currentTabId} to ${url}`);
        tabManager.navigate(currentTabId, url);
        
        // 通知前端更新标签页 URL（通过 executeJavaScript 调用前端 store）
        try {
          await sender.executeJavaScript(`
            (function() {
              if (window.__cosurf_navigateTo && window.__cosurf_updateTab) {
                window.__cosurf_navigateTo(${JSON.stringify(currentTabId)}, ${JSON.stringify(url)});
                window.__cosurf_updateTab(${JSON.stringify(currentTabId)}, { 
                  url: ${JSON.stringify(url)}, 
                  title: ${JSON.stringify(url)},
                  isLoading: true 
                });
              }
            })()
          `);
          console.log(`[ElectronBridge] Notified frontend to navigate to ${url}`);
        } catch (err) {
          console.error('[ElectronBridge] Failed to notify frontend:', err);
        }
        
        return { success: true, message: `Navigated to ${url}` };
      } else {
        // 创建新标签页
        const newTabId = `tab-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
        console.log(`[ElectronBridge] Creating new tab ${newTabId} with ${url}`);
        tabManager.createTab(newTabId, url, url);
        // 通知前端创建了新标签页
        sender.send('webview:create-tab', {
          requestId: newTabId,
          url,
          title: url,
        });
        return { success: true, message: `Created new tab with ${url}`, tabId: newTabId };
      }
    }

    case 'web_agent': {
      // 在当前网页执行自动化操作
      const action = args.action; // click, fill, select, scroll, wait
      const selector = args.selector;
      const value = args.value;

      if (!action || !selector) {
        return { success: false, error: 'Missing action or selector parameter' };
      }

      // 获取当前活动标签页
      const currentTabId = await getActiveTabId();
      if (!currentTabId) {
        return { success: false, error: 'No active tab' };
      }

      // 通过 page:execute_action 执行
      try {
        const result = await tabManager.executeJavaScript(currentTabId, `
          (function() {
            const el = document.querySelector(${JSON.stringify(selector)});
            if (!el) return JSON.stringify({ success: false, message: 'Element not found' });
            
            switch (${JSON.stringify(action)}) {
              case 'click':
                el.click();
                return JSON.stringify({ success: true, message: 'Clicked' });
              case 'fill':
                el.value = ${JSON.stringify(value || '')};
                el.dispatchEvent(new Event('input', { bubbles: true }));
                el.dispatchEvent(new Event('change', { bubbles: true }));
                return JSON.stringify({ success: true, message: 'Filled' });
              case 'scroll':
                el.scrollIntoView({ behavior: 'smooth', block: 'center' });
                return JSON.stringify({ success: true, message: 'Scrolled' });
              default:
                return JSON.stringify({ success: false, message: 'Unknown action' });
            }
          })()
        `);
        return JSON.parse(result);
      } catch (err) {
        return { success: false, error: String(err) };
      }
    }

    case 'summarize_page': {
      // 总结当前页面
      const currentTabId = await getActiveTabId();
      if (!currentTabId) {
        return { success: false, error: 'No active tab' };
      }

      try {
        const content = await tabManager.executeJavaScript(currentTabId, `
          document.body.innerText.substring(0, 15000)
        `);
        return { success: true, content, length: content.length };
      } catch (err) {
        return { success: false, error: String(err) };
      }
    }

    case 'translate': {
      // 翻译当前页面
      const targetLanguage = args.target_language;
      const currentTabId = await getActiveTabId();
      if (!currentTabId) {
        return { success: false, error: 'No active tab' };
      }

      // TODO: 实现真正的翻译逻辑（可以调用翻译 API）
      return { success: true, message: `Translation to ${targetLanguage} - placeholder` };
    }

    case 'export_markdown': {
      // 导出为 Markdown
      const currentTabId = await getActiveTabId();
      if (!currentTabId) {
        return { success: false, error: 'No active tab' };
      }

      try {
        const content = await tabManager.executeJavaScript(currentTabId, `
          document.body.innerText
        `);
        // 简单转换为 Markdown（实际应该使用更复杂的 HTML to Markdown 转换）
        const markdown = `# ${document.title}\n\n${content}`;
        return { success: true, markdown, length: markdown.length };
      } catch (err) {
        return { success: false, error: String(err) };
      }
    }

    default:
      return { success: false, error: `Unknown tool: ${toolName}` };
  }
}

// ===================================================
// MCP Servers
// ===================================================
ipcMain.handle('mcp:load_servers', async (_event, serversJson: string) => {
  try {
    const nat = getNative();
    if (!nat) {
      throw new Error('Native module not available');
    }
    nat.mcpLoadServers(serversJson);
    console.log('[IPC] mcp:load_servers called');
  } catch (err) {
    console.error('[IPC] mcp:load_servers failed:', err);
    throw err;
  }
});
