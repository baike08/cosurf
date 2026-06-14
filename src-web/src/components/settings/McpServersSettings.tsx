import { useState, useEffect, useCallback } from "react";
import { db } from '@/lib/api';
import { mcp } from '@/lib/api';
import { Trash2, Edit2, X, TestTube, Loader2, Upload, Play, Power, PowerOff, Server, RefreshCw } from "lucide-react";
import Editor from "@monaco-editor/react";

// Tauri 错误返回 { code, message } 对象，需要提取 message
function getErrorMessage(err: unknown): string {
  if (err && typeof err === "object") {
    const e = err as Record<string, unknown>;
    if (typeof e.message === "string") return e.message;
    if (typeof e.error === "string") return e.error;
    try { return JSON.stringify(err); } catch { /* ignore */ }
  }
  return String(err);
}

interface McpServerConfig {
  id: string;
  name: string;
  serverType: "stdio" | "http" | "streamableHttp" | "sse";
  url?: string;
  command?: string;
  args?: string[];
  cwd?: string;
  env?: Record<string, string>;
  headers?: Record<string, string>;
  disabled: boolean;
  timeout?: number;
  enabled: boolean;
  createdAt: number;
  updatedAt: number;
}

interface ToolInfo {
  name: string;
  description?: string;
  inputSchema?: unknown;
}

// MCP Server 编辑 JSON 格式
interface McpServerEditJson {
  name: string;
  serverType?: "stdio" | "http" | "streamableHttp" | "sse";
  command?: string;
  args?: string[];
  cwd?: string;
  url?: string;
  headers?: Record<string, string>;
  env?: Record<string, string>;
  disabled?: boolean;
  timeout?: number;
}

// 示例 JSON
const EXAMPLE_JSON = `{
  "mcpServers": {
    "iqs-mcp-server-search": {
      "type": "streamableHttp",
      "url": "https://iqs-mcp.aliyuncs.com/mcp-servers/iqs-mcp-server-search",
      "headers": {
        "X-API-Key": "your-api-key"
      }
    },
    "filesystem": {
      "type": "stdio",
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/path/to/files"]
    },
    "github": {
      "type": "stdio",
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-github"],
      "env": {
        "GITHUB_PERSONAL_ACCESS_TOKEN": "ghp_..."
      }
    }
  }
}`;

export function McpServersSettings() {
  const [mcpServers, setMcpServers] = useState<McpServerConfig[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Import dialog
  const [showImportDialog, setShowImportDialog] = useState(false);
  const [importJson, setImportJson] = useState("");
  const [importError, setImportError] = useState<string | null>(null);

  // Edit dialog
  const [editingServer, setEditingServer] = useState<McpServerConfig | null>(null);
  const [editJson, setEditJson] = useState("");
  const [editError, setEditError] = useState<string | null>(null);

  // Test
  const [testingIds, setTestingIds] = useState<Set<string>>(new Set());
  const [testResults, setTestResults] = useState<Record<string, ToolInfo[]>>({});
  const [testErrors, setTestErrors] = useState<Record<string, string>>({});

  // Toggle
  const [togglingId, setTogglingId] = useState<string | null>(null);

  // 加载 MCP Servers
  const loadMcpServers = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      const servers = await db.listMcpServers();
      setMcpServers(servers);
      
      // 通知 Native 模块加载所有启用的 MCP Servers
      const enabledServers = servers.filter(s => s.enabled);
      if (enabledServers.length > 0) {
        console.log(`[MCP] Loading ${enabledServers.length} enabled servers...`);
        await mcp.loadServers(enabledServers);
      }
    } catch (err) {
      setError(`Failed to load MCP servers: ${getErrorMessage(err)}`);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadMcpServers();
  }, [loadMcpServers]);

  // 当 MCP Servers 加载完成后，自动为每个启用的 server 加载工具列表
  useEffect(() => {
    if (mcpServers.length > 0) {
      mcpServers.forEach(server => {
        // 只为启用的且尚未加载过工具列表的 server 加载
        if (server.enabled && !(server.id in testResults)) {
          console.log(`[MCP] Auto-loading tools for server: ${server.name}`);
          // 异步加载工具列表，不阻塞 UI
          const loadTools = async () => {
            try {
              setTestingIds(prev => new Set(prev).add(server.id));
              
              const result = await db.testMcpServer({
                serverType: server.serverType,
                url: isHttpType(server.serverType) ? (server.url || "") : null,
                command: server.command || null,
                args: server.args || null,
                env: server.env || null,
                apiKey: null, // 不需要额外传递 apiKey，API Key 已经在 headers 中
                headers: server.headers || null,
              });
              console.log(`[MCP] Raw test result for ${server.name}:`, typeof result, result);
              
              // 后端返回的是 JSON 字符串，需要解析
              let parsedResult;
              if (typeof result === 'string') {
                try {
                  parsedResult = JSON.parse(result);
                  console.log(`[MCP] Parsed result:`, parsedResult);
                } catch (e) {
                  console.error(`[MCP] Failed to parse result:`, e);
                  throw new Error(`Failed to parse MCP test result: ${e}`);
                }
              } else {
                parsedResult = result;
              }
              
              const tools = parsedResult.tools || [];
              console.log(`[MCP] Extracted ${tools.length} tools for server: ${server.name}`);
              setTestResults(prev => ({ ...prev, [server.id]: tools }));
            } catch (err) {
              console.warn(`Failed to load tools for server ${server.name}:`, err);
              setTestErrors(prev => ({ ...prev, [server.id]: getErrorMessage(err) }));
            } finally {
              setTestingIds(prev => {
                const next = new Set(prev);
                next.delete(server.id);
                return next;
              });
            }
          };
          loadTools();
        }
      });
    }
  }, [mcpServers]);

  // 判断是否为 HTTP 类型（需要 URL 的类型）
  const isHttpType = (serverType: string) => {
    return ["http", "streamableHttp", "sse"].includes(serverType);
  };

  // 将 McpServerConfig 转为编辑 JSON
  const serverToEditJson = (server: McpServerConfig): string => {
    const obj: McpServerEditJson = {
      name: server.name,
      serverType: server.serverType,
    };
    if (server.command) obj.command = server.command;
    if (server.args?.length) obj.args = server.args;
    if (server.cwd) obj.cwd = server.cwd;
    if (server.url) obj.url = server.url;
    if (server.headers && Object.keys(server.headers).length > 0) obj.headers = server.headers;
    if (server.env && Object.keys(server.env).length > 0) obj.env = server.env;
    if (server.timeout) obj.timeout = server.timeout;
    return JSON.stringify(obj, null, 2);
  };

  // 验证 JSON 语法
  const validateJson = (value: string): string | null => {
    try {
      JSON.parse(value);
      return null;
    } catch (e) {
      return (e as Error).message;
    }
  };

  // 导入 JSON
  const handleImport = async () => {
    const validationError = validateJson(importJson);
    if (validationError) {
      setImportError(`JSON 格式错误: ${validationError}`);
      return;
    }

    try {
      setImportError(null);
      setLoading(true);
      await db.importMcpServersFromJson(importJson);
      setShowImportDialog(false);
      setImportJson("");
      await loadMcpServers();
    } catch (err) {
      setImportError(`导入失败: ${getErrorMessage(err)}`);
    } finally {
      setLoading(false);
    }
  };

  // 保存编辑
  const handleSaveEdit = async () => {
    if (!editingServer) return;

    const validationError = validateJson(editJson);
    if (validationError) {
      setEditError(`JSON 格式错误: ${validationError}`);
      return;
    }

    try {
      setEditError(null);
      const parsed = JSON.parse(editJson) as McpServerEditJson;

      const request = {
        name: parsed.name,
        serverType: parsed.serverType || "stdio",
        command: parsed.command || null,
        args: parsed.args || null,
        cwd: parsed.cwd || null,
        url: parsed.url || null,
        headers: parsed.headers || null,
        env: parsed.env || null,
        disabled: parsed.disabled ?? false,
        timeout: parsed.timeout || null,
        enabled: !(parsed.disabled ?? false),
      };

      await db.updateMcpServer(editingServer.id, request);

      setEditingServer(null);
      setEditJson("");
      await loadMcpServers();
    } catch (err) {
      setEditError(`保存失败: ${getErrorMessage(err)}`);
    }
  };

  // 启用/禁用
  const handleToggle = async (server: McpServerConfig) => {
    setTogglingId(server.id);
    try {
      const newEnabled = !server.enabled;
      await db.updateMcpServer(server.id, { enabled: newEnabled, disabled: !newEnabled });
      await loadMcpServers();
    } catch (err) {
      setError(`Toggle failed: ${getErrorMessage(err)}`);
    } finally {
      setTogglingId(null);
    }
  };

  // 测试
  const handleTest = async (server: McpServerConfig) => {
    console.log(`[MCP] Testing server: ${server.name}`);
    setTestingIds(prev => new Set(prev).add(server.id));
    setError(null);
    setTestErrors(prev => { const next = { ...prev }; delete next[server.id]; return next; });

    try {
      const result = await db.testMcpServer({
        serverType: server.serverType,
        url: isHttpType(server.serverType) ? (server.url || "") : null,
        command: server.command || null,
        args: server.args || null,
        env: server.env || null,
        apiKey: null, // 不需要额外传递 apiKey，API Key 已经在 headers 中
        headers: server.headers || null,
      });

      console.log(`[MCP] Raw test result for ${server.name}:`, typeof result, result);
      
      // 后端返回的是 JSON 字符串，需要解析
      let parsedResult;
      if (typeof result === 'string') {
        try {
          parsedResult = JSON.parse(result);
          console.log(`[MCP] Parsed result:`, parsedResult);
        } catch (e) {
          console.error(`[MCP] Failed to parse result:`, e);
          throw new Error(`Failed to parse MCP test result: ${e}`);
        }
      } else {
        parsedResult = result;
      }
      
      const tools = parsedResult.tools || [];
      console.log(`[MCP] Test completed for ${server.name}: ${tools.length} tools`);
      setTestResults(prev => ({ ...prev, [server.id]: tools }));
    } catch (err) {
      console.error(`[MCP] Test failed for ${server.name}:`, err);
      setTestErrors(prev => ({ ...prev, [server.id]: getErrorMessage(err) }));
    } finally {
      setTestingIds(prev => {
        const next = new Set(prev);
        next.delete(server.id);
        return next;
      });
    }
  };

  // 删除
  const handleDelete = async (id: string) => {
    if (!confirm("确定要删除这个 MCP Server 吗？")) return;
    try {
      setError(null);
      await db.deleteMcpServer(id);
      setTestResults(prev => { const next = { ...prev }; delete next[id]; return next; });
      setTestErrors(prev => { const next = { ...prev }; delete next[id]; return next; });
      await loadMcpServers();
    } catch (err) {
      setError(`删除失败: ${getErrorMessage(err)}`);
    }
  };

  // 显示配置信息
  const getServerDisplay = (server: McpServerConfig) => {
    if (isHttpType(server.serverType)) {
      return server.url || "(no URL)";
    }
    const parts = [server.command || ""];
    if (server.args?.length) parts.push(server.args.join(" "));
    return parts.join(" ") || "(no command)";
  };

  return (
    <div className="space-y-4">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h3 className="text-sm font-medium">MCP Servers</h3>
          <p className="text-2xs text-content-secondary mt-1">
            配置 MCP 服务器，让 AI Agent 使用外部工具
          </p>
        </div>
        <button
          onClick={() => setShowImportDialog(true)}
          className="px-3 py-1.5 text-xs bg-primary text-primary-foreground rounded-md hover:bg-primary/90 flex items-center gap-1"
        >
          <Upload className="w-3 h-3" />
          导入 JSON
        </button>
      </div>

      {/* Error */}
      {error && (
        <div className="p-3 bg-red-500/10 border border-red-500/30 rounded-lg text-xs text-red-600 dark:text-red-400 flex items-start justify-between">
          <span>{error}</span>
          <button onClick={() => setError(null)} className="ml-2 p-0.5 hover:bg-red-500/20 rounded">
            <X className="w-3 h-3" />
          </button>
        </div>
      )}

      {/* Import Dialog */}
      {showImportDialog && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
          <div className="w-[700px] h-[550px] bg-surface rounded-xl shadow-2xl border border-border flex flex-col">
            <div className="flex items-center justify-between px-4 py-3 border-b border-border">
              <h3 className="text-sm font-medium flex items-center gap-2">
                <Upload className="w-4 h-4" />
                导入 MCP Server 配置
              </h3>
              <button onClick={() => { setShowImportDialog(false); setImportJson(""); setImportError(null); }} className="p-1 hover:bg-surface-hover rounded">
                <X className="w-4 h-4" />
              </button>
            </div>

            <div className="flex-1 overflow-hidden flex flex-col">
              {/* 示例 */}
              <div className="px-4 py-2 border-b border-border">
                <details className="text-2xs">
                  <summary className="cursor-pointer text-content-secondary hover:text-content">
                    查看 JSON 格式示例
                  </summary>
                  <pre className="mt-2 p-2 bg-surface-secondary rounded border border-border overflow-x-auto font-mono text-2xs max-h-40">
                    {EXAMPLE_JSON}
                  </pre>
                </details>
              </div>

              {/* Editor */}
              <div className="flex-1 min-h-0">
                <Editor
                  height="100%"
                  defaultLanguage="json"
                  value={importJson}
                  onChange={(v) => setImportJson(v || "")}
                  theme="vs-dark"
                  options={{
                    minimap: { enabled: false },
                    fontSize: 12,
                    lineNumbers: "on",
                    scrollBeyondLastLine: false,
                    wordWrap: "on",
                    tabSize: 2,
                    automaticLayout: true,
                    padding: { top: 8, bottom: 8 },
                  }}
                />
              </div>

              {/* Import Error */}
              {importError && (
                <div className="mx-4 mt-2 p-2 bg-red-500/10 border border-red-500/30 rounded text-xs text-red-600 dark:text-red-400">
                  {importError}
                </div>
              )}
            </div>

            <div className="flex items-center justify-end gap-2 px-4 py-3 border-t border-border">
              <button
                onClick={() => { setShowImportDialog(false); setImportJson(""); setImportError(null); }}
                className="px-3 py-1.5 text-xs border border-border rounded-md hover:bg-surface-hover"
              >
                取消
              </button>
              <button
                onClick={handleImport}
                disabled={loading || !importJson.trim()}
                className="px-3 py-1.5 text-xs bg-primary text-primary-foreground rounded-md hover:bg-primary/90 disabled:opacity-50 flex items-center gap-1"
              >
                {loading ? <Loader2 className="w-3 h-3 animate-spin" /> : <Upload className="w-3 h-3" />}
                导入
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Edit Dialog */}
      {editingServer && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
          <div className="w-[600px] h-[480px] bg-surface rounded-xl shadow-2xl border border-border flex flex-col">
            <div className="flex items-center justify-between px-4 py-3 border-b border-border">
              <h3 className="text-sm font-medium flex items-center gap-2">
                <Edit2 className="w-4 h-4" />
                编辑 MCP Server: {editingServer.name}
              </h3>
              <button onClick={() => { setEditingServer(null); setEditJson(""); setEditError(null); }} className="p-1 hover:bg-surface-hover rounded">
                <X className="w-4 h-4" />
              </button>
            </div>

            <div className="flex-1 overflow-hidden">
              <Editor
                height="100%"
                defaultLanguage="json"
                value={editJson}
                onChange={(v) => setEditJson(v || "")}
                theme="vs-dark"
                options={{
                  minimap: { enabled: false },
                  fontSize: 12,
                  lineNumbers: "on",
                  scrollBeyondLastLine: false,
                  wordWrap: "on",
                  tabSize: 2,
                  automaticLayout: true,
                  padding: { top: 8, bottom: 8 },
                }}
              />
            </div>

            {editError && (
              <div className="mx-4 mt-2 p-2 bg-red-500/10 border border-red-500/30 rounded text-xs text-red-600 dark:text-red-400">
                {editError}
              </div>
            )}

            <div className="flex items-center justify-end gap-2 px-4 py-3 border-t border-border">
              <button
                onClick={() => { setEditingServer(null); setEditJson(""); setEditError(null); }}
                className="px-3 py-1.5 text-xs border border-border rounded-md hover:bg-surface-hover"
              >
                取消
              </button>
              <button
                onClick={handleSaveEdit}
                disabled={loading || !editJson.trim()}
                className="px-3 py-1.5 text-xs bg-primary text-primary-foreground rounded-md hover:bg-primary/90 disabled:opacity-50 flex items-center gap-1"
              >
                {loading ? <Loader2 className="w-3 h-3 animate-spin" /> : <Edit2 className="w-3 h-3" />}
                保存
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Server List */}
      <div className="space-y-2">
        {loading ? (
          <div className="text-center py-8 text-xs text-content-secondary">
            <Loader2 className="w-4 h-4 mx-auto mb-2 animate-spin" />
            加载中...
          </div>
        ) : mcpServers.length === 0 && !showImportDialog ? (
          <div className="text-center py-12 text-xs text-content-secondary border border-dashed border-border rounded-lg">
            <Server className="w-8 h-8 mx-auto mb-2 opacity-40" />
            <p>还没有配置 MCP Server</p>
            <p className="text-2xs mt-1">点击 "导入 JSON" 开始添加</p>
          </div>
        ) : (
          mcpServers.map((server) => (
            <div
              key={server.id}
              className={`p-3 border rounded-lg transition-colors ${
                server.enabled
                  ? "bg-surface-secondary border-border hover:border-primary/30"
                  : "bg-surface-secondary/50 border-border/50 opacity-70"
              }`}
            >
              <div className="flex items-start justify-between gap-2">
                {/* Server Info */}
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2 mb-1">
                    <span className="font-medium text-xs truncate">{server.name}</span>
                    <span className={`px-1.5 py-0.5 text-2xs rounded font-mono ${
                      server.serverType === "stdio"
                        ? "bg-blue-500/20 text-blue-600 dark:text-blue-400"
                        : server.serverType === "streamableHttp"
                        ? "bg-purple-500/20 text-purple-600 dark:text-purple-400"
                        : server.serverType === "sse"
                        ? "bg-orange-500/20 text-orange-600 dark:text-orange-400"
                        : "bg-green-500/20 text-green-600 dark:text-green-400"
                    }`}>
                      {server.serverType}
                    </span>
                    {!server.enabled && (
                      <span className="px-1.5 py-0.5 text-2xs rounded bg-gray-500/20 text-gray-500">
                        已禁用
                      </span>
                    )}
                  </div>
                  <div className="text-2xs text-content-secondary font-mono truncate" title={getServerDisplay(server)}>
                    {getServerDisplay(server)}
                  </div>
                  {server.env && Object.keys(server.env).length > 0 && (
                    <div className="text-2xs text-content-tertiary mt-0.5">
                      env: {Object.keys(server.env).join(", ")}
                    </div>
                  )}
                </div>

                {/* Actions */}
                <div className="flex items-center gap-1 shrink-0">
                  {/* Toggle */}
                  <button
                    onClick={() => handleToggle(server)}
                    disabled={togglingId === server.id}
                    className={`p-1.5 rounded transition-colors ${
                      server.enabled
                        ? "hover:bg-green-500/10 text-green-600 dark:text-green-400"
                        : "hover:bg-gray-500/10 text-gray-500"
                    }`}
                    title={server.enabled ? "禁用" : "启用"}
                  >
                    {togglingId === server.id ? (
                      <Loader2 className="w-3.5 h-3.5 animate-spin" />
                    ) : server.enabled ? (
                      <Power className="w-3.5 h-3.5" />
                    ) : (
                      <PowerOff className="w-3.5 h-3.5" />
                    )}
                  </button>

                  {/* Test / Refresh Tools */}
                  <button
                    onClick={() => handleTest(server)}
                    disabled={testingIds.has(server.id) || !server.enabled}
                    className="p-1.5 hover:bg-blue-500/10 rounded text-blue-600 dark:text-blue-400 disabled:opacity-40"
                    title={testResults[server.id] ? "刷新工具列表" : "测试连接"}
                  >
                    {testingIds.has(server.id) ? (
                      <Loader2 className="w-3.5 h-3.5 animate-spin" />
                    ) : (
                      <RefreshCw className="w-3.5 h-3.5" />
                    )}
                  </button>

                  {/* Edit */}
                  <button
                    onClick={() => {
                      setEditingServer(server);
                      setEditJson(serverToEditJson(server));
                      setEditError(null);
                    }}
                    className="p-1.5 hover:bg-surface-hover rounded"
                    title="编辑"
                  >
                    <Edit2 className="w-3.5 h-3.5" />
                  </button>

                  {/* Delete */}
                  <button
                    onClick={() => handleDelete(server.id)}
                    className="p-1.5 hover:bg-red-500/10 rounded text-red-600 dark:text-red-400"
                    title="删除"
                  >
                    <Trash2 className="w-3.5 h-3.5" />
                  </button>
                </div>
              </div>

              {/* Test Error */}
              {testErrors[server.id] && (
                <div className="mt-2 p-2 bg-red-500/10 border border-red-500/30 rounded text-2xs text-red-600 dark:text-red-400">
                  <span className="font-medium">测试失败: </span>{testErrors[server.id]}
                </div>
              )}

              {/* Test Results */}
              {(() => {
                const tools = testResults[server.id];
                if (!tools || tools.length === 0) return null;
                return (
                  <div className="mt-2 pt-2 border-t border-border/50">
                    <div className="flex items-center gap-1 text-2xs font-medium mb-2 text-green-600 dark:text-green-400">
                      <Play className="w-3 h-3" />
                      可用工具 ({tools.length})
                    </div>
                    <div className="space-y-1 max-h-32 overflow-y-auto">
                      {tools.map((tool, idx) => (
                        <div
                          key={idx}
                          className="text-2xs bg-surface p-2 rounded border border-border"
                        >
                          <div className="font-mono text-blue-600 dark:text-blue-400">
                            {tool.name}
                          </div>
                          {tool.description && (
                            <div className="text-content-secondary mt-0.5 leading-relaxed">
                              {tool.description}
                            </div>
                          )}
                        </div>
                      ))}
                    </div>
                  </div>
                );
              })()}
            </div>
          ))
        )}
      </div>
    </div>
  );
}
