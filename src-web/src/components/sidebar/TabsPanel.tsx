import { useState } from "react";
import {
  X,
  Search,
  Globe,
  Loader2,
  Pin,
  VolumeX,
  Layers,
} from "lucide-react";
import { useTabStore } from "@/stores/tabStore";
import { useUIStore } from "@/stores/uiStore";
import { cn, getDomain } from "@/lib/utils";

export function TabsPanel() {
  const tabs = useTabStore((s) => s.tabs);
  const activeTabId = useTabStore((s) => s.activeTabId);
  const setActiveTab = useTabStore((s) => s.setActiveTab);
  const closeTab = useTabStore((s) => s.closeTab);
  const toggleSidebar = useUIStore((s) => s.toggleSidebar);
  const [searchQuery, setSearchQuery] = useState("");

  const filteredTabs = tabs.filter(
    (t) =>
      !searchQuery.trim() ||
      t.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
      t.url.toLowerCase().includes(searchQuery.toLowerCase())
  );

  const handleTabClick = (tabId: string) => {
    setActiveTab(tabId);
    toggleSidebar();
  };

  const handleCloseTab = (e: React.MouseEvent, tabId: string) => {
    e.stopPropagation();
    closeTab(tabId);
  };

  return (
    <div className="flex flex-col h-full">
      {/* 搜索栏 */}
      <div className="px-3 py-2 border-b border-border/50">
        <div className="flex items-center gap-2 h-8 rounded-lg px-2.5 bg-surface-secondary border border-border focus-within:border-brand-500 focus-within:ring-2 focus-within:ring-brand-500/20 transition-all">
          <Search className="w-3.5 h-3.5 text-content-tertiary shrink-0" />
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder="搜索标签页..."
            className="flex-1 bg-transparent text-xs text-content outline-none placeholder:text-content-tertiary"
          />
        </div>
        <div className="mt-1.5 text-2xs text-content-tertiary px-1">
          共 {tabs.length} 个标签页
          {searchQuery && ` · 找到 ${filteredTabs.length} 个`}
        </div>
      </div>

      {/* 标签列表 */}
      <div className="flex-1 overflow-y-auto">
        {filteredTabs.length === 0 ? (
          <div className="flex flex-col items-center justify-center py-8 text-content-tertiary">
            <Layers className="w-8 h-8 mb-2" />
            <div className="text-xs">
              {searchQuery ? "没有找到匹配的标签页" : "没有打开的标签页"}
            </div>
          </div>
        ) : (
          <div className="py-1">
            {filteredTabs.map((tab) => (
              <div
                key={tab.id}
                onClick={() => handleTabClick(tab.id)}
                className={cn(
                  "flex items-center gap-2.5 px-3 py-2 cursor-pointer group transition-colors",
                  tab.id === activeTabId
                    ? "bg-brand-500/10 border-l-2 border-brand-500"
                    : "hover:bg-surface-hover border-l-2 border-transparent"
                )}
              >
                {/* 图标 */}
                <div className="shrink-0 w-5 h-5 flex items-center justify-center">
                  {tab.isLoading ? (
                    <Loader2 className="w-3.5 h-3.5 text-brand-500 animate-spin" />
                  ) : tab.favicon ? (
                    <img
                      src={tab.favicon}
                      alt=""
                      className="w-4 h-4 rounded-sm"
                    />
                  ) : (
                    <Globe className="w-3.5 h-3.5 text-content-tertiary" />
                  )}
                </div>

                {/* 内容 */}
                <div className="flex-1 min-w-0">
                  <div
                    className={cn(
                      "text-xs truncate",
                      tab.id === activeTabId
                        ? "text-brand-600 font-medium"
                        : "text-content"
                    )}
                  >
                    {tab.title || "新标签页"}
                  </div>
                  <div className="text-2xs text-content-tertiary truncate mt-0.5">
                    {tab.url === "about:blank" ? "新标签页" : getDomain(tab.url)}
                  </div>
                </div>

                {/* 状态图标 */}
                <div className="flex items-center gap-1 shrink-0">
                  {tab.isPinned && (
                    <Pin className="w-3 h-3 text-brand-500" />
                  )}
                  {tab.isMuted && (
                    <VolumeX className="w-3 h-3 text-content-tertiary" />
                  )}
                </div>

                {/* 关闭按钮 */}
                <button
                  onClick={(e) => handleCloseTab(e, tab.id)}
                  className="shrink-0 w-5 h-5 rounded flex items-center justify-center opacity-0 group-hover:opacity-100 hover:bg-surface-active transition-opacity"
                >
                  <X className="w-3 h-3 text-content-tertiary" />
                </button>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
