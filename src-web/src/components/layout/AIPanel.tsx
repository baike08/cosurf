import { useState, useRef, useEffect } from "react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import {
  ChevronDown,
  ChevronLeft,
  ChevronRight,
  Plus,
  Sparkles,
  Paperclip,
  Send,
  Square,
  FileText,
  Globe,
  Image,
  History,
  ThumbsUp,
  ThumbsDown,
  Copy,
  Check,
} from "lucide-react";
import { useConversationStore } from "@/stores/conversationStore";
import { useSettingsStore } from "@/stores/settingsStore";
import { useUIStore } from "@/stores/uiStore";
import { useTabStore } from "@/stores/tabStore";
import { IconButton } from "@/components/ui/IconButton";
import { Tooltip } from "@/components/ui/Tooltip";
import { cn, getDomain } from "@/lib/utils";
import { invoke } from "@/lib/tauri";
import type { Message } from "@cosurf/shared";

export function AIPanel() {
  const aiPanelOpen = useUIStore((s) => s.aiPanelOpen);
  const aiPanelWidth = useUIStore((s) => s.aiPanelWidth);
  const setAIPanelWidth = useUIStore((s) => s.setAIPanelWidth);
  const toggleAIPanel = useUIStore((s) => s.toggleAIPanel);
  const setSidebarPanel = useUIStore((s) => s.setSidebarPanel);
  
  // 订阅整个 conversation store 以确保检测到所有变化
  const { messages, isStreaming, stopStreaming, sendMessage, createConversation } = useConversationStore();
  
  const models = useSettingsStore((s) => s.models);
  const activeModelId = useSettingsStore((s) => s.activeModelId);
  const setActiveModel = useSettingsStore((s) => s.setActiveModel);
  const userName = useSettingsStore((s) => s.settings.userName || "CoCo");

  const [input, setInput] = useState("");
  const [showModelPicker, setShowModelPicker] = useState(false);
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  const activeModel = models.find((m) => m.id === activeModelId);

  // 自动滚动到底部
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages]);

  useEffect(() => {
    if (textareaRef.current) {
      textareaRef.current.style.height = "auto";
      textareaRef.current.style.height =
        Math.min(textareaRef.current.scrollHeight, 120) + "px";
    }
  }, [input]);

  const handleSend = () => {
    if (!input.trim() || isStreaming) return;
    sendMessage(input);
    setInput("");
    if (textareaRef.current) {
      textareaRef.current.style.height = "auto";
    }
  };

  if (!aiPanelOpen) return null;

  return (
    <div className="flex h-full">
      {/* 拖拽手柄 - 位于左侧 */}
      <div
        className="w-1 bg-transparent hover:bg-brand-500/20 cursor-col-resize shrink-0 transition-colors"
        onMouseDown={(e) => {
          e.preventDefault();
          const startX = e.clientX;
          const startWidth = aiPanelWidth;

          const handleMouseMove = (moveEvent: MouseEvent) => {
            // 向左拖动（X减小）增加宽度，向右拖动（X增大）减小宽度
            const delta = startX - moveEvent.clientX;
            setAIPanelWidth(startWidth + delta);
          };

          const handleMouseUp = () => {
            document.removeEventListener("mousemove", handleMouseMove);
            document.removeEventListener("mouseup", handleMouseUp);
            document.body.style.cursor = "";
            document.body.style.userSelect = "";
          };

          document.addEventListener("mousemove", handleMouseMove);
          document.addEventListener("mouseup", handleMouseUp);
          document.body.style.cursor = "col-resize";
          document.body.style.userSelect = "none";
        }}
      />

      {/* AI 面板主体 */}
      <div
        className="flex flex-col bg-surface border-l border-border h-full"
        style={{ width: aiPanelWidth }}
      >
      {/* 顶部控制区域 - 固定不滚动 */}
      <div className="shrink-0 sticky top-0 z-10 bg-surface">
        <PanelHeader
          modelName={activeModel?.name ?? "选择模型"}
          isStreaming={isStreaming}
          onToggle={() => toggleAIPanel()}
          onNewConversation={() => createConversation()}
          onShowModels={() => setShowModelPicker(!showModelPicker)}
          onShowHistory={() => setSidebarPanel("conversations")}
        />

        {showModelPicker && (
          <ModelPicker
            models={models}
            activeModelId={activeModelId}
            onSelect={(id) => {
              setActiveModel(id);
              setShowModelPicker(false);
            }}
          />
        )}
      </div>

      {/* 消息列表 - 可滚动区域 */}
      <div className="flex-1 overflow-y-auto px-4 py-2">
        {messages.length === 0 ? (
          <EmptyState />
        ) : (
          <MessageList messages={messages} userName={userName} />
        )}
        <div ref={messagesEndRef} />
      </div>

      <ChatInput
        input={input}
        setInput={setInput}
        onSend={handleSend}
        onStop={stopStreaming}
        isStreaming={isStreaming}
        textareaRef={textareaRef}
      />
      </div>
    </div>
  );
}

function PanelHeader({
  modelName,
  isStreaming,
  onToggle,
  onNewConversation,
  onShowModels,
  onShowHistory,
}: {
  modelName: string;
  isStreaming: boolean;
  onToggle: () => void;
  onNewConversation: () => void;
  onShowModels: () => void;
  onShowHistory: () => void;
}) {
  return (
    <div className="flex items-center justify-between px-3 h-9 border-b border-border shrink-0">
      <div className="flex items-center gap-2">
        <Sparkles className="w-4 h-4 text-brand-500" />
        <button
          onClick={onShowModels}
          className="text-xs font-medium hover:text-brand-600 transition-colors flex items-center gap-1"
        >
          {modelName}
          <ChevronDown className="w-3 h-3" />
        </button>
        {isStreaming && (
          <span className="flex items-center gap-1 text-2xs text-brand-500">
            <span className="w-1.5 h-1.5 rounded-full bg-brand-500 animate-pulse" />
            生成中...
          </span>
        )}
      </div>
      <div className="flex items-center gap-0.5">
        <Tooltip label="历史会话">
          <IconButton size="sm" onClick={onShowHistory}>
            <History />
          </IconButton>
        </Tooltip>
        <Tooltip label="新对话">
          <IconButton size="sm" onClick={onNewConversation}>
            <Plus />
          </IconButton>
        </Tooltip>
        <Tooltip label="收起">
          <IconButton size="sm" onClick={onToggle}>
            <ChevronLeft />
          </IconButton>
        </Tooltip>
      </div>
    </div>
  );
}

function ModelPicker({
  models,
  activeModelId,
  onSelect,
}: {
  models: { id: string; name: string; provider: string; isLocal: boolean }[];
  activeModelId: string;
  onSelect: (id: string) => void;
}) {
  return (
    <div className="border-b border-border bg-surface-secondary p-2 animate-slide-up">
      <div className="grid grid-cols-1 gap-1">
        {models.map((model) => (
          <button
            key={model.id}
            onClick={() => onSelect(model.id)}
            className={cn(
              "flex items-center gap-2 px-3 py-2 rounded-md text-left text-xs transition-colors",
              model.id === activeModelId
                ? "bg-brand-600 text-white"
                : "hover:bg-surface-hover text-content",
            )}
          >
            <Sparkles className="w-3.5 h-3.5 shrink-0" />
            <div className="flex-1 min-w-0">
              <div className="font-medium">{model.name}</div>
              <div
                className={cn(
                  "text-2xs",
                  model.id === activeModelId
                    ? "text-white/70"
                    : "text-content-tertiary",
                )}
              >
                {model.provider} {model.isLocal ? "· 本地" : ""}
              </div>
            </div>
          </button>
        ))}
      </div>
    </div>
  );
}

function EmptyState() {
  return (
    <div className="flex flex-col items-center justify-center h-full text-center gap-3 py-8">
      <div className="w-12 h-12 rounded-2xl bg-brand-500/10 flex items-center justify-center">
        <Sparkles className="w-6 h-6 text-brand-500" />
      </div>
      <div>
        <div className="text-sm font-medium">你好，我是 CoSurf AI</div>
        <div className="text-xs text-content-secondary mt-1">
          可以帮你总结网页、回答问题、操作页面
        </div>
      </div>
      <div className="flex flex-wrap gap-1.5 justify-center mt-2">
        {["总结这个页面", "翻译为英文", "提取要点", "解释代码"].map(
          (hint) => (
            <span
              key={hint}
              className="px-2.5 py-1 rounded-full text-2xs bg-surface-secondary text-content-secondary border border-border cursor-pointer hover:bg-surface-hover transition-colors"
            >
              {hint}
            </span>
          ),
        )}
      </div>
    </div>
  );
}

function MessageList({ messages, userName }: { messages: Message[]; userName: string }) {
  // 使用消息内容的组合作为 key，确保内容变化时重新渲染
  const lastMsg = messages[messages.length - 1];
  const listKey = lastMsg 
    ? `${messages.length}-${lastMsg.id}-${lastMsg.content.length}-${lastMsg.thinkingContent?.length || 0}`
    : 'empty';
  
  return (
    <div className="space-y-3" key={listKey}>
      {messages.map((msg) => (
        <MessageItem key={msg.id} message={msg} userName={userName} />
      ))}
    </div>
  );
}

function ThinkingBlock({
  content,
  isStreaming,
}: {
  content: string;
  isStreaming: boolean;
}) {
  const [collapsed, setCollapsed] = useState(!isStreaming);
  const scrollRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (isStreaming && !collapsed && scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [content, isStreaming, collapsed]);

  // Auto-collapse when streaming finishes (only if user hasn't manually expanded)
  useEffect(() => {
    if (!isStreaming && content && collapsed) {
      // Keep collapsed state as is - don't auto-expand
      // User can manually expand to view thinking process
    }
  }, [isStreaming]);

  if (!content) return null;

  return (
    <div className="mb-2">
      <button
        onClick={() => setCollapsed(!collapsed)}
        className="w-full flex flex-col items-start gap-1.5 text-xs text-purple-500 hover:text-purple-600 transition-colors"
      >
        <div className="flex items-center gap-1.5">
          {collapsed ? (
            <ChevronRight className="w-3.5 h-3.5" />
          ) : (
            <ChevronDown className="w-3.5 h-3.5" />
          )}
          <span className="font-medium">
            {isStreaming ? "思考中..." : "思考过程"}
          </span>
          {isStreaming && (
            <span className="w-1.5 h-1.5 rounded-full bg-purple-400 animate-pulse" />
          )}
        </div>
        {collapsed && (
          <div className="text-2xs text-purple-400 ml-5 whitespace-nowrap overflow-hidden text-ellipsis max-w-full">
            {content.replace(/\n/g, ' ').length > 80 
              ? content.replace(/\n/g, ' ').slice(0, 80) + "..." 
              : content.replace(/\n/g, ' ')}
          </div>
        )}
      </button>
      {!collapsed && (
        <div
          ref={scrollRef}
          className="max-h-[200px] overflow-y-auto mt-1.5 ml-5 text-xs text-purple-600/80 whitespace-pre-wrap break-words leading-relaxed"
        >
          {content}
        </div>
      )}
    </div>
  );
}

function MessageItem({ message, userName }: { message: Message; userName: string }) {
  const isUser = message.role === "user";
  const isStreaming = message.status === "streaming";
  const thinking = isUser ? "" : message.thinkingContent;
  const response = message.content;
  const [copied, setCopied] = useState(false);
  const feedback = message.feedback || "";

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(response);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch {
      // fallback
      const ta = document.createElement("textarea");
      ta.value = response;
      document.body.appendChild(ta);
      ta.select();
      document.execCommand("copy");
      document.body.removeChild(ta);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  const handleFeedback = async (type: "like" | "dislike") => {
    const newFeedback = feedback === type ? "" : type;
    try {
      const updated = await invoke<Message>("set_message_feedback", {
        id: message.id,
        feedback: newFeedback,
      });
      // 更新本地状态
      const store = useConversationStore.getState();
      store.messages = store.messages.map((m) =>
        m.id === message.id ? { ...m, feedback: updated.feedback } : m
      );
      useConversationStore.setState({ messages: store.messages });
    } catch (e) {
      console.error("Failed to set feedback:", e);
    }
  };

  return (
    <div className={cn("flex gap-2", isUser ? "flex-row-reverse" : "")}>
      <div
        className={cn(
          "w-6 h-6 rounded-lg flex items-center justify-center shrink-0 text-2xs font-bold",
          isUser
            ? "bg-brand-600 text-white"
            : "bg-surface-tertiary text-content-secondary",
        )}
      >
        {isUser ? userName.slice(0, 2) : "AI"}
      </div>
      <div className="max-w-[80%] group">
        <div
          className={cn(
            "rounded-xl px-3 py-2 text-xs leading-relaxed",
            isUser
              ? "bg-brand-600 text-white rounded-tr-sm"
              : "bg-surface-secondary text-content rounded-tl-sm",
          )}
        >
          {message.attachments.length > 0 && (
            <div className="flex flex-wrap gap-1 mb-2">
              {message.attachments.map((att) => (
                <div
                  key={att.id}
                  className={cn(
                    "flex items-center gap-1 px-2 py-0.5 rounded text-2xs",
                    isUser ? "bg-white/20" : "bg-surface-tertiary",
                  )}
                >
                  {att.type === "webpage" && <Globe className="w-3 h-3" />}
                  {att.type === "file" && <FileText className="w-3 h-3" />}
                  {att.type === "image" && <Image className="w-3 h-3" />}
                  {att.name}
                </div>
              ))}
            </div>
          )}

          {/* Thinking block inside the same bubble */}
          {thinking && (
            <ThinkingBlock content={thinking} isStreaming={isStreaming} />
          )}

          {/* Response content with Markdown rendering */}
          <div className="markdown-content break-words">
            {isUser ? (
              <div className="whitespace-pre-wrap">{response}</div>
            ) : (
              <ReactMarkdown
                remarkPlugins={[remarkGfm]}
                components={{
                  // 自定义样式 - 超紧凑版
                  h1: ({ children }) => <h1 className="text-xs font-bold mt-1 mb-0.5 leading-tight">{children}</h1>,
                  h2: ({ children }) => <h2 className="text-xs font-bold mt-0.5 mb-0.5 leading-tight">{children}</h2>,
                  h3: ({ children }) => <h3 className="text-xs font-semibold mt-0.5 mb-0.5 leading-tight">{children}</h3>,
                  p: ({ children }) => <p className="mb-0.5 last:mb-0 leading-relaxed">{children}</p>,
                  ul: ({ children }) => <ul className="list-disc list-inside mb-0.5 ml-2 space-y-0">{children}</ul>,
                  ol: ({ children }) => <ol className="list-decimal list-inside mb-0.5 ml-2 space-y-0">{children}</ol>,
                  li: ({ children }) => <li className="leading-relaxed">{children}</li>,
                  code: ({ className, children, ...props }) => {
                    const isInline = !className;
                    return isInline ? (
                      <code className="bg-surface-tertiary px-1 py-0.5 rounded text-xs font-mono" {...props}>
                        {children}
                      </code>
                    ) : (
                      <code className="block bg-surface-tertiary px-2 py-1 rounded text-xs font-mono my-0.5 overflow-x-auto leading-relaxed" {...props}>
                        {children}
                      </code>
                    );
                  },
                  pre: ({ children }) => <pre className="my-0.5">{children}</pre>,
                  blockquote: ({ children }) => (
                    <blockquote className="border-l-2 border-border pl-2 py-0.5 my-0.5 text-content-secondary italic leading-relaxed">
                      {children}
                    </blockquote>
                  ),
                  a: ({ children, href }) => {
                    const addTab = useTabStore.getState().addTab;
                    
                    const handleClick = (e: React.MouseEvent) => {
                      e.preventDefault();
                      if (href) {
                        console.log('[AIPanel] 🖱️ Link clicked:', href);
                        const newTabId = addTab(href, getDomain(href));
                        console.log('[AIPanel] ✅ New tab created:', newTabId);
                      }
                    };
                    
                    return (
                      <a 
                        href={href} 
                        onClick={handleClick}
                        className="text-brand-500 hover:text-brand-600 underline cursor-pointer" 
                      >
                        {children}
                      </a>
                    );
                  },
                  table: ({ children }) => (
                    <div className="overflow-x-auto my-0.5">
                      <table className="border-collapse text-xs w-full">{children}</table>
                    </div>
                  ),
                  th: ({ children }) => (
                    <th className="border border-border px-1.5 py-0.5 bg-surface-tertiary font-semibold text-left">{children}</th>
                  ),
                  td: ({ children }) => (
                    <td className="border border-border px-1.5 py-0.5">{children}</td>
                  ),
                  strong: ({ children }) => <strong className="font-semibold">{children}</strong>,
                  em: ({ children }) => <em className="italic">{children}</em>,
                  hr: () => <hr className="my-1 border-border" />,
                }}
              >
                {response}
              </ReactMarkdown>
            )}
            {isStreaming && (
              <span className="inline-block w-1.5 h-3.5 bg-current animate-pulse ml-0.5 align-text-bottom" />
            )}
          </div>
        </div>

        {/* Action buttons for assistant messages (shown on hover or when feedback is active) */}
        {!isUser && !isStreaming && response && (
          <div className={cn(
            "flex items-center gap-0.5 mt-1 ml-1 transition-opacity",
            feedback ? "opacity-100" : "opacity-0 group-hover:opacity-100"
          )}>
            <button
              onClick={() => handleFeedback("like")}
              className={cn(
                "p-1 rounded transition-colors",
                feedback === "like"
                  ? "text-green-500 bg-green-500/10"
                  : "text-content-tertiary hover:text-green-500 hover:bg-green-500/10"
              )}
              title="点赞"
            >
              <ThumbsUp className="w-3 h-3" />
            </button>
            <button
              onClick={() => handleFeedback("dislike")}
              className={cn(
                "p-1 rounded transition-colors",
                feedback === "dislike"
                  ? "text-red-500 bg-red-500/10"
                  : "text-content-tertiary hover:text-red-500 hover:bg-red-500/10"
              )}
              title="点踩"
            >
              <ThumbsDown className="w-3 h-3" />
            </button>
            <button
              onClick={handleCopy}
              className={cn(
                "p-1 rounded transition-colors",
                copied
                  ? "text-brand-500 bg-brand-500/10"
                  : "text-content-tertiary hover:text-content-secondary hover:bg-surface-hover"
              )}
              title={copied ? "已复制" : "复制"}
            >
              {copied ? <Check className="w-3 h-3" /> : <Copy className="w-3 h-3" />}
            </button>
          </div>
        )}
      </div>
    </div>
  );
}

function ChatInput({
  input,
  setInput,
  onSend,
  onStop,
  isStreaming,
  textareaRef,
}: {
  input: string;
  setInput: (v: string) => void;
  onSend: () => void;
  onStop: () => void;
  isStreaming: boolean;
  textareaRef: React.RefObject<HTMLTextAreaElement | null>;
}) {
  return (
    <div className="shrink-0 border-t border-border p-2">
      <div className="flex items-end gap-1.5 bg-surface-secondary rounded-xl px-2 py-1.5">
        <Tooltip label="添加附件">
          <IconButton size="sm" className="mb-0.5">
            <Paperclip />
          </IconButton>
        </Tooltip>

        <textarea
          ref={textareaRef as React.Ref<HTMLTextAreaElement>}
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter" && !e.shiftKey) {
              e.preventDefault();
              onSend();
            }
          }}
          rows={1}
          className="flex-1 bg-transparent text-xs text-content outline-none resize-none placeholder:text-content-tertiary max-h-[120px] py-1"
          placeholder="输入消息，Shift+Enter 换行..."
        />

        {isStreaming ? (
          <Tooltip label="停止生成">
            <IconButton 
              size="sm" 
              variant="solid" 
              className="mb-0.5"
              onClick={onStop}
            >
              <Square />
            </IconButton>
          </Tooltip>
        ) : (
          <Tooltip label="发送">
            <IconButton
              size="sm"
              variant="solid"
              className="mb-0.5"
              disabled={!input.trim()}
              onClick={onSend}
            >
              <Send />
            </IconButton>
          </Tooltip>
        )}
      </div>
    </div>
  );
}
