import { useEffect, useState, useRef } from "react";
import {
  Search,
  Trash2,
  FolderPlus,
  FolderOpen,
  Folder,
  ExternalLink,
  Plus,
  ChevronRight,
  Home,
  Loader2,
  Star,
  X,
} from "lucide-react";
import { useBookmarkStore } from "@/stores/bookmarkStore";
import { useTabStore } from "@/stores/tabStore";
import { useUIStore } from "@/stores/uiStore";
import { getDomain, cn } from "@/lib/utils";

export function BookmarksPanel() {
  const bookmarks = useBookmarkStore((s) => s.bookmarks);
  const folders = useBookmarkStore((s) => s.folders);
  const currentFolderId = useBookmarkStore((s) => s.currentFolderId);
  const loading = useBookmarkStore((s) => s.loading);
  const searchQuery = useBookmarkStore((s) => s.searchQuery);
  const loadBookmarks = useBookmarkStore((s) => s.loadBookmarks);
  const loadFolders = useBookmarkStore((s) => s.loadFolders);
  const setCurrentFolder = useBookmarkStore((s) => s.setCurrentFolder);
  const setSearchQuery = useBookmarkStore((s) => s.setSearchQuery);
  const deleteBookmark = useBookmarkStore((s) => s.deleteBookmark);
  const addFolder = useBookmarkStore((s) => s.addFolder);
  const deleteFolder = useBookmarkStore((s) => s.deleteFolder);
  const toggleSidebar = useUIStore((s) => s.toggleSidebar);
  const addTab = useTabStore((s) => s.addTab);

  const [showNewFolder, setShowNewFolder] = useState(false);
  const [newFolderName, setNewFolderName] = useState("");
  const newFolderInputRef = useRef<HTMLInputElement>(null);

  // 打开面板时加载数据
  useEffect(() => {
    loadBookmarks();
    loadFolders();
  }, [loadBookmarks, loadFolders]);

  // 新建文件夹
  const handleCreateFolder = async () => {
    if (!newFolderName.trim()) return;
    await addFolder(newFolderName.trim(), currentFolderId || undefined);
    setNewFolderName("");
    setShowNewFolder(false);
  };

  // 点击书签
  const handleBookmarkClick = (url: string, title: string) => {
    addTab(url, title);
    toggleSidebar();
  };

  // 过滤书签（搜索）
  const filteredBookmarks = searchQuery.trim()
    ? bookmarks.filter(
        (b) =>
          b.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
          b.url.toLowerCase().includes(searchQuery.toLowerCase())
      )
    : bookmarks;

  // 当前文件夹名称
  const currentFolder = folders.find((f) => f.id === currentFolderId);

  return (
    <div className="flex flex-col h-full">
      {/* 搜索栏 */}
      <div className="px-3 py-2 border-b border-border/50 space-y-2">
        <div className="flex items-center gap-2 h-8 rounded-lg px-2.5 bg-surface-secondary border border-border focus-within:border-brand-500 focus-within:ring-2 focus-within:ring-brand-500/20 transition-all">
          {loading ? (
            <Loader2 className="w-3.5 h-3.5 text-content-tertiary shrink-0 animate-spin" />
          ) : (
            <Search className="w-3.5 h-3.5 text-content-tertiary shrink-0" />
          )}
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder="搜索书签..."
            className="flex-1 bg-transparent text-xs text-content outline-none placeholder:text-content-tertiary"
          />
          {searchQuery && (
            <button
              onClick={() => setSearchQuery("")}
              className="shrink-0 text-content-tertiary hover:text-content"
            >
              <X className="w-3 h-3" />
            </button>
          )}
        </div>

        {/* 操作栏 */}
        <div className="flex items-center justify-between px-1">
          <div className="flex items-center gap-1 text-2xs text-content-tertiary">
            {/* 面包屑导航 */}
            <button
              onClick={() => setCurrentFolder(null)}
              className={cn(
                "flex items-center gap-0.5 hover:text-content transition-colors",
                !currentFolderId && "text-content font-medium"
              )}
            >
              <Home className="w-3 h-3" />
              根目录
            </button>
            {currentFolder && (
              <>
                <ChevronRight className="w-3 h-3" />
                <span className="text-content font-medium truncate max-w-[100px]">
                  {currentFolder.name}
                </span>
              </>
            )}
          </div>
          <div className="flex items-center gap-1">
            <button
              onClick={() => {
                setShowNewFolder(!showNewFolder);
                if (!showNewFolder) {
                  setTimeout(() => newFolderInputRef.current?.focus(), 50);
                }
              }}
              className="text-2xs text-content-tertiary hover:text-brand-500 transition-colors flex items-center gap-1 px-1.5 py-0.5 rounded hover:bg-surface-hover"
            >
              <FolderPlus className="w-3 h-3" />
              新建文件夹
            </button>
          </div>
        </div>

        {/* 新建文件夹输入 */}
        {showNewFolder && (
          <div className="flex items-center gap-2 px-1">
            <Folder className="w-3.5 h-3.5 text-amber-500 shrink-0" />
            <input
              ref={newFolderInputRef}
              type="text"
              value={newFolderName}
              onChange={(e) => setNewFolderName(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter") handleCreateFolder();
                if (e.key === "Escape") {
                  setShowNewFolder(false);
                  setNewFolderName("");
                }
              }}
              placeholder="文件夹名称..."
              className="flex-1 bg-surface-secondary border border-border rounded px-2 py-1 text-xs outline-none focus:border-brand-500 focus:ring-1 focus:ring-brand-500/20"
            />
            <button
              onClick={handleCreateFolder}
              disabled={!newFolderName.trim()}
              className="text-2xs text-brand-500 hover:text-brand-600 disabled:opacity-40 font-medium"
            >
              创建
            </button>
          </div>
        )}
      </div>

      {/* 文件夹列表 */}
      {folders.length > 0 && !searchQuery.trim() && (
        <div className="px-3 py-2 border-b border-border/30">
          <div className="text-2xs font-medium text-content-tertiary uppercase tracking-wider mb-1.5 px-1">
            文件夹
          </div>
          <div className="space-y-0.5">
            {folders.map((folder) => (
              <div
                key={folder.id}
                className={cn(
                  "flex items-center gap-2 px-2 py-1.5 rounded-md cursor-pointer group transition-colors",
                  currentFolderId === folder.id
                    ? "bg-brand-500/10 text-brand-600"
                    : "hover:bg-surface-hover text-content"
                )}
                onClick={() => setCurrentFolder(folder.id)}
              >
                <FolderOpen
                  className={cn(
                    "w-3.5 h-3.5 shrink-0",
                    currentFolderId === folder.id ? "text-brand-500" : "text-amber-500"
                  )}
                />
                <span className="flex-1 text-xs truncate">{folder.name}</span>
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    if (confirm(`确定删除文件夹「${folder.name}」及其中的所有书签吗？`)) {
                      deleteFolder(folder.id);
                    }
                  }}
                  className="opacity-0 group-hover:opacity-100 w-5 h-5 rounded flex items-center justify-center hover:bg-surface-active hover:text-red-500 transition-all"
                >
                  <Trash2 className="w-3 h-3 text-content-tertiary" />
                </button>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* 书签列表 */}
      <div className="flex-1 overflow-y-auto">
        {filteredBookmarks.length === 0 && !loading ? (
          <div className="flex flex-col items-center justify-center py-12 text-content-tertiary">
            <Star className="w-8 h-8 mb-2" />
            <div className="text-xs">
              {searchQuery ? "没有找到匹配的书签" : "暂无书签"}
            </div>
            {!searchQuery && (
              <div className="text-2xs mt-1 text-content-tertiary">
                点击地址栏旁的星标添加书签
              </div>
            )}
          </div>
        ) : (
          <div className="py-1">
            {/* 书签统计 */}
            <div className="px-3 py-1.5 text-2xs text-content-tertiary">
              {filteredBookmarks.length} 个书签
            </div>
            {filteredBookmarks.map((bm) => (
              <div
                key={bm.id}
                className="flex items-center gap-2 px-3 py-1.5 hover:bg-surface-hover cursor-pointer group"
                onClick={() => handleBookmarkClick(bm.url, bm.title)}
              >
                <div className="w-4 h-4 rounded-sm bg-surface-tertiary flex items-center justify-center shrink-0">
                  {bm.favicon ? (
                    <img
                      src={bm.favicon}
                      alt=""
                      className="w-3.5 h-3.5 rounded-sm"
                      onError={(e) => {
                        (e.target as HTMLImageElement).style.display = "none";
                      }}
                    />
                  ) : (
                    <span className="text-2xs font-bold text-content-tertiary">
                      {getDomain(bm.url).charAt(0).toUpperCase()}
                    </span>
                  )}
                </div>
                <div className="flex-1 min-w-0">
                  <div className="text-xs truncate text-content">{bm.title}</div>
                  <div className="text-2xs text-content-tertiary truncate">
                    {getDomain(bm.url)}
                  </div>
                </div>
                <div className="flex items-center gap-0.5 shrink-0 opacity-0 group-hover:opacity-100 transition-opacity">
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      handleBookmarkClick(bm.url, bm.title);
                    }}
                    className="w-5 h-5 rounded flex items-center justify-center hover:bg-surface-active transition-colors"
                    title="在新标签页打开"
                  >
                    <ExternalLink className="w-3 h-3 text-content-tertiary" />
                  </button>
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      deleteBookmark(bm.id);
                    }}
                    className="w-5 h-5 rounded flex items-center justify-center hover:bg-surface-active hover:text-red-500 transition-colors"
                    title="删除书签"
                  >
                    <Trash2 className="w-3 h-3 text-content-tertiary" />
                  </button>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
