import { useState, useEffect } from "react";
import { ChevronDown, ChevronUp } from "lucide-react";
import { cn } from "@/lib/utils";

/**
 * Thinking 状态类型
 */
export type ThinkingState = 
  | 'idle'           // 空闲
  | 'thinking'       // 正在思考
  | 'tool_call'      // 正在执行工具
  | 'streaming';     // 正在流式输出

/**
 * ThinkingIndicator 属性
 */
interface ThinkingIndicatorProps {
  state: ThinkingState;
  message?: string;
  content?: string;
  collapsible?: boolean;
}

/**
 * Spinner 动画帧定义（借鉴 Codex TUI 设计）
 */
const SPINNER_ANIMATIONS = {
  thinking: ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'],  // CLASSIC
  tool_call: ['⠉', '⠒', '⣀', '⠒'],                                  // BOUNCE
  streaming: ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'],              // PROGRESS
};

/**
 * 状态对应的提示消息
 */
const STATE_MESSAGES = {
  idle: '',
  thinking: '正在思考...',
  tool_call: '正在执行工具...',
  streaming: '正在生成回答...',
};

/**
 * ThinkingIndicator 组件
 * 
 * 借鉴 Codex TUI 的设计，提供：
 * - 动态 Spinner 动画（根据状态显示不同动画）
 * - 折叠/展开功能
 * - 进度指示
 */
export function ThinkingIndicator({
  state,
  message,
  content,
  collapsible = true,
}: ThinkingIndicatorProps) {
  const [isExpanded, setIsExpanded] = useState(false);
  const [frameIndex, setFrameIndex] = useState(0);

  // 获取当前状态的动画帧
  const animationFrames = SPINNER_ANIMATIONS[state] || [];
  const currentFrame = animationFrames[frameIndex] || '';

  // 启动动画定时器
  useEffect(() => {
    if (state === 'idle') return;

    const frames = SPINNER_ANIMATIONS[state];
    if (!frames || frames.length === 0) return;

    const interval = setInterval(() => {
      setFrameIndex((prev) => (prev + 1) % frames.length);
    }, 80); // 80ms 每帧，流畅的动画

    return () => clearInterval(interval);
  }, [state]);

  // 如果处于空闲状态，不显示
  if (state === 'idle') return null;

  const displayMessage = message || STATE_MESSAGES[state];

  return (
    <div className="my-2 border-l-2 border-brand-300 pl-3">
      {/* Header - 始终显示 */}
      <div className="flex items-center gap-2 text-xs text-gray-500">
        {/* Spinner 动画 */}
        <span 
          className={cn(
            "inline-block w-4 h-4 text-brand-500 font-mono",
            state === 'streaming' && "animate-pulse"
          )}
        >
          {currentFrame}
        </span>
        
        {/* 状态消息 */}
        <span className="flex-1">{displayMessage}</span>

        {/* 折叠按钮（如果有内容且可折叠） */}
        {collapsible && content && (
          <button
            onClick={() => setIsExpanded(!isExpanded)}
            className="p-1 hover:bg-gray-100 rounded transition-colors"
            title={isExpanded ? "收起" : "展开"}
          >
            {isExpanded ? (
              <ChevronUp className="w-3 h-3" />
            ) : (
              <ChevronDown className="w-3 h-3" />
            )}
          </button>
        )}
      </div>

      {/* Content - 可折叠的内容 */}
      {content && isExpanded && (
        <div className="mt-2 p-2 bg-gray-50 rounded text-xs text-gray-700 whitespace-pre-wrap overflow-auto max-h-40">
          {content}
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
 * <ThinkingIndicator 
 *   state={isStreaming ? 'streaming' : 'thinking'}
 *   content={message.thinkingContent}
 * />
 * 
 * // 工具调用时
 * <ThinkingIndicator 
 *   state="tool_call"
 *   message={`正在执行 ${toolName}...`}
 * />
 * ```
 */
