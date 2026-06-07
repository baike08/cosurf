export interface Conversation {
  id: string;
  title: string;
  isPinned: boolean;
  modelId: string;
  messageCount: number;
  createdAt: string;
  updatedAt: string;
}

export interface ConversationWithMessages extends Conversation {
  messages: import("./message").Message[];
}
