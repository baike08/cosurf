import { Plus, X, VolumeX, Loader2, Globe } from "lucide-react";
import { useTabStore } from "@/stores/tabStore";
import { cn, truncate } from "@/lib/utils";
import { IconButton } from "@/components/ui/IconButton";

export function TabBar() {
  // 【关键修复】分别订阅 tabs 和 activeTabId，避免对象引用导致的无限循环
  const tabs = useTabStore((s) => s.tabs);
  const activeTabId = useTabStore((s) => s.activeTabId);
  const setActiveTab = useTabStore((s) => s.setActiveTab);
  const closeTab = useTabStore((s) => s.closeTab);
  const addTab = useTabStore((s) => s.addTab);

  return (
    <div className="h-tab-bar flex items-end bg-surface-secondary border-b border-border drag-region select-none">
      <div className="flex items-end flex-1 overflow-x-auto no-drag px-1 pt-1 gap-px">
        {tabs.map((tab) => (
          <div
            key={tab.id}
            onClick={() => setActiveTab(tab.id)}
            className={cn(
              "group flex items-center gap-1.5 h-[30px] min-w-[120px] max-w-[220px] px-2.5 rounded-t-lg cursor-pointer transition-colors",
              tab.id === activeTabId
                ? "bg-surface text-content"
                : "bg-transparent text-content-secondary hover:bg-surface-hover",
            )}
          >
            <div className="shrink-0 w-4 h-4 flex items-center justify-center">
              {tab.isLoading ? (
                <Loader2 className="w-3 h-3 animate-spin text-brand-500" />
              ) : tab.favicon ? (
                <img
                  src={tab.favicon}
                  alt=""
                  className="w-3.5 h-3.5 rounded-sm"
                  draggable={false}
                />
              ) : (
                <Globe className="w-3.5 h-3.5 text-content-tertiary" />
              )}
            </div>

            <span className="text-xs truncate flex-1 leading-tight">
              {truncate(tab.title, 24)}
            </span>

            {tab.isMuted && (
              <VolumeX className="w-3 h-3 text-content-tertiary shrink-0" />
            )}

            <button
              onClick={(e) => {
                e.stopPropagation();
                closeTab(tab.id);
              }}
              className={cn(
                "shrink-0 w-4 h-4 rounded-sm flex items-center justify-center transition-opacity",
                "opacity-0 group-hover:opacity-100 hover:bg-surface-active",
                tab.id === activeTabId && "opacity-60",
              )}
            >
              <X className="w-3 h-3" />
            </button>
          </div>
        ))}
      </div>

      <div className="flex items-center gap-0.5 px-1.5 pb-1.5 no-drag">
        <IconButton 
          size="sm" 
          onClick={() => addTab()}
        >
          <Plus />
        </IconButton>
      </div>
    </div>
  );
}
