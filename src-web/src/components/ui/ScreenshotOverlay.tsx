import { useEffect } from "react";
import { X, Copy, Download, Camera } from "lucide-react";
import { useScreenshotStore } from "@/stores/screenshotStore";
import { ScreenshotSelector } from "./ScreenshotSelector";
import { IconButton } from "@/components/ui/IconButton";
import { Tooltip } from "@/components/ui/Tooltip";
import { cn } from "@/lib/utils";

export function ScreenshotOverlay() {
  const isOpen = useScreenshotStore((s) => s.isOpen);
  const showSelector = useScreenshotStore((s) => s.showSelector);
  const fullScreenImage = useScreenshotStore((s) => s.fullScreenImage);
  const screenWidth = useScreenshotStore((s) => s.screenWidth);
  const screenHeight = useScreenshotStore((s) => s.screenHeight);
  const imageData = useScreenshotStore((s) => s.imageData);
  const imageWidth = useScreenshotStore((s) => s.imageWidth);
  const imageHeight = useScreenshotStore((s) => s.imageHeight);
  const saving = useScreenshotStore((s) => s.saving);
  const copying = useScreenshotStore((s) => s.copying);
  const toast = useScreenshotStore((s) => s.toast);
  const close = useScreenshotStore((s) => s.close);
  const copyToClipboard = useScreenshotStore((s) => s.copyToClipboard);
  const saveToFile = useScreenshotStore((s) => s.saveToFile);
  const init = useScreenshotStore((s) => s.init);
  const captureRegion = useScreenshotStore((s) => s.captureRegion);

  // 初始化监听
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    init().then((fn) => {
      unlisten = fn;
    });
    return () => {
      unlisten?.();
    };
  }, [init]);

  // Esc 键关闭
  useEffect(() => {
    if (!isOpen) return;
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") close();
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [isOpen, close]);

  if (!isOpen && !showSelector) return null;

  // 显示区域选择器
  if (showSelector && fullScreenImage) {
    return (
      <ScreenshotSelector
        fullScreenImage={fullScreenImage}
        screenWidth={screenWidth}
        screenHeight={screenHeight}
        onCapture={captureRegion}
        onCancel={() => useScreenshotStore.getState().close()}
      />
    );
  }

  const src = `data:image/png;base64,${imageData}`;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center animate-fade-in">
      {/* 背景遮罩 */}
      <div
        className="absolute inset-0 bg-black/60 backdrop-blur-sm"
        onClick={close}
      />

      {/* 主体内容 */}
      <div className="relative flex flex-col items-center gap-3 max-w-[90vw] max-h-[90vh]">
        {/* 顶部信息栏 */}
        <div className="flex items-center justify-between w-full bg-surface/90 backdrop-blur rounded-xl px-4 py-2 shadow-lg border border-border/50">
          <div className="flex items-center gap-2">
            <Camera className="w-4 h-4 text-brand-500" />
            <span className="text-sm font-medium">截图预览</span>
            <span className="text-2xs text-content-tertiary">
              {imageWidth} × {imageHeight}
            </span>
          </div>
          <div className="flex items-center gap-1">
            <span className="text-2xs text-content-tertiary mr-2">
              Ctrl+Shift+X
            </span>
            <Tooltip label="关闭 (Esc)">
              <IconButton size="sm" onClick={close}>
                <X />
              </IconButton>
            </Tooltip>
          </div>
        </div>

        {/* 图片预览 */}
        <div className="relative rounded-xl overflow-hidden shadow-2xl border border-border/50 bg-surface">
          <img
            src={src}
            alt="Screenshot"
            className="max-w-[85vw] max-h-[70vh] object-contain"
          />
        </div>

        {/* 操作栏 */}
        <div className="flex items-center gap-2 bg-surface/90 backdrop-blur rounded-xl px-4 py-2 shadow-lg border border-border/50">
          <button
            onClick={copyToClipboard}
            disabled={copying || saving}
            className={cn(
              "flex items-center gap-1.5 px-4 py-2 rounded-lg text-xs font-medium transition-all",
              "bg-brand-600 text-white hover:bg-brand-700",
              "disabled:opacity-50 disabled:cursor-not-allowed",
            )}
          >
            <Copy className="w-3.5 h-3.5" />
            {copying ? "复制中..." : "复制到剪贴板"}
          </button>

          <button
            onClick={saveToFile}
            disabled={saving || copying}
            className={cn(
              "flex items-center gap-1.5 px-4 py-2 rounded-lg text-xs font-medium transition-all",
              "bg-surface-secondary text-content hover:bg-surface-hover border border-border",
              "disabled:opacity-50 disabled:cursor-not-allowed",
            )}
          >
            <Download className="w-3.5 h-3.5" />
            {saving ? "保存中..." : "保存为文件"}
          </button>

          <div className="w-px h-5 bg-border mx-1" />

          <button
            onClick={close}
            className="flex items-center gap-1.5 px-3 py-2 rounded-lg text-xs text-content-tertiary hover:text-content hover:bg-surface-hover transition-all"
          >
            取消
          </button>
        </div>

        {/* Toast 提示 */}
        {toast && (
          <div className="absolute bottom-0 left-1/2 -translate-x-1/2 translate-y-12 px-4 py-2 rounded-lg bg-surface shadow-lg border border-border text-sm font-medium animate-slide-up">
            {toast}
          </div>
        )}
      </div>
    </div>
  );
}
