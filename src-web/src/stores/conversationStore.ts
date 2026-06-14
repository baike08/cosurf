import { create } from "zustand";
import type { Conversation, Message } from "@cosurf/shared";
import { generateId } from "@/lib/utils";
import { db, ai } from "@/lib/api";
import { on } from "@/lib/events";
import { useSettingsStore } from "./settingsStore";

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
      console.log('[ConversationStore] Loading conversations...');
      set({ isLoading: true });
      const convs = await db.listConversations();
      console.log('[ConversationStore] Loaded conversations:', convs.length);
      set({
        conversations: convs as Conversation[],
        activeConversationId: convs[0]?.id ?? null,
        isLoading: false,
      });
      
      if (convs[0]) {
        await get().loadMessages(convs[0].id);
      }
    } catch (error) {
      console.error("[ConversationStore] Failed to load conversations:", error);
      set({ isLoading: false });
    }
  },

  loadMessages: async (conversationId) => {
    try {
      const msgs = await db.listMessages(conversationId);
      set({ messages: msgs as Message[] });
    } catch (error) {
      console.error("[ConversationStore] Failed to load messages:", error);
    }
  },

  setActiveConversation: async (id) => {
    set({ activeConversationId: id });
    await get().loadMessages(id);
  },

  createConversation: async () => {
    try {
      const conv = await db.createConversation("新对话");
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
      await db.deleteConversation(id);
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
    let { activeConversationId, messages } = get();
    if (!content.trim()) return;

    // 如果没有活跃对话，自动创建一个
    if (!activeConversationId) {
      try {
        const conv = await db.createConversation("新对话");
        set((state) => ({
          conversations: [conv as Conversation, ...state.conversations],
          activeConversationId: conv.id,
        }));
        activeConversationId = conv.id;
        messages = [];
      } catch (error) {
        console.error("[ConversationStore] Failed to auto-create conversation:", error);
        return;
      }
    }

    const currentConvId = activeConversationId!;
    const now = new Date().toISOString();

    // 本地创建用户消息
    const userMsg: Message = {
      id: generateId(),
      conversationId: currentConvId,
      role: "user",
      content: content.trim(),
      thinkingContent: "",
      status: "complete",
      attachments: [],
      createdAt: now,
      updatedAt: now,
      feedback: "",
    };

    const tempAssistantId = generateId();
    const assistantMsg: Message = {
      id: tempAssistantId,
      conversationId: currentConvId,
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

    // 先保存消息到 DB，捕获 DB 生成的真实 ID
    let dbAssistantMsgId = tempAssistantId;
    try {
      await db.createMessage(currentConvId, "user", content.trim());
      const createdAssistant = await db.createMessage(currentConvId, "assistant", "");
      if (createdAssistant?.id) {
        dbAssistantMsgId = createdAssistant.id;
        console.log('[ConversationStore] DB assistant message created with ID:', dbAssistantMsgId);
      }
    } catch (e) {
      console.warn("[ConversationStore] Failed to persist messages to DB:", e);
    }

    // 设置流式事件监听
    let unlistenChunk: (() => void) | null = null;
    let unlistenError: (() => void) | null = null;

    try {
      unlistenChunk = on<{
        conversationId: string;
        messageId: string;
        delta: string;
        isThinking: boolean;
        done: boolean;
      }>("ai:stream-chunk", (payload) => {
        console.log('[ConversationStore] stream-chunk received:', payload.conversationId, 'done:', payload.done, 'delta len:', payload.delta?.length);
        if (payload.conversationId === currentConvId) {
          get().appendStreamDelta(payload.delta, payload.isThinking);
          // Rust stream.rs 已通过 save_chunk_to_db 直接保存到 DB，这里不再重复保存
          if (payload.done) {
            get().finishStream();
            db.completeMessage(dbAssistantMsgId).catch(() => {});  // 标记消息完成
            unlistenChunk?.();
            unlistenError?.();
          }
        } else {
          console.warn('[ConversationStore] Ignoring chunk for different conversation:', payload.conversationId, 'expected:', currentConvId);
        }
      });

      unlistenError = on<{
        conversationId: string;
        error: string;
      }>("ai:stream-error", (payload) => {
        if (payload.conversationId === currentConvId) {
          get().appendStreamDelta(`\n\n\u274c ${payload.error}`);
          get().finishStream();
          unlistenError?.();
          unlistenChunk?.();
        }
      });

      // 监听工具调用 (仅日志)
      on<{ conversationId: string; toolName: string }>("ai:tool-call-start", (payload) => {
        if (payload.conversationId === currentConvId) {
          console.log('[Tool Call]', payload.toolName);
        }
      });

      // 准备 AI 请求参数
      const activeModel = useSettingsStore.getState().models.find(
        (m) => m.id === useSettingsStore.getState().activeModelId
      );
      if (!activeModel) {
        throw new Error("没有已激活的模型配置，请先在设置中添加模型");
      }

      // 加载对话中的所有消息用于上下文
      const dbMessages = await db.listMessages(currentConvId);
      const chatMessages = dbMessages.map((m: any) => ({
        role: m.role,
        content: m.content,
      }));

      // 调用 AI 流式对话（使用 DB 生成的真实 ID）
      await ai.sendChat(activeModel, chatMessages, currentConvId, dbAssistantMsgId);
    } catch (error) {
      console.error("[ConversationStore] Failed to send message:", error);
      const errorMsg = error instanceof Error ? error.message : String(error);
      get().appendStreamDelta(`\n\n\u274c 发送失败: ${errorMsg}\n\n请检查：\n1. 模型是否已配置并激活\n2. API Key 是否正确\n3. 网络连接是否正常`);
      get().finishStream();
      unlistenChunk?.();
      unlistenError?.();
    }
  },

  stopStreaming: async () => {
    try {
      await ai.stopGeneration();
      get().finishStream();
    } catch (error) {
      console.error("Failed to stop generation:", error);
    }
  },

  appendStreamDelta: (delta, isThinking = false) => {
    set((state) => {
      const msgs = [...state.messages];
      const last = msgs[msgs.length - 1];
      if (last && last.role === "assistant") {
        if (last.status === "streaming") {
          if (isThinking) {
            msgs[msgs.length - 1] = {
              ...last,
              thinkingContent: last.thinkingContent + delta,
            };
          } else {
            msgs[msgs.length - 1] = {
              ...last,
              content: last.content + delta,
            };
          }
        } else if (last.status === "complete" && delta.length > 0) {
          // 工具调用后的新一轮，重置为 streaming
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
        }
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
        const activeModel = useSettingsStore.getState().models.find(
          (m) => m.id === useSettingsStore.getState().activeModelId
        );
        if (!activeModel) return;

        const response = await ai.generateTitle(contextMessages, activeModel);
        
        // 更新会话标题
        await db.updateConversation(activeConversationId, { title: response });
        
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
