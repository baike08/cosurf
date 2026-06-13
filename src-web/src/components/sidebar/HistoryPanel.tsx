import { useEffect } from "react";
import {
  Search,
  Trash2,
  Clock,
  Loader2,
  X,
} from "lucide-react";
import { useHistoryStore } from "@/stores/historyStore";
import { useTabStore } from "@/stores/tabStore";
import { useUIStore } from "@/stores/uiStore";
import { getDomain, formatTime } from "@/lib/utils";

export function HistoryPanel() {
  const entries = useHistoryStore((s) => s.entries);
  const loading = useHistoryStore((s) => s.loading);
  const searchQuery = useHistoryStore((s) => s.searchQuery);
  const setSearchQuery = useHistoryStore((s) => s.setSearchQuery);
  const loadHistory = useHistoryStore((s) => s.loadHistory);
  const deleteEntry = useHistoryStore((s) => s.deleteEntry);
  const clearAll = useHistoryStore((s) => s.clearAll);
  const toggleSidebar = useUIStore((s) => s.toggleSidebar);
  const addTab = useTabStore((s) => s.addTab);

  // 打开面板时加载历史
  useEffect(() => {
    loadHistory();
  }, [loadHistory]);

  const handleEntryClick = (url: string) => {
    addTab(url, getDomain(url));
    toggleSidebar();
  };

  // 按日期分组
  const groupedEntries = groupByDate(entries);

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
            placeholder="搜索历史记录..."
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
        <div className="flex items-center justify-between px-1">
          <div className="text-2xs text-content-tertiary">
            {entries.length} 条记录
          </div>
          {entries.length > 0 && (
            <button
              onClick={() => {
                if (confirm("确定要清除所有历史记录吗？")) {
                  clearAll();
                }
              }}
              className="text-2xs text-content-tertiary hover:text-red-500 transition-colors flex items-center gap-1"
            >
              <Trash2 className="w-3 h-3" />
              清除全部
            </button>
          )}
        </div>
      </div>

      {/* 历史列表 */}
      <div className="flex-1 overflow-y-auto">
        {entries.length === 0 && !loading ? (
          <div className="flex flex-col items-center justify-center py-12 text-content-tertiary">
            <Clock className="w-8 h-8 mb-2" />
            <div className="text-xs">
              {searchQuery ? "没有找到匹配的记录" : "暂无浏览历史"}
            </div>
          </div>
        ) : (
          <div className="py-1">
            {groupedEntries.map(({ date, items }) => (
              <div key={date}>
                <div className="px-3 py-1.5 text-2xs font-medium text-content-tertiary sticky top-0 bg-surface/95 backdrop-blur-sm border-b border-border/30">
                  {date}
                </div>
                {items.map((entry) => (
                  <div
                    key={entry.id}
                    className="flex items-center gap-2 px-3 py-1.5 hover:bg-surface-hover cursor-pointer group"
                    onClick={() => handleEntryClick(entry.url)}
                  >
                    <div className="w-4 h-4 rounded-sm bg-surface-tertiary flex items-center justify-center shrink-0">
                      <span className="text-2xs font-bold text-content-tertiary">
                        {getDomain(entry.url).charAt(0).toUpperCase()}
                      </span>
                    </div>
                    <div className="flex-1 min-w-0">
                      <div className="text-xs truncate text-content">
                        {entry.title || getDomain(entry.url)}
                      </div>
                      <div className="text-2xs text-content-tertiary truncate">
                        {getDomain(entry.url)} · {formatTime(entry.visitedAt)}
                      </div>
                    </div>
                    <div className="flex items-center gap-0.5 shrink-0 opacity-0 group-hover:opacity-100 transition-opacity">
                      <button
                        onClick={(e) => {
                          e.stopPropagation();
                          deleteEntry(entry.id);
                        }}
                        className="w-5 h-5 rounded flex items-center justify-center hover:bg-surface-active hover:text-red-500 transition-colors"
                      >
                        <Trash2 className="w-3 h-3 text-content-tertiary" />
                      </button>
                    </div>
                  </div>
                ))}
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

function groupByDate(
  entries: { id: string; title: string; url: string; visitedAt: string }[]
): { date: string; items: typeof entries }[] {
  const groups: Map<string, typeof entries> = new Map();
  const now = new Date();
  const today = new Date(now.getFullYear(), now.getMonth(), now.getDate());
  const yesterday = new Date(today.getTime() - 86400000);

  for (const entry of entries) {
    const date = new Date(entry.visitedAt);
    let label: string;

    if (date >= today) {
      label = "今天";
    } else if (date >= yesterday) {
      label = "昨天";
    } else if (today.getTime() - date.getTime() < 7 * 86400000) {
      label = "最近7天";
    } else if (today.getTime() - date.getTime() < 30 * 86400000) {
      label = "最近30天";
    } else {
      label = date.toLocaleDateString("zh-CN", {
        year: "numeric",
        month: "long",
        day: "numeric",
      });
    }

    if (!groups.has(label)) {
      groups.set(label, []);
    }
    groups.get(label)!.push(entry);
  }

  return Array.from(groups.entries()).map(([date, items]) => ({
    date,
    items,
  }));
}
