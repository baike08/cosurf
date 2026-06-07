import {
  Download,
  X,
  FolderOpen,
  Trash2,
  Ban,
  Check,
  AlertCircle,
} from "lucide-react";
import { useDownloadStore } from "@/stores/downloadStore";
import { IconButton } from "@/components/ui/IconButton";
import type { DownloadItem } from "@cosurf/shared";

export function DownloadsPanel() {
  const downloads = useDownloadStore((s) => s.downloads);
  const clearCompleted = useDownloadStore((s) => s.clearCompleted);
  const removeDownload = useDownloadStore((s) => s.removeDownload);
  const cancelDownload = useDownloadStore((s) => s.cancelDownload);

  if (downloads.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center h-full text-center gap-3 py-8">
        <Download className="w-10 h-10 text-content-tertiary" />
        <div className="text-sm text-content-secondary">暂无下载</div>
        <div className="text-xs text-content-tertiary">
          下载文件时会显示在这里
        </div>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center justify-between px-3 py-2 border-b border-border">
        <div className="flex items-center gap-2">
          <Download className="w-4 h-4 text-content-secondary" />
          <span className="text-sm font-medium">
            下载 ({downloads.length})
          </span>
        </div>
        <div className="flex items-center gap-1">
          <IconButton size="sm" onClick={clearCompleted}>
            <Trash2 className="w-3.5 h-3.5" />
          </IconButton>
        </div>
      </div>

      <div className="flex-1 overflow-y-auto">
        {downloads.map((item) => (
          <DownloadRow
            key={item.id}
            item={item}
            onCancel={() => cancelDownload(item.id)}
            onRemove={() => removeDownload(item.id)}
          />
        ))}
      </div>
    </div>
  );
}

function DownloadRow({
  item,
  onCancel,
  onRemove,
}: {
  item: DownloadItem;
  onCancel: () => void;
  onRemove: () => void;
}) {
  const progress =
    item.totalBytes > 0
      ? Math.round((item.receivedBytes / item.totalBytes) * 100)
      : 0;

  return (
    <div className="flex items-start gap-3 px-3 py-2.5 border-b border-border/50 hover:bg-surface-hover/50 transition-colors group">
      <div className="mt-1 shrink-0">
        {item.state === "completed" ? (
          <Check className="w-4 h-4 text-green-500" />
        ) : item.state === "in_progress" ? (
          <Download className="w-4 h-4 text-brand-500 animate-pulse" />
        ) : item.state === "cancelled" ? (
          <Ban className="w-4 h-4 text-content-tertiary" />
        ) : (
          <AlertCircle className="w-4 h-4 text-red-500" />
        )}
      </div>

      <div className="flex-1 min-w-0">
        <div className="flex items-start justify-between gap-2">
          <div className="flex-1 min-w-0">
            <div className="text-xs font-medium text-content truncate">
              {item.filename}
            </div>
            <div className="text-2xs text-content-tertiary mt-0.5 truncate">
              {formatBytes(item.receivedBytes)} / {formatBytes(item.totalBytes)}
              {item.state === "in_progress" && ` · ${progress}%`}
            </div>
          </div>

          <div className="flex items-center gap-0.5 shrink-0 opacity-0 group-hover:opacity-100 transition-opacity">
            {item.state === "completed" && (
              <IconButton size="sm">
                <FolderOpen className="w-3.5 h-3.5" />
              </IconButton>
            )}
            {item.state === "in_progress" && (
              <IconButton size="sm" onClick={onCancel}>
                <Ban className="w-3.5 h-3.5" />
              </IconButton>
            )}
            <IconButton size="sm" onClick={onRemove}>
              <X className="w-3.5 h-3.5" />
            </IconButton>
          </div>
        </div>

        {/* 进度条 */}
        {item.state === "in_progress" && (
          <div className="mt-2 h-1 bg-surface-tertiary rounded-full overflow-hidden">
            <div
              className="h-full bg-brand-500 rounded-full transition-all"
              style={{ width: `${progress}%` }}
            />
          </div>
        )}

        {item.state === "completed" && (
          <div className="text-2xs text-green-500 mt-0.5">
            下载完成
          </div>
        )}

        {item.state === "cancelled" && (
          <div className="text-2xs text-content-tertiary mt-0.5">
            已取消
          </div>
        )}

        {item.state === "interrupted" && (
          <div className="text-2xs text-red-500 mt-0.5">
            下载失败: {item.error || "网络错误"}
          </div>
        )}
      </div>
    </div>
  );
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024)
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
}
