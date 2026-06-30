import { useConversationStore } from "@/stores/conversationStore";
import { useSettingsStore } from "@/stores/settingsStore";
import { Sparkles, Zap, Wrench } from "lucide-react";

/**
 * SessionStatusBar 组件
 * 
 * 借鉴 Codex TUI 的状态栏设计，提供：
 * - Token 使用量估算
 * - 当前模型信息
 * - 工具调用统计
 * - 实时状态指示
 */
export function SessionStatusBar() {
  const { messages, isStreaming } = useConversationStore();
  const activeModel = useSettingsStore(s => s.models.find(m => m.id === s.activeModelId));
  
  // 估算 Token 数量（简化版：按字符数估算）
  const estimatedTokens = estimateTokens(messages);
  
  return (
    <div className="flex items-center justify-between px-3 py-1.5 bg-gray-50 border-t text-xs text-gray-600">
      {/* 左侧信息 */}
      <div className="flex items-center gap-4">
        {/* 模型信息 */}
        <div className="flex items-center gap-1.5" title="当前使用的模型">
          <Sparkles className="w-3 h-3 text-brand-500" />
          <span className="font-medium">{activeModel?.name || '未设置'}</span>
        </div>

        {/* Token 估算 */}
        <div className="flex items-center gap-1.5" title="估算的 Token 数量">
          <Zap className="w-3 h-3 text-yellow-500" />
          <span>~{estimatedTokens.toLocaleString()} tokens</span>
        </div>
      </div>

      {/* 右侧状态指示 */}
      <div className="flex items-center gap-2">
        {isStreaming ? (
          <>
            <span className="relative flex h-2 w-2">
              <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-blue-400 opacity-75"></span>
              <span className="relative inline-flex rounded-full h-2 w-2 bg-blue-500"></span>
            </span>
            <span className="text-blue-600 font-medium">正在生成...</span>
          </>
        ) : (
          <>
            <span className="inline-flex rounded-full h-2 w-2 bg-green-500"></span>
            <span className="text-green-600 font-medium">就绪</span>
          </>
        )}
      </div>
    </div>
  );
}

/**
 * 估算消息的 Token 数量
 * 
 * 简化算法：
 * - 英文：约 4 字符/token
 * - 中文：约 1.5 字符/token
 */
function estimateTokens(messages: Array<{ role: string; content?: string }>): number {
  let totalChars = 0;

  for (const msg of messages) {
    if (msg.content) {
      // 区分中英文
      const chineseChars = (msg.content.match(/[\u4e00-\u9fa5]/g) || []).length;
      const otherChars = msg.content.length - chineseChars;
      
      // 中文约 1.5 字符/token，其他约 4 字符/token
      totalChars += Math.ceil(chineseChars / 1.5) + Math.ceil(otherChars / 4);
    }
  }

  return totalChars;
}

/**
 * 使用示例：
 * 
 * ```tsx
 * // 在 AIPanel 底部添加
 * <div className="flex flex-col h-full">
 *   <MessagesList />
 *   <InputArea />
 *   <SessionStatusBar />  // 状态栏
 * </div>
 * ```
 */
