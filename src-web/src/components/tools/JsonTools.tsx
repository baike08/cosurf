import { useState, useCallback, useEffect, useMemo } from "react";
import {
  FileJson,
  FileEdit,
  FileCheck,
  Copy,
  Trash2,
  Download,
  Upload,
  AlertCircle,
  CheckCircle2,
  Minimize2,
  Maximize2,
  ArrowRight,
  Search,
} from "lucide-react";
import { cn } from "@/lib/utils";

// JSON 解析器
export function JsonParser() {
  const [input, setInput] = useState("");
  const [output, setOutput] = useState("");
  const [error, setError] = useState("");
  const [indent, setIndent] = useState(2);
  const [copied, setCopied] = useState(false);
  const [searchQuery, setSearchQuery] = useState("");
  const [copiedValue, setCopiedValue] = useState<string | null>(null);
  const [parsedJson, setParsedJson] = useState<any>(null);
  const [viewMode, setViewMode] = useState<"text" | "tree">("text");
  const [contextMenu, setContextMenu] = useState<{ x: number; y: number; value: any; path: string } | null>(null);

  const handleParse = useCallback(() => {
    if (!input.trim()) {
      setOutput("");
      setError("");
      setParsedJson(null);
      return;
    }
    try {
      const parsed = JSON.parse(input);
      setParsedJson(parsed);
      setOutput(JSON.stringify(parsed, null, indent));
      setError("");
    } catch (e: any) {
      setError(e.message);
      setOutput("");
      setParsedJson(null);
    }
  }, [input, indent]);

  const handleCopy = useCallback(() => {
    if (output) {
      navigator.clipboard.writeText(output);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  }, [output]);

  const handleClear = useCallback(() => {
    setInput("");
    setOutput("");
    setError("");
    setSearchQuery("");
    setParsedJson(null);
    setCopiedValue(null);
  }, []);

  // 复制单个值
  const copyValue = useCallback((value: any, key?: string) => {
    let textToCopy: string;
    if (typeof value === "object" && value !== null) {
      textToCopy = JSON.stringify(value, null, 2);
    } else {
      textToCopy = String(value);
    }
    
    navigator.clipboard.writeText(textToCopy);
    setCopiedValue(key || textToCopy.substring(0, 50));
    setTimeout(() => setCopiedValue(null), 2000);
  }, []);

  // 显示右键菜单
  const showContextMenu = useCallback((e: React.MouseEvent, value: any, path: string) => {
    e.preventDefault();
    setContextMenu({ x: e.clientX, y: e.clientY, value, path });
  }, []);

  // 关闭右键菜单
  useEffect(() => {
    const handleClick = () => setContextMenu(null);
    if (contextMenu) {
      document.addEventListener('click', handleClick);
      return () => document.removeEventListener('click', handleClick);
    }
  }, [contextMenu]);

  // 复制为紧凑JSON
  const copyAsCompact = useCallback((value: any) => {
    const text = JSON.stringify(value);
    navigator.clipboard.writeText(text);
    setCopiedValue("compact");
    setTimeout(() => setCopiedValue(null), 2000);
    setContextMenu(null);
  }, []);

  // 复制为格式化JSON
  const copyAsFormatted = useCallback((value: any) => {
    const text = JSON.stringify(value, null, 2);
    navigator.clipboard.writeText(text);
    setCopiedValue("formatted");
    setTimeout(() => setCopiedValue(null), 2000);
    setContextMenu(null);
  }, []);

  // 高亮搜索结果
  const highlightedOutput = useMemo(() => {
    if (!searchQuery.trim() || !output) return output;
    
    const query = searchQuery.toLowerCase();
    let result = output;
    
    // 匹配 key 和 value（包括字符串）
    const regex = new RegExp(`("[^"]*${query.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')}[^"]*")`, 'gi');
    result = result.replace(regex, '<mark class="bg-yellow-500/30 text-yellow-700 dark:text-yellow-300 rounded px-0.5">$1</mark>');
    
    return result;
  }, [output, searchQuery]);

  // JSON 树形视图组件
  const JsonTreeView = ({ data, level = 0, path = "" }: { data: any; level?: number; path?: string }) => {
    if (data === null) return <span className="text-purple-500">null</span>;
    
    if (typeof data !== "object") {
      const displayValue = typeof data === "string" ? `"${data}"` : String(data);
      const colorClass = typeof data === "string" ? "text-green-600 dark:text-green-400" : 
                        typeof data === "number" ? "text-blue-600 dark:text-blue-400" :
                        typeof data === "boolean" ? "text-orange-600 dark:text-orange-400" : "";
      
      return (
        <span className="group relative inline-flex items-center gap-1">
          <span className={colorClass}>{displayValue}</span>
          <button
            onClick={() => copyValue(data, path)}
            onContextMenu={(e) => showContextMenu(e, data, path)}
            className="opacity-0 group-hover:opacity-100 transition-opacity p-1 rounded hover:bg-brand-500/20 text-content-tertiary hover:text-brand-500"
            title={`复制 ${path || "value"}`}
          >
            <Copy className="w-3 h-3" />
          </button>
          {copiedValue === (path || String(data).substring(0, 50)) && (
            <span className="absolute -top-6 left-0 bg-brand-600 text-white text-2xs px-2 py-1 rounded whitespace-nowrap">
              已复制!
            </span>
          )}
        </span>
      );
    }
    
    const isArray = Array.isArray(data);
    const entries = Object.entries(data);
    const isEmpty = entries.length === 0;
    
    if (isEmpty) {
      return (
        <span 
          className="group relative inline-flex items-center gap-1 cursor-pointer"
          onContextMenu={(e) => showContextMenu(e, data, path)}
        >
          {isArray ? "[]" : "{}"}
          <button
            onClick={() => copyValue(data, path)}
            className="opacity-0 group-hover:opacity-100 transition-opacity p-1 rounded hover:bg-brand-500/20 text-content-tertiary hover:text-brand-500"
            title={`复制 ${path || "empty object/array"}`}
          >
            <Copy className="w-3 h-3" />
          </button>
        </span>
      );
    }
    
    return (
      <div className="group relative">
        <span className="text-content-tertiary">{isArray ? "[" : "{"}</span>
        {/* 对象/数组的复制按钮 */}
        <button
          onClick={() => copyValue(data, path)}
          onContextMenu={(e) => showContextMenu(e, data, path)}
          className="opacity-0 group-hover:opacity-100 absolute -right-8 top-0 p-1 rounded hover:bg-brand-500/20 text-content-tertiary hover:text-brand-500 transition-opacity"
          title={`复制整个${isArray ? '数组' : '对象'} (${entries.length}项)`}
        >
          <Copy className="w-3 h-3" />
        </button>
        <div className="ml-4">
          {entries.map(([key, value], index) => (
            <div key={index} className="flex items-start">
              {!isArray && (
                <span className="text-red-600 dark:text-red-400 mr-2">"{key}":</span>
              )}
              <JsonTreeView 
                data={value} 
                level={level + 1}
                path={isArray ? `${path}[${index}]` : `${path}${path ? "." : ""}${key}`}
              />
              {index < entries.length - 1 && <span className=",">,</span>}
            </div>
          ))}
        </div>
        <span className="text-content-tertiary">{isArray ? "]" : "}"}</span>
      </div>
    );
  };

  const handleDownload = useCallback(() => {
    if (output) {
      const blob = new Blob([output], { type: "application/json" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = "formatted.json";
      a.click();
      URL.revokeObjectURL(url);
    }
  }, [output]);

  const handleFileUpload = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (file) {
      const reader = new FileReader();
      reader.onload = (ev) => {
        const text = ev.target?.result as string;
        setInput(text);
      };
      reader.readAsText(file);
    }
  }, []);

  useEffect(() => {
    const timer = setTimeout(() => {
      handleParse();
    }, 300);
    return () => clearTimeout(timer);
  }, [input, indent, handleParse]);

  // 渲染右键菜单
  const renderContextMenu = () => {
    if (!contextMenu) return null;
    
    const isObject = typeof contextMenu.value === "object" && contextMenu.value !== null;
    const isArray = Array.isArray(contextMenu.value);
    const itemCount = isObject ? Object.keys(contextMenu.value).length : 0;
    
    return (
      <div
        className="fixed bg-surface border border-border rounded-lg shadow-xl py-1 z-[9999] min-w-[200px]"
        style={{ left: contextMenu.x, top: contextMenu.y }}
      >
        {/* 显示路径 */}
        <div className="px-3 py-2 text-xs text-content-tertiary border-b border-border/50">
          <span className="font-medium">{isArray ? '数组' : '对象'}:</span>
          <span className="ml-1 font-mono truncate max-w-[180px] inline-block align-middle">
            {contextMenu.path || '(root)'}
          </span>
          {isObject && (
            <span className="ml-1 text-2xs">({itemCount}项)</span>
          )}
        </div>
        
        {/* 复制选项 */}
        <button
          onClick={() => copyValue(contextMenu.value, contextMenu.path)}
          className="w-full px-3 py-2 text-left text-sm hover:bg-surface-hover flex items-center gap-2"
        >
          <Copy className="w-3.5 h-3.5" />
          复制值
        </button>
        
        {isObject && (
          <>
            <button
              onClick={() => copyAsCompact(contextMenu.value)}
              className="w-full px-3 py-2 text-left text-sm hover:bg-surface-hover flex items-center gap-2"
            >
              <Copy className="w-3.5 h-3.5" />
              复制为紧凑JSON
            </button>
            <button
              onClick={() => copyAsFormatted(contextMenu.value)}
              className="w-full px-3 py-2 text-left text-sm hover:bg-surface-hover flex items-center gap-2"
            >
              <Copy className="w-3.5 h-3.5" />
              复制为格式化JSON
            </button>
          </>
        )}
      </div>
    );
  };

  return (
    <div className="h-full flex flex-col bg-surface-secondary">
      {/* 工具栏 */}
      <div className="flex items-center gap-2 px-4 py-3 border-b border-border bg-surface">
        <FileJson className="w-5 h-5 text-amber-500" />
        <h2 className="text-base font-medium text-content">JSON 解析</h2>
        
        {/* 搜索框 */}
        <div className="relative ml-4">
          <Search className="absolute left-2 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-content-tertiary" />
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder="搜索 key 或 value..."
            className="h-8 pl-8 pr-6 rounded-md bg-surface-secondary border border-border text-sm text-content outline-none focus:border-brand-500 w-48 transition-colors"
          />
          {searchQuery && (
            <button
              onClick={() => setSearchQuery("")}
              className="absolute right-1.5 top-1/2 -translate-y-1/2 w-5 h-5 flex items-center justify-center rounded-full hover:bg-content-tertiary/10 text-content-tertiary"
            >
              ×
            </button>
          )}
        </div>
        
        <div className="flex-1" />
        <div className="flex items-center gap-1.5">
          <label className="text-xs text-content-tertiary">缩进:</label>
          <select
            value={indent}
            onChange={(e) => setIndent(Number(e.target.value))}
            className="h-8 px-2.5 rounded-md bg-surface-secondary border border-border text-sm text-content outline-none"
          >
            <option value={2}>2 空格</option>
            <option value={4}>4 空格</option>
            <option value={8}>8 空格</option>
          </select>
        </div>
        <button
          onClick={handleCopy}
          disabled={!output}
          className="h-8 px-3 rounded-md text-sm border border-border hover:bg-surface-hover text-content-secondary hover:text-content transition-colors disabled:opacity-40 flex items-center gap-1.5"
        >
          <Copy className="w-3.5 h-3.5" />
          {copied ? "已复制" : "复制"}
        </button>
        <button
          onClick={handleDownload}
          disabled={!output}
          className="h-8 px-3 rounded-md text-sm border border-border hover:bg-surface-hover text-content-secondary hover:text-content transition-colors disabled:opacity-40 flex items-center gap-1.5"
        >
          <Download className="w-3.5 h-3.5" />
          下载
        </button>
        <label className="h-8 px-3 rounded-md text-sm border border-border hover:bg-surface-hover text-content-secondary hover:text-content transition-colors cursor-pointer flex items-center gap-1.5">
          <Upload className="w-3.5 h-3.5" />
          上传
          <input type="file" accept=".json,.txt" onChange={handleFileUpload} className="hidden" />
        </label>
        <button
          onClick={handleClear}
          className="h-8 px-3 rounded-md text-sm border border-border hover:bg-red-500/10 hover:text-red-500 hover:border-red-500/30 text-content-secondary transition-colors flex items-center gap-1.5"
        >
          <Trash2 className="w-3.5 h-3.5" />
          清空
        </button>
      </div>

      {/* 错误提示 */}
      {error && (
        <div className="flex items-center gap-2 px-4 py-2.5 bg-red-500/10 border-b border-red-500/20 text-red-500 text-sm">
          <AlertCircle className="w-4 h-4 shrink-0" />
          <span>{error}</span>
        </div>
      )}

      {/* 主内容区 */}
      <div className="flex-1 flex overflow-hidden">
        {/* 输入区 */}
        <div className="flex-1 flex flex-col border-r border-border">
          <div className="px-3 py-2 bg-surface-secondary/50 border-b border-border/50 text-xs text-content-tertiary">
            输入 JSON
          </div>
          <textarea
            value={input}
            onChange={(e) => setInput(e.target.value)}
            placeholder='在此粘贴 JSON 数据，例如：\n{"name": "CoSurf", "version": "1.0"}'
            className="flex-1 w-full p-4 bg-transparent text-sm text-content font-mono resize-none outline-none placeholder:text-content-tertiary"
            spellCheck={false}
          />
        </div>

        {/* 箭头 */}
        <div className="w-8 flex items-center justify-center bg-surface-secondary/30">
          <ArrowRight className="w-4 h-4 text-content-tertiary" />
        </div>

        {/* 输出区 */}
        <div className="flex-1 flex flex-col">
          <div className="px-3 py-2 bg-surface-secondary/50 border-b border-border/50 text-xs text-content-tertiary flex items-center justify-between">
            <div className="flex items-center gap-2">
              <span>格式化结果</span>
              {output && !error && (
                <CheckCircle2 className="w-3.5 h-3.5 text-green-500" />
              )}
              {searchQuery && highlightedOutput !== output && (
                <span className="text-amber-600 dark:text-amber-400 text-2xs ml-1">已高亮匹配项</span>
              )}
            </div>
            
            {/* 视图切换按钮 */}
            {parsedJson && (
              <div className="flex items-center gap-1">
                <button
                  onClick={() => setViewMode("text")}
                  className={cn(
                    "px-2 py-1 rounded text-2xs transition-colors",
                    viewMode === "text" ? "bg-brand-500/20 text-brand-600 dark:text-brand-400" : "hover:bg-surface-hover"
                  )}
                >
                  文本
                </button>
                <button
                  onClick={() => setViewMode("tree")}
                  className={cn(
                    "px-2 py-1 rounded text-2xs transition-colors",
                    viewMode === "tree" ? "bg-brand-500/20 text-brand-600 dark:text-brand-400" : "hover:bg-surface-hover"
                  )}
                >
                  树形
                </button>
              </div>
            )}
          </div>
          
          {/* 内容区域 */}
          {viewMode === "tree" && parsedJson ? (
            <div className="flex-1 overflow-auto p-4 font-mono text-sm">
              <JsonTreeView data={parsedJson} />
            </div>
          ) : searchQuery && highlightedOutput !== output ? (
            <pre
              className="flex-1 w-full p-4 bg-transparent text-sm text-content font-mono overflow-auto whitespace-pre-wrap"
              dangerouslySetInnerHTML={{ __html: highlightedOutput }}
            />
          ) : (
            <textarea
              value={output}
              readOnly
              className="flex-1 w-full p-4 bg-transparent text-sm text-content font-mono resize-none outline-none"
              placeholder="格式化后的 JSON 将显示在这里..."
              spellCheck={false}
            />
          )}
        </div>
      </div>
      
      {/* 右键菜单 */}
      {renderContextMenu()}
    </div>
  );
}

// JSON 编辑器
export function JsonEditor() {
  const [jsonText, setJsonText] = useState('{\n  "name": "CoSurf",\n  "version": "1.0.0",\n  "features": ["AI", "Browser", "Tools"]\n}');
  const [error, setError] = useState("");
  const [saved, setSaved] = useState(false);
  const [copied, setCopied] = useState(false);

  const validate = useCallback((text: string) => {
    if (!text.trim()) {
      setError("");
      return;
    }
    try {
      JSON.parse(text);
      setError("");
    } catch (e: any) {
      setError(e.message);
    }
  }, []);

  useEffect(() => {
    const timer = setTimeout(() => validate(jsonText), 300);
    return () => clearTimeout(timer);
  }, [jsonText, validate]);

  const handleFormat = useCallback(() => {
    try {
      const parsed = JSON.parse(jsonText);
      setJsonText(JSON.stringify(parsed, null, 2));
      setSaved(true);
      setTimeout(() => setSaved(false), 2000);
    } catch (e: any) {
      setError(e.message);
    }
  }, [jsonText]);

  const handleMinify = useCallback(() => {
    try {
      const parsed = JSON.parse(jsonText);
      setJsonText(JSON.stringify(parsed));
    } catch (e: any) {
      setError(e.message);
    }
  }, [jsonText]);

  const handleCopy = useCallback(() => {
    navigator.clipboard.writeText(jsonText);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  }, [jsonText]);

  const handleAddField = useCallback(() => {
    try {
      const parsed = JSON.parse(jsonText);
      if (typeof parsed === "object" && !Array.isArray(parsed)) {
        parsed["newField"] = "newValue";
        setJsonText(JSON.stringify(parsed, null, 2));
      }
    } catch (e: any) {
      setError(e.message);
    }
  }, [jsonText]);

  return (
    <div className="h-full flex flex-col bg-surface-secondary">
      {/* 工具栏 */}
      <div className="flex items-center gap-2 px-4 py-3 border-b border-border bg-surface">
        <FileEdit className="w-5 h-5 text-blue-500" />
        <h2 className="text-base font-medium text-content">JSON 编辑</h2>
        <div className="flex-1" />
        {error ? (
          <div className="flex items-center gap-1.5 text-red-500 text-sm">
            <AlertCircle className="w-3.5 h-3.5" />
            <span className="max-w-[300px] truncate">{error}</span>
          </div>
        ) : saved ? (
          <div className="flex items-center gap-1.5 text-green-500 text-sm">
            <CheckCircle2 className="w-3.5 h-3.5" />
            <span>已格式化</span>
          </div>
        ) : (
          <div className="flex items-center gap-1.5 text-green-500 text-sm">
            <CheckCircle2 className="w-3.5 h-3.5" />
            <span>格式正确</span>
          </div>
        )}
        <button
          onClick={handleFormat}
          className="h-8 px-3 rounded-md text-sm bg-brand-600 text-white hover:bg-brand-700 transition-colors flex items-center gap-1.5"
        >
          <Maximize2 className="w-3.5 h-3.5" />
          格式化
        </button>
        <button
          onClick={handleMinify}
          className="h-8 px-3 rounded-md text-sm border border-border hover:bg-surface-hover text-content-secondary hover:text-content transition-colors flex items-center gap-1.5"
        >
          <Minimize2 className="w-3.5 h-3.5" />
          压缩
        </button>
        <button
          onClick={handleAddField}
          className="h-8 px-3 rounded-md text-sm border border-border hover:bg-surface-hover text-content-secondary hover:text-content transition-colors"
        >
          + 添加字段
        </button>
        <button
          onClick={handleCopy}
          className="h-8 px-3 rounded-md text-sm border border-border hover:bg-surface-hover text-content-secondary hover:text-content transition-colors flex items-center gap-1.5"
        >
          <Copy className="w-3.5 h-3.5" />
          {copied ? "已复制" : "复制"}
        </button>
      </div>

      {/* 编辑器 */}
      <div className="flex-1 flex flex-col overflow-hidden">
        <div className="px-3 py-2 bg-surface-secondary/50 border-b border-border/50 text-xs text-content-tertiary flex items-center justify-between">
          <span>JSON 编辑器</span>
          <span>
            行数: {jsonText.split("\n").length} · 字符: {jsonText.length}
          </span>
        </div>
        <textarea
          value={jsonText}
          onChange={(e) => setJsonText(e.target.value)}
          className="flex-1 w-full p-4 bg-transparent text-sm text-content font-mono resize-none outline-none"
          spellCheck={false}
        />
      </div>
    </div>
  );
}

// JSON 验证器
export function JsonValidator() {
  const [input, setInput] = useState("");
  const [results, setResults] = useState<
    Array<{ type: "success" | "error" | "info"; message: string }>
  >([]);
  const [isValid, setIsValid] = useState<boolean | null>(null);

  const handleValidate = useCallback(() => {
    if (!input.trim()) {
      setResults([]);
      setIsValid(null);
      return;
    }

    const newResults: Array<{
      type: "success" | "error" | "info";
      message: string;
    }> = [];

    // 基本检查
    const trimmed = input.trim();
    if (!trimmed.startsWith("{") && !trimmed.startsWith("[")) {
      newResults.push({
        type: "error",
        message: "JSON 必须以 { 或 [ 开头",
      });
      setResults(newResults);
      setIsValid(false);
      return;
    }

    // 括号匹配检查
    let braceCount = 0;
    let bracketCount = 0;
    let inString = false;
    let escape = false;

    for (let i = 0; i < trimmed.length; i++) {
      const char = trimmed[i];
      if (escape) {
        escape = false;
        continue;
      }
      if (char === "\\") {
        escape = true;
        continue;
      }
      if (char === '"') {
        inString = !inString;
        continue;
      }
      if (!inString) {
        if (char === "{") braceCount++;
        if (char === "}") braceCount--;
        if (char === "[") bracketCount++;
        if (char === "]") bracketCount--;
      }
    }

    if (braceCount !== 0) {
      newResults.push({
        type: "error",
        message: `花括号不匹配: ${braceCount > 0 ? "缺少 " + braceCount + " 个 }" : "多出 " + -braceCount + " 个 }"}`,
      });
    }
    if (bracketCount !== 0) {
      newResults.push({
        type: "error",
        message: `方括号不匹配: ${bracketCount > 0 ? "缺少 " + bracketCount + " 个 ]" : "多出 " + -bracketCount + " 个 ]"}`,
      });
    }

    // 尝试解析
    try {
      const parsed = JSON.parse(trimmed);
      newResults.push({
        type: "success",
        message: "JSON 格式验证通过",
      });

      // 类型信息
      const type = Array.isArray(parsed) ? "Array" : typeof parsed;
      newResults.push({
        type: "info",
        message: `根类型: ${type}`,
      });

      if (typeof parsed === "object" && parsed !== null) {
        const keys = Object.keys(parsed);
        newResults.push({
          type: "info",
          message: `顶层字段数: ${keys.length}`,
        });
        keys.forEach((key) => {
          const val = parsed[key];
          const valType = Array.isArray(val) ? "array" : typeof val;
          newResults.push({
            type: "info",
            message: `  "${key}": ${valType}${val === null ? " (null)" : ""}`,
          });
        });
      }

      if (Array.isArray(parsed)) {
        newResults.push({
          type: "info",
          message: `数组长度: ${parsed.length}`,
        });
      }

      // 大小信息
      const bytes = new TextEncoder().encode(trimmed).length;
      newResults.push({
        type: "info",
        message: `数据大小: ${bytes < 1024 ? bytes + " B" : (bytes / 1024).toFixed(1) + " KB"}`,
      });

      setIsValid(true);
    } catch (e: any) {
      newResults.push({
        type: "error",
        message: `解析错误: ${e.message}`,
      });
      setIsValid(false);
    }

    setResults(newResults);
  }, [input]);

  useEffect(() => {
    const timer = setTimeout(() => {
      handleValidate();
    }, 300);
    return () => clearTimeout(timer);
  }, [input, handleValidate]);

  return (
    <div className="h-full flex flex-col bg-surface-secondary">
      {/* 工具栏 */}
      <div className="flex items-center gap-2 px-4 py-3 border-b border-border bg-surface">
        <FileCheck className="w-5 h-5 text-green-500" />
        <h2 className="text-base font-medium text-content">JSON 检查</h2>
        <div className="flex-1" />
        {isValid === true && (
          <div className="flex items-center gap-1.5 text-green-500 text-sm font-medium">
            <CheckCircle2 className="w-4 h-4" />
            验证通过
          </div>
        )}
        {isValid === false && (
          <div className="flex items-center gap-1.5 text-red-500 text-sm font-medium">
            <AlertCircle className="w-4 h-4" />
            格式错误
          </div>
        )}
        <button
          onClick={() => {
            setInput("");
            setResults([]);
            setIsValid(null);
          }}
          className="h-8 px-3 rounded-md text-sm border border-border hover:bg-red-500/10 hover:text-red-500 hover:border-red-500/30 text-content-secondary transition-colors flex items-center gap-1.5"
        >
          <Trash2 className="w-3.5 h-3.5" />
          清空
        </button>
      </div>

      {/* 主内容区 */}
      <div className="flex-1 flex overflow-hidden">
        {/* 输入区 */}
        <div className="flex-1 flex flex-col border-r border-border">
          <div className="px-3 py-2 bg-surface-secondary/50 border-b border-border/50 text-xs text-content-tertiary">
            输入 JSON 进行验证
          </div>
          <textarea
            value={input}
            onChange={(e) => setInput(e.target.value)}
            placeholder='在此粘贴需要验证的 JSON 数据...'
            className="flex-1 w-full p-4 bg-transparent text-sm text-content font-mono resize-none outline-none placeholder:text-content-tertiary"
            spellCheck={false}
          />
        </div>

        {/* 结果区 */}
        <div className="flex-1 flex flex-col">
          <div className="px-3 py-2 bg-surface-secondary/50 border-b border-border/50 text-xs text-content-tertiary">
            验证结果
          </div>
          <div className="flex-1 overflow-y-auto p-4 space-y-2">
            {results.length === 0 ? (
              <div className="text-center py-12 text-content-tertiary text-sm">
                输入 JSON 后自动验证
              </div>
            ) : (
              results.map((result, i) => (
                <div
                  key={i}
                  className={cn(
                    "flex items-start gap-2 px-3 py-2.5 rounded-lg text-sm",
                    result.type === "success" &&
                      "bg-green-500/10 text-green-600 border border-green-500/20",
                    result.type === "error" &&
                      "bg-red-500/10 text-red-500 border border-red-500/20",
                    result.type === "info" &&
                      "bg-surface-secondary text-content-secondary border border-border/50"
                  )}
                >
                  {result.type === "success" && (
                    <CheckCircle2 className="w-4 h-4 shrink-0 mt-0.5" />
                  )}
                  {result.type === "error" && (
                    <AlertCircle className="w-4 h-4 shrink-0 mt-0.5" />
                  )}
                  {result.type === "info" && (
                    <span className="w-4 h-4 shrink-0 mt-0.5 text-center text-xs">
                      ℹ
                    </span>
                  )}
                  <span className="font-mono">{result.message}</span>
                </div>
              ))
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
