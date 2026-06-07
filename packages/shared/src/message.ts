export type MessageRole = "user" | "assistant" | "system";

export type MessageStatus = "pending" | "streaming" | "complete" | "error";

export interface MessageAttachment {
  id: string;
  type: "webpage" | "selection" | "file" | "image";
  name: string;
  content?: string;
  filePath?: string;
  mimeType?: string;
}

export interface Message {
  id: string;
  conversationId: string;
  role: MessageRole;
  content: string;
  thinkingContent: string;
  status: MessageStatus;
  attachments: MessageAttachment[];
  createdAt: string;
  updatedAt: string;
  /** User feedback: "" | "like" | "dislike" */
  feedback: string;
}

export interface StreamChunk {
  conversationId: string;
  messageId: string;
  delta: string;
  isThinking: boolean;
  done: boolean;
}
