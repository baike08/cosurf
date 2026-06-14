/**
 * CoSurf Electron Preload 脚本
 * 
 * 运行在主窗口的渲染进程中（contextIsolation: true）
 * 通过 contextBridge 暴露安全的 API 给前端 React 应用
 * 
 * 替代 Tauri 的 invoke/listen/emit API
 */

import { contextBridge, ipcRenderer, IpcRendererEvent } from 'electron';

// ===== ElectronAPI 接口定义 =====
export interface ElectronAPI {
  // 发送请求并等待回复 (替代 Tauri invoke)
  invoke(channel: string, ...args: any[]): Promise<any>;
  
  // 监听主进程发来的事件 (替代 Tauri listen)
  on(channel: string, callback: (payload: any) => void): () => void;
  
  // 向主进程发送消息 (替代 Tauri emit)
  send(channel: string, ...args: any[]): void;
  
  // 一次性监听
  once(channel: string, callback: (payload: any) => void): void;
  
  // 移除监听器
  removeAllListeners(channel: string): void;
}

// ===== 允许的 IPC 通道白名单 (安全控制) =====
const ALLOWED_INVOKE_CHANNELS = [
  // 数据库操作
  'db:list_conversations',
  'db:get_conversation',
  'db:create_conversation',
  'db:update_conversation',
  'db:delete_conversation',
  'db:get_conversation_with_messages',
  'db:list_messages',
  'db:get_message',
  'db:create_message',
  'db:update_message',
  'db:delete_message',
  'db:append_message_content',
  'db:complete_message',
  'db:set_message_feedback',
  'db:list_bookmarks',
  'db:create_bookmark',
  'db:delete_bookmark',
  'db:list_bookmark_folders',
  'db:create_bookmark_folder',
  'db:delete_bookmark_folder',
  'db:get_settings',
  'db:get_setting',
  'db:set_setting',
  'db:list_model_configs',
  'db:get_model_config',
  'db:get_active_model',
  'db:create_model_config',
  'db:update_model_config',
  'db:set_active_model',
  'db:delete_model_config',
  'db:get_skills_directory',
  'db:set_skills_directory',
  'db:list_mcp_servers',
  'db:get_mcp_server',
  'db:create_mcp_server',
  'db:update_mcp_server',
  'db:delete_mcp_server',
  'db:test_mcp_server',
  'db:import_mcp_servers_from_json',
  // MCP
  'mcp:load_servers',
  'db:list_history',
  'db:search_history',
  'db:add_history',
  'db:clear_history',
  'db:delete_history_entry',
  // Agent Prompts
  'db:list_agent_prompts',
  'db:get_agent_prompt',
  'db:set_agent_prompt',
  'db:toggle_agent_prompt',
  // AI
  'ai:send_chat',
  'ai:stop_generation',
  'ai:generate_title',
  // Agent
  'agent:execute',
  'agent:configure_qwen',
  'agent:summarize_page',
  'agent:extract_memory',
  // 标签页管理
  'tab:create',
  'tab:switch',
  'tab:close',
  'tab:navigate',
  'tab:back',
  'tab:forward',
  'tab:get_state',
  'tab:get_title',
  'tab:set_active',
  // 页面操作
  'page:get_content',
  'page:screenshot',
  'page:execute_script',
  'page:inject_context',
  'page:summarize',
  'page:execute_action',
  // 截图
  'screenshot:capture_full',
  'screenshot:capture_region',
  'screenshot:save',
  'screenshot:copy_to_clipboard',
  // Skills
  'skills:list',
  'skills:delete_skill',
  'skills:toggle_skill',
  'skills:import_skill_from_markdown',
  'skills:import_skill_from_directory',
  'skills:list_skill_files',
  'skills:get_skill_content',
  'db:set_skills_directory',
  // 页面缓存
  'cache:save',
  'cache:load',
  'cache:cleanup',
  // 对话框
  'dialog:open_file',
  'dialog:save_file',
  // Shell
  'shell:open_url',
  // 窗口控制
  'window:minimize',
  'window:maximize',
  'window:close',
  'window:is_maximized',
];

const ALLOWED_SEND_CHANNELS = [
  'open-new-tab',
  'cosurf:new-tab-response',
  'cosurf:tab-url-response',
  'window:minimize',
  'window:maximize',
  'window:close',
];

const ALLOWED_ON_CHANNELS = [
  'ai:stream-chunk',
  'ai:stream-error',
  'ai:tool-call-start',
  'ai:tool-call-result',
  'tab:create',
  'tab:navigate',
  'tab:title-updated',
  'tab:loading',
  'tab:loaded',
  'tab:switched',
  'screenshot-fullscreen-captured',
  'screenshot-captured',
  'cosurf:page-content',
  'cosurf:page-content-error',
  'shortcut:screenshot',
  'updater:update-available',
  'webview:create-tab',
  'webview:get-tab-info',
  'webview:get-tab-url',
  'webview:navigating',
  'webview:reload',
  'webview:get-content',
  'cosurf:tab-url-response',
  'cosurf:new-tab-response',
  'element-selected',
];

// ===== 暴露 API =====
const electronAPI: ElectronAPI = {
  invoke(channel: string, ...args: any[]): Promise<any> {
    if (!ALLOWED_INVOKE_CHANNELS.includes(channel)) {
      console.warn(`[Preload] Blocked invoke to unauthorized channel: ${channel}`);
      return Promise.reject(new Error(`Unauthorized IPC channel: ${channel}`));
    }
    return ipcRenderer.invoke(channel, ...args);
  },

  on(channel: string, callback: (payload: any) => void): () => void {
    if (!ALLOWED_ON_CHANNELS.includes(channel)) {
      console.warn(`[Preload] Blocked listener on unauthorized channel: ${channel}`);
      return () => {};
    }
    const listener = (_event: IpcRendererEvent, ...args: any[]) => {
      callback(args.length === 1 ? args[0] : args);
    };
    ipcRenderer.on(channel, listener);
    // 返回取消订阅函数
    return () => {
      ipcRenderer.removeListener(channel, listener);
    };
  },

  send(channel: string, ...args: any[]): void {
    if (!ALLOWED_SEND_CHANNELS.includes(channel)) {
      console.warn(`[Preload] Blocked send to unauthorized channel: ${channel}`);
      return;
    }
    ipcRenderer.send(channel, ...args);
  },

  once(channel: string, callback: (payload: any) => void): void {
    const listener = (_event: IpcRendererEvent, ...args: any[]) => {
      callback(args.length === 1 ? args[0] : args);
    };
    ipcRenderer.once(channel, listener);
  },

  removeAllListeners(channel: string): void {
    ipcRenderer.removeAllListeners(channel);
  },
};

// 通过 contextBridge 安全地暴露给渲染进程
contextBridge.exposeInMainWorld('electronAPI', electronAPI);

// ===== 窗口控制 API (给标题栏用) =====
contextBridge.exposeInMainWorld('windowControls', {
  minimize: () => ipcRenderer.invoke('window:minimize'),
  maximize: () => ipcRenderer.invoke('window:maximize'),
  close: () => ipcRenderer.invoke('window:close'),
  isMaximized: () => ipcRenderer.invoke('window:is_maximized'),
});
