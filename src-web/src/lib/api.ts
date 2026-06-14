/**
 * 统一 API 适配层
 *
 * 替代 Tauri invoke() 调用，封装所有 Electron IPC 通信。
 * 每个方法直接调用 window.electronAPI.invoke(channel, ...args)
 * 使用位置参数（与 ipc-handlers.ts 签名一致）。
 *
 * 使用方式:
 *   import { db, ai, tab, page, screenshot, cache, dialog, shell, skills, agent } from '@/lib/api';
 */

// ===== 底层 invoke 封装 =====
function invoke<T = any>(channel: string, ...args: any[]): Promise<T> {
  if (!window.electronAPI) {
    console.warn('[api] electronAPI not available');
    return Promise.reject(new Error('Electron API not available'));
  }
  return window.electronAPI.invoke(channel, ...args);
}

/**
 * 解析 N-API 返回的 JSON 字符串
 * Rust db_* 函数返回 Result<String>，在 JS 侧是 string 类型
 */
function parseJSON<T>(result: any): T {
  if (typeof result === 'string') {
    try {
      return JSON.parse(result) as T;
    } catch {
      return result as unknown as T;
    }
  }
  return result as T;
}

/**
 * 解析 N-API 返回的 JSON 字符串，null 表示无数据
 */
function parseJSONOrNull<T>(result: any): T | null {
  if (result === null || result === undefined) return null;
  if (typeof result === 'string') {
    try {
      return JSON.parse(result) as T;
    } catch {
      return result as unknown as T;
    }
  }
  return result as T;
}

// ============================================================
// DB 操作 — 数据库 CRUD
// ============================================================
export const db = {
  // ----- Conversations -----
  listConversations: async () =>
    parseJSON<any[]>(await invoke<string>('db:list_conversations')),

  getConversation: async (id: string) =>
    parseJSON<any>(await invoke<string>('db:get_conversation', id)),

  createConversation: async (title: string, modelId?: string) =>
    parseJSON<any>(await invoke<string>('db:create_conversation', title, modelId ?? '')),

  updateConversation: async (id: string, updates: { title?: string; isPinned?: boolean; modelId?: string }) =>
    parseJSON<any>(await invoke<string>('db:update_conversation', id, updates.title ?? null, updates.isPinned ?? null, updates.modelId ?? null)),

  deleteConversation: (id: string) =>
    invoke<void>('db:delete_conversation', id),

  getConversationWithMessages: async (id: string) =>
    parseJSON<any>(await invoke<string>('db:get_conversation_with_messages', id)),

  // ----- Messages -----
  listMessages: async (conversationId: string) =>
    parseJSON<any[]>(await invoke<string>('db:list_messages', conversationId)),

  getMessage: async (id: string) =>
    parseJSON<any>(await invoke<string>('db:get_message', id)),

  createMessage: async (conversationId: string, role: string, content: string, attachments?: string) =>
    parseJSON<any>(await invoke<string>('db:create_message', conversationId, role, content, attachments)),

  updateMessage: (id: string, content: string, status?: string) =>
    invoke<any>('db:update_message', id, content, status),

  deleteMessage: (id: string) =>
    invoke<void>('db:delete_message', id),

  appendMessageContent: (id: string, delta: string, isThinking: boolean = false) =>
    invoke<void>('db:append_message_content', id, delta, isThinking),

  completeMessage: (id: string) =>
    invoke<void>('db:complete_message', id),

  setMessageFeedback: (id: string, feedback: string) =>
    invoke<void>('db:set_message_feedback', id, feedback),

  // ----- Bookmarks -----
  listBookmarks: async (folderId?: string | null) =>
    parseJSON<any[]>(await invoke<string>('db:list_bookmarks', folderId ?? null)),

  createBookmark: async (title: string, url: string, favicon?: string | null, folderId?: string | null) =>
    parseJSON<any>(await invoke<string>('db:create_bookmark', title, url, favicon ?? null, folderId ?? null)),

  deleteBookmark: (id: string) =>
    invoke<void>('db:delete_bookmark', id),

  listBookmarkFolders: async (parentId?: string | null) =>
    parseJSON<any[]>(await invoke<string>('db:list_bookmark_folders', parentId ?? null)),

  createBookmarkFolder: async (name: string, parentId?: string | null) =>
    parseJSON<any>(await invoke<string>('db:create_bookmark_folder', name, parentId ?? null)),

  deleteBookmarkFolder: (id: string) =>
    invoke<void>('db:delete_bookmark_folder', id),

  // ----- Settings -----
  getSettings: async () =>
    parseJSON<Record<string, string>>(await invoke<string>('db:get_settings')),

  getSetting: (key: string) =>
    invoke<string | null>('db:get_setting', key),

  setSetting: (key: string, value: string) =>
    invoke<void>('db:set_setting', key, value),

  // ----- Model Configs -----
  listModelConfigs: async () =>
    parseJSON<any[]>(await invoke<string>('db:list_model_configs')),

  getModelConfig: async (id: string) =>
    parseJSON<any>(await invoke<string>('db:get_model_config', id)),

  getActiveModel: async () =>
    parseJSONOrNull<any>(await invoke<any>('db:get_active_model')),

  createModelConfig: async (config: Record<string, any>) =>
    parseJSON<any>(await invoke<string>('db:create_model_config',
      config.name, config.provider, config.modelId,
      config.apiKey ?? null, config.baseUrl ?? null,
      config.temperature, config.topP, config.maxTokens)),

  updateModelConfig: async (id: string, updates: Record<string, any>) =>
    parseJSON<any>(await invoke<string>('db:update_model_config',
      id, updates.name ?? null,
      updates.apiKey !== undefined ? [updates.apiKey] : null,
      updates.baseUrl !== undefined ? [updates.baseUrl] : null,
      updates.temperature, updates.topP, updates.maxTokens)),

  setActiveModel: (id: string) =>
    invoke<void>('db:set_active_model', id),

  deleteModelConfig: (id: string) =>
    invoke<void>('db:delete_model_config', id),

  // ----- Skills Config -----
  getSkillsDirectory: async () => {
    console.log('[API] Calling db:get_skills_directory');
    const result = await invoke<string | null>('db:get_skills_directory');
    console.log('[API] db:get_skills_directory result:', result);
    return result;
  },

  setSkillsDirectory: (directory: string) =>
    invoke<void>('db:set_skills_directory', directory),

  getIqsApiKey: async () => {
    console.log('[API] Calling db:get_iqs_api_key');
    const result = await invoke<string | null>('db:get_iqs_api_key');
    console.log('[API] db:get_iqs_api_key result:', result);
    return result;
  },

  setIqsApiKey: (apiKey: string) =>
    invoke<void>('db:set_iqs_api_key', apiKey),

  // ----- MCP Servers -----
  listMcpServers: async () =>
    parseJSON<any[]>(await invoke<string>('db:list_mcp_servers')),

  getMcpServer: async (id: string) =>
    parseJSON<any>(await invoke<string>('db:get_mcp_server', id)),

  createMcpServer: async (config: Record<string, any>) =>
    parseJSON<any>(await invoke<string>('db:create_mcp_server',
      config.name, config.serverType ?? 'stdio',
      config.url ?? null, config.command ?? null, config.args ?? null,
      config.cwd ?? null, config.env ?? null, config.timeout ?? null,
      config.disabled ?? false, config.enabled ?? true,
      // headers 需要序列化为 JSON 字符串
      config.headers ? JSON.stringify(config.headers) : null)),

  updateMcpServer: async (id: string, updates: Record<string, any>) =>
    parseJSON<any>(await invoke<string>('db:update_mcp_server',
      id, updates.name ?? null,
      updates.serverType ?? null,
      // url/command/args/cwd/env/timeout 使用 Option<Option<String>> 类型
      // None = 不更新, Some(null) = 设置为null, Some(value) = 更新为新值
      updates.url !== undefined ? updates.url : null,
      updates.command !== undefined ? updates.command : null,
      updates.args !== undefined ? updates.args : null,
      updates.cwd !== undefined ? updates.cwd : null,
      updates.env !== undefined ? updates.env : null,
      updates.timeout !== undefined ? updates.timeout : null,
      updates.disabled, updates.enabled,
      // headers 需要序列化为 JSON 字符串
      updates.headers !== undefined ? (updates.headers ? JSON.stringify(updates.headers) : null) : null)),

  deleteMcpServer: (id: string) =>
    invoke<void>('db:delete_mcp_server', id),

  testMcpServer: (config: Record<string, any>) =>
    invoke<any>('db:test_mcp_server', JSON.stringify(config)),

  importMcpServersFromJson: (json: string) =>
    invoke<any>('db:import_mcp_servers_from_json', json),

  // ----- History -----
  listHistory: async (limit: number, offset?: number) =>
    parseJSON<any[]>(await invoke<string>('db:list_history', limit, offset ?? 0)),

  searchHistory: async (query: string, limit?: number) =>
    parseJSON<any[]>(await invoke<string>('db:search_history', query, limit ?? 100)),

  addHistory: (title: string, url: string) =>
    invoke<any>('db:add_history', title, url),

  clearHistory: () =>
    invoke<void>('db:clear_history'),

  deleteHistoryEntry: (id: string) =>
    invoke<void>('db:delete_history_entry', id),

  // ----- Agent Prompts -----
  listAgentPrompts: async () =>
    parseJSON<any[]>(await invoke<string>('db:list_agent_prompts')),

  getAgentPrompt: async (name: string) => {
    const result = await invoke<string | null>('db:get_agent_prompt', name);
    return result ? parseJSON<any>(result) : null;
  },

  setAgentPrompt: (name: string, content: string, description?: string) =>
    invoke<void>('db:set_agent_prompt', name, content, description ?? null),

  toggleAgentPrompt: (name: string) =>
    invoke<boolean>('db:toggle_agent_prompt', name),
};

// ============================================================
// AI 对话
// ============================================================
export const ai = {
  /**
   * 发送聊天消息（流式响应通过事件推送）
   */
  sendChat: (config: any, messages: any[], conversationId: string, messageId: string) =>
    invoke<void>('ai:send_chat', {
      config: typeof config === 'string' ? config : JSON.stringify(config),
      messages: typeof messages === 'string' ? messages : JSON.stringify(messages),
      conversationId,
      messageId,
    }),

  stopGeneration: () =>
    invoke<void>('ai:stop_generation'),

  generateTitle: (content: string, config: any) =>
    invoke<string>('ai:generate_title', content, typeof config === 'string' ? config : JSON.stringify(config)),
};

// ============================================================
// Agent 操作
// ============================================================
export const agent = {
  execute: (params: Record<string, any>) =>
    invoke<void>('agent:execute', params),

  configureQwen: (config: Record<string, any>) =>
    invoke<any>('agent:configure_qwen', config),

  summarizePage: (params: Record<string, any>) =>
    invoke<void>('agent:summarize_page', params),

  extractMemory: (params: Record<string, any>) =>
    invoke<any>('agent:extract_memory', params),
};

// ============================================================
// 标签页管理
// ============================================================
export const tab = {
  create: (url: string, title?: string) =>
    invoke<any>('tab:create', url, title ?? '新标签页'),

  switch: (id: string) =>
    invoke<void>('tab:switch', id),

  close: (id: string) =>
    invoke<void>('tab:close', id),

  navigate: (id: string, url: string) =>
    invoke<void>('tab:navigate', id, url),

  back: (id: string) =>
    invoke<boolean>('tab:back', id),

  forward: (id: string) =>
    invoke<boolean>('tab:forward', id),

  getState: (id: string) =>
    invoke<any>('tab:get_state', id),

  getTitle: (id: string) =>
    invoke<string>('tab:get_title', id),

  setActive: (id: string) =>
    invoke<void>('tab:set_active', id),
};

// ============================================================
// 页面操作
// ============================================================
export const page = {
  getContent: (tabId: string) =>
    invoke<string>('page:get_content', tabId),

  screenshot: (tabId: string) =>
    invoke<string>('page:screenshot', tabId),

  executeScript: (tabId: string, script: string) =>
    invoke<any>('page:execute_script', tabId, script),

  injectContext: (tabId: string) =>
    invoke<any>('page:inject_context', tabId),

  summarize: (tabId: string) =>
    invoke<string>('page:summarize', tabId),

  executeAction: (tabId: string, action: string, selector: string, value?: string) =>
    invoke<string>('page:execute_action', tabId, action, selector, value),
};

// ============================================================
// 截图
// ============================================================
export const screenshot = {
  captureFull: () =>
    invoke<string>('screenshot:capture_full'),

  captureRegion: (base64Data: string, x: number, y: number, width: number, height: number, screenWidth: number, screenHeight: number) =>
    invoke<string>('screenshot:capture_region', base64Data, x, y, width, height, screenWidth, screenHeight),

  save: (base64Data: string, filePath: string) =>
    invoke<boolean>('screenshot:save', base64Data, filePath),

  copyToClipboard: (base64Data: string) =>
    invoke<boolean>('screenshot:copy_to_clipboard', base64Data),
};

// ============================================================
// Skills 管理
// ============================================================
export const skills = {
  list: async () =>
    parseJSON<any[]>(await invoke<string>('skills:list')),

  delete: (id: string) =>
    invoke<void>('skills:delete_skill', id),

  toggle: (id: string, enabled: boolean) =>
    invoke<void>('skills:toggle_skill', { skill_id: id, enabled }),

  importMarkdown: async (content: string) =>
    parseJSON<any>(await invoke<string>('skills:import_skill_from_markdown', content)),

  importDirectory: async (dirPath: string) =>
    parseJSON<any>(await invoke<string>('skills:import_skill_from_directory', dirPath)),

  setDirectory: (dir: string) =>
    invoke<void>('db:set_skills_directory', dir),

  listFiles: async () =>
    parseJSON<any[]>(await invoke<string>('skills:list_skill_files')),

  getContent: (id: string) =>
    invoke<string>('skills:get_skill_content', id),
};

// ============================================================
// 页面缓存
// ============================================================
export const cache = {
  save: (key: string, data: string) =>
    invoke<string>('cache:save', key, data),

  load: (key: string) =>
    invoke<string | null>('cache:load', key),

  cleanup: () =>
    invoke<number>('cache:cleanup'),
};

// ============================================================
// 对话框
// ============================================================
export const dialog = {
  openFile: (options?: Record<string, any>) =>
    invoke<any>('dialog:open_file', options),

  saveFile: (options?: Record<string, any>) =>
    invoke<any>('dialog:save_file', options),
};

// ============================================================
// Shell
// ============================================================
export const shell = {
  openUrl: (url: string) =>
    invoke<void>('shell:open_url', url),
};

// ============================================================
// 窗口控制
// ============================================================
export const win = {
  minimize: () => invoke<void>('window:minimize'),
  maximize: () => invoke<void>('window:maximize'),
  close: () => invoke<void>('window:close'),
  isMaximized: () => invoke<boolean>('window:is_maximized'),
};

// ============================================================
// MCP Servers
// ============================================================
export const mcp = {
  /**
   * 加载所有启用的 MCP Servers
   * @param servers - MCP Server 配置数组
   */
  loadServers: (servers: Array<Record<string, any>>) =>
    invoke<void>('mcp:load_servers', JSON.stringify(servers)),
};
