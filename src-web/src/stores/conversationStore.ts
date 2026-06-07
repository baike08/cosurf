import { create } from "zustand";
import type { Conversation, Message } from "@cosurf/shared";
import { generateId } from "@/lib/utils";
import { invoke } from "@/lib/tauri";
import { listen } from "@tauri-apps/api/event";
import { useSettingsStore } from "./settingsStore";

interface BackendConversation {
  id: string;
  title: string;
  isPinned: boolean;
  modelId?: string;
  messageCount: number;
  createdAt: string;
  updatedAt: string;
}

interface BackendMessage {
  id: string;
  conversationId: string;
  role: string;
  content: string;
  thinkingContent: string;
  status: string;
  attachments: any[];
  createdAt: string;
  updatedAt: string;
  feedback: string;
}

interface ConversationState {
  conversations: Conversation[];
  activeConversationId: string | null;
  messages: Message[];
  isStreaming: boolean;
  isLoading: boolean;

  loadConversations: () => Promise<void>;
  loadMessages: (conversationId: string) => Promise<void>;
  setActiveConversation: (id: string) => Promise<void>;
  createConversation: () => Promise<void>;
  deleteConversation: (id: string) => Promise<void>;
  sendMessage: (content: string) => Promise<void>;
  stopStreaming: () => Promise<void>;
  appendStreamDelta: (delta: string, isThinking?: boolean) => void;
  finishStream: () => void;
  checkAndUpdateTitle: () => Promise<void>;
}

export const useConversationStore = create<ConversationState>((set, get) => ({
  conversations: [],
  activeConversationId: null,
  messages: [],
  isStreaming: false,
  isLoading: false,

  loadConversations: async () => {
    try {
      console.log('[ConversationStore] 🔄 Loading conversations...');
      set({ isLoading: true });
      const convs = await invoke<BackendConversation[]>("list_conversations");
      console.log('[ConversationStore] ✅ Loaded conversations:', convs.length);
      set({
        conversations: convs as Conversation[],
        activeConversationId: convs[0]?.id ?? null,
        isLoading: false,
      });
      
      // 加载第一个对话的消息
      if (convs[0]) {
        console.log('[ConversationStore] 📥 Loading messages for first conversation:', convs[0].id);
        await get().loadMessages(convs[0].id);
      }
    } catch (error) {
      console.error("[ConversationStore] ❌ Failed to load conversations:", error);
      set({ isLoading: false });
    }
  },

  loadMessages: async (conversationId) => {
    try {
      console.log('[ConversationStore] 📥 Loading messages for conversation:', conversationId);
      const msgs = await invoke<BackendMessage[]>("list_messages", {
        conversationId,
      });
      console.log('[ConversationStore] ✅ Loaded messages:', msgs.length);
      set({ messages: msgs as Message[] });
    } catch (error) {
      console.error("[ConversationStore] ❌ Failed to load messages:", error);
    }
  },

  setActiveConversation: async (id) => {
    set({ activeConversationId: id });
    await get().loadMessages(id);
  },

  createConversation: async () => {
    try {
      const conv = await invoke<BackendConversation>("create_conversation", {
        request: {
          title: "新对话",
          isPinned: false,
        },
      });
      
      set((state) => ({
        conversations: [conv as Conversation, ...state.conversations],
        activeConversationId: conv.id,
        messages: [],
      }));
    } catch (error) {
      console.error("Failed to create conversation:", error);
    }
  },

  deleteConversation: async (id) => {
    try {
      await invoke("delete_conversation", { id });
      set((state) => {
        const filtered = state.conversations.filter((c) => c.id !== id);
        const newActiveId =
          state.activeConversationId === id
            ? (filtered[0]?.id ?? null)
            : state.activeConversationId;
        return {
          conversations: filtered,
          activeConversationId: newActiveId,
          messages:
            state.activeConversationId === id ? [] : state.messages,
        };
      });
    } catch (error) {
      console.error("Failed to delete conversation:", error);
    }
  },

  sendMessage: async (content) => {
    console.log('[ConversationStore] 📤 Sending message:', content.slice(0, 50));
    let { activeConversationId, messages } = get();
    if (!content.trim()) {
      console.warn('[ConversationStore] ⚠️ Empty content, skipping');
      return;
    }

    // 如果没有活跃对话，自动创建一个
    if (!activeConversationId) {
      console.log('[ConversationStore] 🆕 No active conversation, creating new one...');
      try {
        const conv = await invoke<BackendConversation>("create_conversation", {
          request: {
            title: "新对话", // 使用默认标题，后续会用AI生成
          },
        });
        console.log('[ConversationStore] ✅ Created new conversation:', conv.id);
        set((state) => ({
          conversations: [conv as Conversation, ...state.conversations],
          activeConversationId: conv.id,
        }));
        activeConversationId = conv.id;
        messages = [];
      } catch (error) {
        console.error("[ConversationStore] ❌ Failed to auto-create conversation:", error);
        return;
      }
    }

    // 先添加用户消息到本地
    const now = new Date().toISOString();
    const userMsg: Message = {
      id: generateId(),
      conversationId: activeConversationId!,
      role: "user",
      content: content.trim(),
      thinkingContent: "",
      status: "complete",
      attachments: [],
      createdAt: now,
      updatedAt: now,
      feedback: "",
    };

    const assistantMsg: Message = {
      id: generateId(),
      conversationId: activeConversationId!,
      role: "assistant",
      content: "",
      thinkingContent: "",
      status: "streaming",
      attachments: [],
      createdAt: now,
      updatedAt: now,
      feedback: "",
    };

    set({
      messages: [...messages, userMsg, assistantMsg],
      isStreaming: true,
    });

    // 先设置监听器，再调用后端（确保不会错过事件）
    try {
      // 保存当前的 conversation ID，避免闭包问题
      const currentConvId = activeConversationId!;

      const unlistenChunk = await listen<{
        conversation_id: string;
        message_id: string;
        delta: string;
        is_thinking: boolean;
        done: boolean;
      }>("ai:stream-chunk", (event) => {
        const payload = event.payload;
        if (payload.conversation_id === currentConvId) {
          console.log('[ConversationStore] 📨 Received chunk:', {
            deltaLength: payload.delta.length,
            isThinking: payload.is_thinking,
            done: payload.done,
            deltaPreview: payload.delta.slice(0, 50)
          });
          
          get().appendStreamDelta(payload.delta, payload.is_thinking);
          
          if (payload.done) {
            console.log('[ConversationStore] ✅ Stream finished');
            get().finishStream();
            unlistenChunk();
            unlistenError();
          }
        }
      });

      // 监听错误事件
      const unlistenError = await listen<{
        conversation_id: string;
        error: string;
      }>("ai:stream-error", (event) => {
        const payload = event.payload;
        if (payload.conversation_id === currentConvId) {
          get().appendStreamDelta(`\n\n❌ ${payload.error}`);
          get().finishStream();
          unlistenError();
          unlistenChunk();
        }
      });

      // 监听工具调用开始
      const _unlistenToolStart = await listen<{
        conversation_id: string;
        message_id: string;
        tool_name: string;
        arguments: any;
      }>("ai:tool-call-start", (event) => {
        const payload = event.payload;
        if (payload.conversation_id === currentConvId) {
          console.log('[Tool Call Start]', payload.tool_name, payload.arguments);
          // 不在消息内容中显示工具执行通知，避免重复
          // get().appendStreamDelta(`\n\n🔧 正在执行: ${payload.tool_name}...`);
        }
      });
      void _unlistenToolStart; // Mark as intentionally unused

      // 监听工具调用结果
      const _unlistenToolResult = await listen<{
        conversation_id: string;
        message_id: string;
        tool_name: string;
        output: string;
        success: boolean;
      }>("ai:tool-call-result", (event) => {
        const payload = event.payload;
        if (payload.conversation_id === currentConvId) {
          console.log('[Tool Call Result]', payload.tool_name, payload.output);
          // 不在消息内容中显示工具执行结果，避免重复
          // if (payload.success) {
          //   get().appendStreamDelta(`\n\n✅ ${payload.tool_name} 执行成功: ${payload.output}`);
          // } else {
          //   get().appendStreamDelta(`\n\n❌ ${payload.tool_name} 执行失败: ${payload.output}`);
          // }
        }
      });
      void _unlistenToolResult; // Mark as intentionally unused

      // 调用后端 AI 服务
      console.log('[ConversationStore] 🚀 Calling backend send_chat_message...');
      await invoke("send_chat_message", {
        conversationId: currentConvId,
        content: content.trim(),
      });
      console.log('[ConversationStore] ✅ Backend call completed');
    } catch (error) {
      console.error("[ConversationStore] ❌ Failed to send message to backend:", error);
      // 更健壮的错误处理
      let errorMsg: string;
      if (error instanceof Error) {
        errorMsg = error.message;
      } else if (typeof error === 'string') {
        errorMsg = error;
      } else if (typeof error === 'object' && error !== null) {
        // 尝试从对象中提取错误信息
        const errObj = error as any;
        errorMsg = errObj.message || errObj.error || JSON.stringify(error);
      } else {
        errorMsg = String(error);
      }
      console.error('[ConversationStore] 💥 Error details:', { errorMsg, errorType: typeof error });
      get().appendStreamDelta(`\n\n❌ 发送失败: ${errorMsg}\n\n请检查：\n1. 模型是否已配置并激活\n2. API Key 是否正确\n3. 网络连接是否正常`);
      get().finishStream();
    }
  },

  stopStreaming: async () => {
    try {
      await invoke("stop_generation");
      get().finishStream();
    } catch (error) {
      console.error("Failed to stop generation:", error);
    }
  },

  appendStreamDelta: (delta, isThinking = false) => {
    set((state) => {
      const msgs = [...state.messages];
      const last = msgs[msgs.length - 1];
      console.log('[appendStreamDelta]', {
        hasLast: !!last,
        role: last?.role,
        status: last?.status,
        isThinking,
        deltaLength: delta.length,
        deltaPreview: delta.slice(0, 50)
      });
      if (last && last.role === "assistant") {
        if (last.status === "streaming") {
          if (isThinking) {
            msgs[msgs.length - 1] = {
              ...last,
              thinkingContent: last.thinkingContent + delta,
            };
            console.log('[appendStreamDelta] ✅ Appended to thinkingContent, new length:', (msgs[msgs.length - 1]?.thinkingContent || '').length);
          } else {
            msgs[msgs.length - 1] = {
              ...last,
              content: last.content + delta,
            };
            console.log('[appendStreamDelta] ✅ Appended to content, new length:', (msgs[msgs.length - 1]?.content || '').length);
          }
        } else if (last.status === "complete" && delta.length > 0) {
          // 如果消息是 complete 状态但有新内容（工具调用后的新一轮），重置为 streaming
          console.log('[appendStreamDelta] 🔄 Resetting message status from complete to streaming');
          if (isThinking) {
            msgs[msgs.length - 1] = {
              ...last,
              status: "streaming",
              thinkingContent: last.thinkingContent + delta,
            };
          } else {
            msgs[msgs.length - 1] = {
              ...last,
              status: "streaming",
              content: last.content + delta,
            };
          }
          console.log('[appendStreamDelta] ✅ Reset and appended, new status:', msgs[msgs.length - 1]?.status);
        } else {
          console.warn('[appendStreamDelta] ❌ Skipped - status is not streaming:', last.status, 'delta length:', delta.length);
        }
      } else {
        console.warn('[appendStreamDelta] ❌ Skipped - conditions not met');
      }
      return { messages: msgs };
    });
  },

  finishStream: () => {
    set((state) => {
      const msgs = [...state.messages];
      const last = msgs[msgs.length - 1];
      if (last && last.role === "assistant") {
        msgs[msgs.length - 1] = { ...last, status: "complete" };
      }
      return { messages: msgs, isStreaming: false };
    });
    
    // 检查是否需要更新会话标题
    get().checkAndUpdateTitle();
  },

  checkAndUpdateTitle: async () => {
    const { activeConversationId, messages, conversations } = get();
    if (!activeConversationId) return;
    
    // 获取用户名称
    const userName = useSettingsStore.getState().settings.userName || "CoCo";
    
    // 计算对话轮数（用户+助手算一轮）
    const userMessages = messages.filter(m => m.role === "user" && m.status === "complete");
    const roundCount = userMessages.length;
    
    // 找到当前会话
    const currentConv = conversations.find(c => c.id === activeConversationId);
    if (!currentConv) return;
    
    // 如果超过3轮且标题还是"新对话"或第一条消息（未AI生成），则生成新标题
    const firstUserMessage = messages.find(m => m.role === "user");
    const needsUpdate = (
      currentConv.title === "新对话" || // 标题仍是默认的"新对话"
      (firstUserMessage && (
        currentConv.title === firstUserMessage.content || // 标题等于第一条消息
        (firstUserMessage.content.length > 20 && Math.abs(currentConv.title.length - firstUserMessage.content.length) < 10)
      ))
    );
    
    if (roundCount >= 1 && needsUpdate) {
      try {
        // 提取前几条消息用于生成标题
        const contextMessages = messages.slice(0, 6).map(m => 
          `${m.role === "user" ? userName : "AI"}: ${m.content}`
        ).join("\n");
        
        console.log(`检测到需要更新会话标题（已${roundCount}轮对话）`);
        
        // 调用 AI 生成标题
        const response = await invoke<string>("generate_conversation_title", {
          context: contextMessages,
        });
        
        // 更新会话标题
        await invoke("update_conversation", {
          id: activeConversationId,
          request: {
            title: response,
          },
        });
        
        // 更新本地状态
        set((state) => ({
          conversations: state.conversations.map(c => 
            c.id === activeConversationId ? { ...c, title: response } : c
          ),
        }));
        
        console.log(`会话标题已自动更新为: ${response}`);
      } catch (error) {
        console.error("Failed to update conversation title:", error);
      }
    }
  },
}));
