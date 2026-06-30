import { useState } from "react";
import { ChevronDown, ChevronUp, CheckCircle, XCircle, Loader2, Clock } from "lucide-react";
import { cn } from "@/lib/utils";

/**
 * 工具调用状态
 */
export type ToolCallStatus = 
  | 'pending'     // 等待执行
  | 'executing'   // 正在执行
  | 'success'     // 执行成功
  | 'failed';     // 执行失败

/**
 * ToolCallCard 属性
 */
interface ToolCallCardProps {
  toolName: string;
  args: Record<string, any>;
  status: ToolCallStatus;
  result?: string;
  duration?: number;  // 执行耗时（毫秒）
  error?: string;
}

/**
 * 状态配置
 */
const STATUS_CONFIG = {
  pending: {
    icon: Loader2,
    color: 'bg-yellow-100 text-yellow-800 border-yellow-300',
    label: '等待中',
    animate: true,
  },
  executing: {
    icon: Loader2,
    color: 'bg-blue-100 text-blue-800 border-blue-300',
    label: '执行中',
    animate: true,
  },
  success: {
    icon: CheckCircle,
    color: 'bg-green-100 text-green-800 border-green-300',
    label: '成功',
    animate: false,
  },
  failed: {
    icon: XCircle,
    color: 'bg-red-100 text-red-800 border-red-300',
    label: '失败',
    animate: false,
  },
};

/**
 * ToolCallCard 组件
 * 
 * 借鉴 Codex TUI 的工具调用展示设计，提供：
 * - 状态徽章（pending/executing/success/failed）
 * - 可折叠的参数和结果详情
 * - 执行时间显示
 * - 错误信息展示
 */
export function ToolCallCard({
  toolName,
  args,
  status,
  result,
  duration,
  error,
}: ToolCallCardProps) {
  const [isExpanded, setIsExpanded] = useState(false);
  const config = STATUS_CONFIG[status];
  const Icon = config.icon;

  // 格式化执行时间
  const formatDuration = (ms?: number) => {
    if (!ms) return null;
    if (ms < 1000) return `${ms}ms`;
    return `${(ms / 1000).toFixed(1)}s`;
  };

  return (
    <div className="my-2 border rounded-lg overflow-hidden shadow-sm">
      {/* Header */}
      <div 
        className={cn(
          "px-3 py-2 flex items-center justify-between border-b",
          config.color
        )}
      >
        <div className="flex items-center gap-2">
          {/* 状态图标 */}
          <Icon 
            className={cn(
              "w-4 h-4",
              config.animate && "animate-spin"
            )} 
          />
          
          {/* 工具名称 */}
          <span className="font-medium text-sm">{toolName}</span>
          
          {/* 状态标签 */}
          <span className="text-xs opacity-75">{config.label}</span>
        </div>

        <div className="flex items-center gap-2">
          {/* 执行时间 */}
          {duration && (
            <div className="flex items-center gap-1 text-xs opacity-75">
              <Clock className="w-3 h-3" />
              <span>{formatDuration(duration)}</span>
            </div>
          )}

          {/* 展开/收起按钮 */}
          {(args || result || error) && (
            <button
              onClick={() => setIsExpanded(!isExpanded)}
              className="p-1 hover:bg-black/10 rounded transition-colors"
              title={isExpanded ? "收起详情" : "展开详情"}
            >
              {isExpanded ? (
                <ChevronUp className="w-4 h-4" />
              ) : (
                <ChevronDown className="w-4 h-4" />
              )}
            </button>
          )}
        </div>
      </div>

      {/* Collapsible Details */}
      {isExpanded && (args || result || error) && (
        <div className="bg-gray-50 divide-y divide-gray-200">
          {/* 参数 */}
          {args && Object.keys(args).length > 0 && (
            <div className="px-3 py-2">
              <div className="text-xs text-gray-500 mb-1 font-medium">参数：</div>
              <pre className="text-xs bg-white p-2 rounded border overflow-auto max-h-32">
                {JSON.stringify(args, null, 2)}
              </pre>
            </div>
          )}

          {/* 结果 */}
          {result && (
            <div className="px-3 py-2">
              <div className="text-xs text-gray-500 mb-1 font-medium">结果：</div>
              <pre className="text-xs bg-white p-2 rounded border overflow-auto max-h-32 whitespace-pre-wrap">
                {result}
              </pre>
            </div>
          )}

          {/* 错误信息 */}
          {error && (
            <div className="px-3 py-2">
              <div className="text-xs text-red-600 mb-1 font-medium">错误：</div>
              <pre className="text-xs bg-red-50 p-2 rounded border border-red-200 overflow-auto max-h-32 whitespace-pre-wrap">
                {error}
              </pre>
            </div>
          )}
        </div>
      )}
    </div>
  );
}

/**
 * 使用示例：
 * 
 * ```tsx
 * // 在 AIPanel 中使用
 * <ToolCallCard
 *   toolName="browser_navigate"
 *   args={{ url: "https://example.com" }}
 *   status="executing"
 * />
 * 
 * // 执行完成后
 * <ToolCallCard
 *   toolName="web_search"
 *   args={{ query: "React hooks" }}
 *   status="success"
 *   result="Found 10 results..."
 *   duration={1234}
 * />
 * 
 * // 执行失败
 * <ToolCallCard
 *   toolName="file_read"
 *   args={{ path: "/tmp/test.txt" }}
 *   status="failed"
 *   error="File not found"
 * />
 * ```
 */
