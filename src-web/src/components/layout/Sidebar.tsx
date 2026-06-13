import {
  Bookmark,
  History,
  MessageSquare,
  X,
  Pin,
  Trash2,
  Plus,
  Download,
  Layers,
} from "lucide-react";
import { useUIStore, type SidebarPanel } from "@/stores/uiStore";
import { useConversationStore } from "@/stores/conversationStore";
import { cn, truncate } from "@/lib/utils";
import { IconButton } from "@/components/ui/IconButton";
import { DownloadsPanel } from "@/components/sidebar/DownloadsPanel";
import { TabsPanel } from "@/components/sidebar/TabsPanel";
import { HistoryPanel } from "@/components/sidebar/HistoryPanel";
import { BookmarksPanel } from "@/components/sidebar/BookmarksPanel";

const panelConfig: Record<Exclude<SidebarPanel, "none">, { icon: typeof Bookmark; label: string }> = {
  bookmarks: { icon: Bookmark, label: "书签" },
  history: { icon: History, label: "历史记录" },
  conversations: { icon: MessageSquare, label: "对话历史" },
  downloads: { icon: Download, label: "下载" },
  tabs: { icon: Layers, label: "标签管理" },
};

export function Sidebar() {
  const sidebarOpen = useUIStore((s) => s.sidebarOpen);
  const sidebarPanel = useUIStore((s) => s.sidebarPanel);
  const sidebarWidth = useUIStore((s) => s.sidebarWidth);
  const toggleSidebar = useUIStore((s) => s.toggleSidebar);

  if (!sidebarOpen || sidebarPanel === "none") return null;

  const config = panelConfig[sidebarPanel];
  const Icon = config.icon;

  return (
    <div 
      className="h-full flex flex-col bg-surface animate-fade-in"
      style={{ width: sidebarWidth }}
    >
      <div className="flex items-center justify-between px-3 h-10 border-b border-border shrink-0">
        <div className="flex items-center gap-2">
          <Icon className="w-4 h-4 text-content-secondary" />
          <span className="text-sm font-medium">{config.label}</span>
        </div>
        <IconButton size="sm" onClick={toggleSidebar}>
          <X />
        </IconButton>
      </div>

      <div className="flex-1 overflow-y-auto pt-2">
        {sidebarPanel === "bookmarks" && <BookmarksPanel />}
        {sidebarPanel === "history" && <HistoryPanel />}
        {sidebarPanel === "conversations" && <ConversationPanel />}
        {sidebarPanel === "downloads" && <DownloadsPanel />}
        {sidebarPanel === "tabs" && <TabsPanel />}
      </div>
    </div>
  );
}

function ConversationPanel() {
  const conversations = useConversationStore((s) => s.conversations);
  const activeConversationId = useConversationStore((s) => s.activeConversationId);
  const setActiveConversation = useConversationStore((s) => s.setActiveConversation);
  const deleteConversation = useConversationStore((s) => s.deleteConversation);
  const createConversation = useConversationStore((s) => s.createConversation);

  const pinned = conversations.filter((c) => c.isPinned);
  const unpinned = conversations.filter((c) => !c.isPinned);

  return (
    <div className="py-1">
      <div className="px-3 py-2">
        <button
          onClick={() => createConversation()}
          className={cn(
            "w-full flex items-center justify-center gap-1.5 h-8 rounded-md text-xs font-medium",
            "bg-brand-600 text-white hover:bg-brand-700 transition-colors",
          )}
        >
          <Plus className="w-3.5 h-3.5" />
          新对话
        </button>
      </div>

      {pinned.length > 0 && (
        <>
          <div className="px-3 py-1.5 text-2xs font-medium text-content-tertiary uppercase tracking-wider">
            已置顶
          </div>
          {pinned.map((conv) => (
            <ConversationItem
              key={conv.id}
              id={conv.id}
              title={conv.title}
              messageCount={conv.messageCount}
              isActive={conv.id === activeConversationId}
              isPinned
              onSelect={() => setActiveConversation(conv.id)}
              onDelete={() => deleteConversation(conv.id)}
            />
          ))}
        </>
      )}

      {unpinned.length > 0 && (
        <>
          <div className="px-3 py-1.5 text-2xs font-medium text-content-tertiary uppercase tracking-wider">
            最近
          </div>
          {unpinned.map((conv) => (
            <ConversationItem
              key={conv.id}
              id={conv.id}
              title={conv.title}
              messageCount={conv.messageCount}
              isActive={conv.id === activeConversationId}
              isPinned={false}
              onSelect={() => setActiveConversation(conv.id)}
              onDelete={() => deleteConversation(conv.id)}
            />
          ))}
        </>
      )}
    </div>
  );
}

function ConversationItem({
  title,
  messageCount,
  isActive,
  isPinned,
  onSelect,
  onDelete,
}: {
  id: string;
  title: string;
  messageCount: number;
  isActive: boolean;
  isPinned: boolean;
  onSelect: () => void;
  onDelete: () => void;
}) {
  return (
    <div
      onClick={onSelect}
      className={cn(
        "flex items-center gap-2 px-3 py-2 cursor-pointer group",
        isActive ? "bg-surface-active" : "hover:bg-surface-hover",
      )}
    >
      <div className="flex-1 min-w-0">
        <div className="text-xs truncate flex items-center gap-1">
          {isPinned && <Pin className="w-3 h-3 text-brand-500 shrink-0" />}
          {truncate(title, 20)}
        </div>
        <div className="text-2xs text-content-tertiary">
          {messageCount} 条消息
        </div>
      </div>
      <IconButton
        size="sm"
        className="opacity-0 group-hover:opacity-100"
        onClick={(e) => {
          e.stopPropagation();
          onDelete();
        }}
      >
        <Trash2 className="w-3 h-3" />
      </IconButton>
    </div>
  );
}
