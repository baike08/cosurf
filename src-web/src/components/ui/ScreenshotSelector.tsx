import { useState, useRef, useEffect } from "react";
import { X } from "lucide-react";

interface ScreenshotSelectorProps {
  fullScreenImage: string;
  screenWidth: number;
  screenHeight: number;
  onCapture: (rect: { x: number; y: number; width: number; height: number }) => void;
  onCancel: () => void;
}

export function ScreenshotSelector({ fullScreenImage, screenWidth, screenHeight, onCapture, onCancel }: ScreenshotSelectorProps) {
  const [isSelecting, setIsSelecting] = useState(false);
  const [startPos, setStartPos] = useState<{ x: number; y: number } | null>(null);
  const [currentRect, setCurrentRect] = useState<{ x: number; y: number; width: number; height: number } | null>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const imageRef = useRef<HTMLImageElement>(null);
  const [imageRect, setImageRect] = useState<{ x: number; y: number; width: number; height: number } | null>(null);

  // 计算图片实际显示位置和缩放比例
  useEffect(() => {
    const updateImageRect = () => {
      if (imageRef.current && imageRef.current.complete) {
        const rect = imageRef.current.getBoundingClientRect();
        console.log('Image rect updated:', rect, 'screen dimensions:', screenWidth, screenHeight);
        setImageRect({
          x: rect.left,
          y: rect.top,
          width: rect.width,
          height: rect.height,
        });
      }
    };
    
    // 等待图片加载完成
    if (imageRef.current && imageRef.current.complete) {
      updateImageRect();
    } else if (imageRef.current) {
      imageRef.current.onload = updateImageRect;
    }
    
    // 稍微延迟确保布局完成
    const timer = setTimeout(updateImageRect, 100);
    
    window.addEventListener('resize', updateImageRect);
    return () => {
      window.removeEventListener('resize', updateImageRect);
      clearTimeout(timer);
    };
  }, [fullScreenImage, screenWidth, screenHeight]);

  // Esc 键取消
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") onCancel();
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [onCancel]);

  const handleMouseDown = (e: React.MouseEvent) => {
    e.preventDefault();
    if (!imageRect) return;
    
    setIsSelecting(true);
    // 将屏幕坐标转换为图片的物理像素坐标
    const dprX = (e.clientX - imageRect.x) * (screenWidth / imageRect.width);
    const dprY = (e.clientY - imageRect.y) * (screenHeight / imageRect.height);
    setStartPos({ x: dprX, y: dprY });
  };

  const handleMouseMove = (e: React.MouseEvent) => {
    if (!isSelecting || !startPos || !imageRect) return;

    // 将屏幕坐标转换为图片的物理像素坐标
    const dprX = (e.clientX - imageRect.x) * (screenWidth / imageRect.width);
    const dprY = (e.clientY - imageRect.y) * (screenHeight / imageRect.height);

    const x = Math.min(startPos.x, dprX);
    const y = Math.min(startPos.y, dprY);
    const width = Math.abs(dprX - startPos.x);
    const height = Math.abs(dprY - startPos.y);

    setCurrentRect({ x, y, width, height });
  };

  const handleMouseUp = () => {
    if (currentRect && currentRect.width > 10 && currentRect.height > 10) {
      // 四舍五入坐标，避免小数点误差
      const finalRect = {
        x: Math.round(Math.max(0, currentRect.x)),
        y: Math.round(Math.max(0, currentRect.y)),
        width: Math.round(currentRect.width),
        height: Math.round(currentRect.height),
      };
      console.log('Capturing region:', finalRect, 'from screen:', screenWidth, screenHeight);
      onCapture(finalRect);
    }
    setIsSelecting(false);
    setStartPos(null);
    setCurrentRect(null);
  };

  return (
    <div
      ref={containerRef}
      className="fixed inset-0 z-[9999] cursor-crosshair bg-black/40"
      onMouseDown={handleMouseDown}
      onMouseMove={handleMouseMove}
      onMouseUp={handleMouseUp}
      onClick={(e) => {
        if (e.target === containerRef.current) onCancel();
      }}
    >
      {/* 全屏截图背景 */}
      <div className="absolute inset-0 flex items-center justify-center">
        <img
          ref={imageRef}
          src={`data:image/png;base64,${fullScreenImage}`}
          alt="Screenshot"
          className="max-w-full max-h-full object-contain"
        />
      </div>

      {/* 提示文本 */}
      {!isSelecting && (
        <div className="fixed top-8 left-1/2 -translate-x-1/2 px-4 py-2 bg-surface/90 backdrop-blur rounded-lg border border-border shadow-lg text-sm font-medium z-[10000]">
          拖拽鼠标选择截图区域，按 Esc 取消
        </div>
      )}

      {/* 选择框 */}
      {currentRect && imageRect && (
        <div
          className="absolute border-2 border-brand-500 bg-brand-500/10 pointer-events-none z-[10000]"
          style={{
            left: `${imageRect.x + currentRect.x * (imageRect.width / screenWidth)}px`,
            top: `${imageRect.y + currentRect.y * (imageRect.height / screenHeight)}px`,
            width: `${currentRect.width * (imageRect.width / screenWidth)}px`,
            height: `${currentRect.height * (imageRect.height / screenHeight)}px`,
          }}
        >
          {/* 尺寸显示 */}
          <div className="absolute -top-6 left-0 px-2 py-0.5 bg-brand-500 text-white text-xs rounded whitespace-nowrap">
            {Math.round(currentRect.width)} × {Math.round(currentRect.height)}
          </div>
        </div>
      )}

      {/* 取消按钮 */}
      <button
        onClick={onCancel}
        className="fixed top-4 right-4 p-2 rounded-lg bg-surface/90 backdrop-blur border border-border hover:bg-surface-hover transition-colors z-[10000]"
      >
        <X className="w-4 h-4" />
      </button>
    </div>
  );
}
