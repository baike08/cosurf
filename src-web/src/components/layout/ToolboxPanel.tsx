import { useEffect, useRef, useState } from "react";
import {
  X,
  Braces,
  Regex,
  QrCode,
  Lock,
  Diff,
  Search,
  FileJson,
  FileEdit,
  FileCheck,
  Wrench,
} from "lucide-react";
import { useUIStore } from "@/stores/uiStore";
import { useTabStore } from "@/stores/tabStore";
import { cn } from "@/lib/utils";

// 工具定义
interface ToolItem {
  id: string;
  name: string;
  description: string;
  icon: React.ReactNode;
  url: string;
}

// 工具分类
interface ToolCategory {
  id: string;
  name: string;
  icon: React.ReactNode;
  color: string;
  tools: ToolItem[];
}

const TOOL_CATEGORIES: ToolCategory[] = [
  {
    id: "json",
    name: "JSON 工具",
    icon: <Braces className="w-4 h-4" />,
    color: "from-amber-500 to-orange-600",
    tools: [
      {
        id: "json-parser",
        name: "JSON 解析",
        description: "格式化和美化 JSON 数据",
        icon: <FileJson className="w-4 h-4" />,
        url: "cosurf://tools/json-parser",
      },
      {
        id: "json-editor",
        name: "JSON 编辑",
        description: "在线编辑和修改 JSON 数据",
        icon: <FileEdit className="w-4 h-4" />,
        url: "cosurf://tools/json-editor",
      },
      {
        id: "json-validator",
        name: "JSON 检查",
        description: "验证 JSON 格式并定位错误",
        icon: <FileCheck className="w-4 h-4" />,
        url: "cosurf://tools/json-validator",
      },
    ],
  },
  {
    id: "regex",
    name: "正则表达式",
    icon: <Regex className="w-4 h-4" />,
    color: "from-blue-500 to-indigo-600",
    tools: [
      {
        id: "regex-tester",
        name: "正则测试",
        description: "在线测试正则表达式（即将推出）",
        icon: <Regex className="w-4 h-4" />,
        url: "cosurf://tools/regex-tester",
      },
    ],
  },
  {
    id: "qrcode",
    name: "二维码",
    icon: <QrCode className="w-4 h-4" />,
    color: "from-green-500 to-emerald-600",
    tools: [
      {
        id: "qrcode-generator",
        name: "二维码生成",
        description: "生成二维码（即将推出）",
        icon: <QrCode className="w-4 h-4" />,
        url: "cosurf://tools/qrcode-generator",
      },
    ],
  },
  {
    id: "crypto",
    name: "加密/解密",
    icon: <Lock className="w-4 h-4" />,
    color: "from-purple-500 to-violet-600",
    tools: [
      {
        id: "crypto-tool",
        name: "加密解密",
        description: "文本加密与解密（即将推出）",
        icon: <Lock className="w-4 h-4" />,
        url: "cosurf://tools/crypto",
      },
    ],
  },
  {
    id: "diff",
    name: "数据对比",
    icon: <Diff className="w-4 h-4" />,
    color: "from-pink-500 to-rose-600",
    tools: [
      {
        id: "text-diff",
        name: "文本对比",
        description: "对比两段文本差异（即将推出）",
        icon: <Diff className="w-4 h-4" />,
        url: "cosurf://tools/text-diff",
      },
    ],
  },
];

export function ToolboxPanel() {
  const toolboxOpen = useUIStore((s) => s.toolboxOpen);
  const closeToolbox = useUIStore((s) => s.closeToolbox);
  const addTab = useTabStore((s) => s.addTab);
  const panelRef = useRef<HTMLDivElement>(null);
  const [searchQuery, setSearchQuery] = useState("");

  // 点击外部关闭
  useEffect(() => {
    if (!toolboxOpen) return;
    const handleClickOutside = (e: MouseEvent) => {
      if (panelRef.current && !panelRef.current.contains(e.target as Node)) {
        closeToolbox();
      }
    };
    // 延迟添加，避免与打开按钮的点击事件冲突
    const timer = setTimeout(() => {
      document.addEventListener("mousedown", handleClickOutside);
    }, 100);
    return () => {
      clearTimeout(timer);
      document.removeEventListener("mousedown", handleClickOutside);
    };
  }, [toolboxOpen, closeToolbox]);

  // ESC 关闭
  useEffect(() => {
    if (!toolboxOpen) return;
    const handleEsc = (e: KeyboardEvent) => {
      if (e.key === "Escape") closeToolbox();
    };
    window.addEventListener("keydown", handleEsc);
    return () => window.removeEventListener("keydown", handleEsc);
  }, [toolboxOpen, closeToolbox]);

  const handleToolClick = (tool: ToolItem) => {
    addTab(tool.url, tool.name);
    closeToolbox();
  };

  // 搜索过滤
  const filteredCategories = TOOL_CATEGORIES.map((cat) => ({
    ...cat,
    tools: cat.tools.filter(
      (t) =>
        !searchQuery.trim() ||
        t.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
        t.description.toLowerCase().includes(searchQuery.toLowerCase())
    ),
  })).filter((cat) => cat.tools.length > 0);

  if (!toolboxOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-start justify-center pt-16 bg-black/20 backdrop-blur-sm">
      <div
        ref={panelRef}
        className="w-[640px] max-h-[70vh] bg-surface rounded-2xl shadow-2xl border border-border overflow-hidden flex flex-col animate-in fade-in zoom-in-95 duration-200"
      >
        {/* 头部 */}
        <div className="flex items-center gap-3 px-5 py-4 border-b border-border">
          <div className="w-8 h-8 rounded-lg bg-gradient-to-br from-brand-500 to-brand-700 flex items-center justify-center">
            <Wrench className="w-4 h-4 text-white" />
          </div>
          <div className="flex-1">
            <h2 className="text-sm font-semibold text-content">工具箱</h2>
            <p className="text-2xs text-content-tertiary">常用开发工具集合</p>
          </div>
          <button
            onClick={closeToolbox}
            className="w-7 h-7 rounded-lg flex items-center justify-center hover:bg-surface-hover text-content-tertiary hover:text-content transition-colors"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        {/* 搜索框 */}
        <div className="px-5 py-3 border-b border-border/50">
          <div className="flex items-center gap-2 h-9 rounded-lg px-3 bg-surface-secondary border border-border focus-within:border-brand-500 focus-within:ring-2 focus-within:ring-brand-500/20 transition-all">
            <Search className="w-3.5 h-3.5 text-content-tertiary shrink-0" />
            <input
              type="text"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              placeholder="搜索工具..."
              className="flex-1 bg-transparent text-xs text-content outline-none placeholder:text-content-tertiary"
              autoFocus
            />
          </div>
        </div>

        {/* 工具列表 */}
        <div className="flex-1 overflow-y-auto px-5 py-4 space-y-5">
          {filteredCategories.length === 0 ? (
            <div className="text-center py-8 text-content-tertiary text-sm">
              没有找到匹配的工具
            </div>
          ) : (
            filteredCategories.map((category) => (
              <div key={category.id}>
                <div className="flex items-center gap-2 mb-2.5">
                  <div
                    className={cn(
                      "w-6 h-6 rounded-md bg-gradient-to-br flex items-center justify-center text-white",
                      category.color
                    )}
                  >
                    {category.icon}
                  </div>
                  <h3 className="text-xs font-medium text-content-secondary">
                    {category.name}
                  </h3>
                  <div className="flex-1 h-px bg-border/50" />
                </div>
                <div className="grid grid-cols-3 gap-2.5">
                  {category.tools.map((tool) => (
                    <button
                      key={tool.id}
                      onClick={() => handleToolClick(tool)}
                      className="group flex flex-col items-start gap-2 p-3.5 rounded-xl border border-border hover:border-brand-500/50 hover:bg-surface-hover transition-all text-left"
                    >
                      <div className="w-8 h-8 rounded-lg bg-surface-secondary group-hover:bg-brand-500/10 flex items-center justify-center text-content-secondary group-hover:text-brand-500 transition-colors">
                        {tool.icon}
                      </div>
                      <div>
                        <div className="text-xs font-medium text-content group-hover:text-brand-600 transition-colors">
                          {tool.name}
                        </div>
                        <div className="text-2xs text-content-tertiary mt-0.5 line-clamp-2">
                          {tool.description}
                        </div>
                      </div>
                    </button>
                  ))}
                </div>
              </div>
            ))
          )}
        </div>

        {/* 底部 */}
        <div className="px-5 py-2.5 border-t border-border/50 bg-surface-secondary/30">
          <div className="text-2xs text-content-tertiary text-center">
            按 ESC 关闭 · 点击工具在新标签页中打开
          </div>
        </div>
      </div>
    </div>
  );
}
