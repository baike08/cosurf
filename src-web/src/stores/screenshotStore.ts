import { create } from "zustand";
import { on } from "@/lib/events";
import { screenshot as screenshotApi, dialog as dialogApi } from "@/lib/api";

interface ScreenshotState {
  isOpen: boolean;
  showSelector: boolean;
  fullScreenImage: string | null;  // 全屏截图
  screenWidth: number;
  screenHeight: number;
  imageData: string | null;  // 裁剪后的截图
  imageWidth: number;
  imageHeight: number;
  saving: boolean;
  copying: boolean;
  toast: string | null;

  init: () => Promise<() => void>;
  close: () => void;
  captureRegion: (rect: { x: number; y: number; width: number; height: number }) => void;
  copyToClipboard: () => Promise<void>;
  saveToFile: () => Promise<void>;
}

export const useScreenshotStore = create<ScreenshotState>((set, get) => ({
  isOpen: false,
  showSelector: false,
  fullScreenImage: null,
  screenWidth: 0,
  screenHeight: 0,
  imageData: null,
  imageWidth: 0,
  imageHeight: 0,
  saving: false,
  copying: false,
  toast: null,

  init: async () => {
    // 监听全局快捷键截图事件
    const unlistenShortcut = on<void>("shortcut:screenshot", async () => {
      console.log('[screenshotStore] 📸 Global shortcut triggered');
      try {
        // 触发全屏截图
        console.log('[screenshotStore] Calling captureFull...');
        const resultJson = await screenshotApi.captureFull();
        console.log('[screenshotStore] captureFull succeeded, result length:', resultJson.length);
        
        // 解析 JSON 结果
        const result = JSON.parse(resultJson);
        const { image, width, height } = result;
        
        console.log('[screenshotStore] Parsed result: width=', width, 'height=', height, 'image length=', image.length);
        
        // 直接触发自定义事件（不通过 IPC）
        window.dispatchEvent(new CustomEvent('screenshot-fullscreen-captured', {
          detail: { image, width, height }
        }));
      } catch (error: any) {
        console.error("[screenshotStore] Screenshot failed:", error);
        set({ toast: `✗ 截图失败: ${error.message}` });
      }
    });

    // 监听全屏截图完成事件（使用自定义事件）
    const handleFullscreenCaptured = (event: Event) => {
      const customEvent = event as CustomEvent<{
        image: string;
        width: number;
        height: number;
      }>;
      const { image, width, height } = customEvent.detail;
      set({ showSelector: true, fullScreenImage: image, screenWidth: width, screenHeight: height, toast: null });
    };
    window.addEventListener('screenshot-fullscreen-captured', handleFullscreenCaptured);

    // 监听裁剪后截图完成事件（使用自定义事件）
    const handleCaptured = (event: Event) => {
      const customEvent = event as CustomEvent<{
        image: string;
        width: number;
        height: number;
      }>;
      const { image, width, height } = customEvent.detail;
      set({ isOpen: true, showSelector: false, fullScreenImage: null, imageData: image, imageWidth: width, imageHeight: height, toast: null });
    };
    window.addEventListener('screenshot-captured', handleCaptured);

    return () => {
      unlistenShortcut();
      window.removeEventListener('screenshot-fullscreen-captured', handleFullscreenCaptured);
      window.removeEventListener('screenshot-captured', handleCaptured);
    };
  },

  close: () => {
    set({ isOpen: false, showSelector: false, fullScreenImage: null, imageData: null, toast: null });
  },

  captureRegion: async (rect) => {
    const { fullScreenImage, screenWidth, screenHeight } = get();
    if (!fullScreenImage) return;

    try {
      console.log('[screenshotStore] Capturing region:', rect);
      // 调用后端裁剪截图
      const resultJson = await screenshotApi.captureRegion(
        fullScreenImage,
        Math.round(rect.x),
        Math.round(rect.y),
        Math.round(rect.width),
        Math.round(rect.height),
        screenWidth,
        screenHeight,
      );
      
      console.log('[screenshotStore] Capture region succeeded, result length:', resultJson.length);
      
      // 解析 JSON 结果
      const result = JSON.parse(resultJson);
      const { image, width, height } = result;
      
      console.log('[screenshotStore] Parsed cropped result: width=', width, 'height=', height);
      
      // 直接触发自定义事件（不通过 IPC）
      window.dispatchEvent(new CustomEvent('screenshot-captured', {
        detail: { image, width, height }
      }));
    } catch (error: any) {
      console.error("[screenshotStore] Capture failed:", error);
      set({ showSelector: false, fullScreenImage: null, toast: `✗ 截图失败: ${error.message}` });
    }
  },

  copyToClipboard: async () => {
    const { imageData } = get();
    if (!imageData) return;
    set({ copying: true });
    try {
      await screenshotApi.copyToClipboard(imageData);
      set({ toast: "✓ 已复制到剪贴板" });
      setTimeout(() => get().close(), 800);
    } catch (error) {
      console.error("Copy failed:", error);
      set({ toast: "✗ 复制失败" });
    } finally {
      set({ copying: false });
    }
  },

  saveToFile: async () => {
    const { imageData } = get();
    if (!imageData) return;
    set({ saving: true });
    try {
      const result = await dialogApi.saveFile({
        title: "保存截图",
        defaultPath: `screenshot_${Date.now()}.png`,
        filters: [{ name: "PNG 图片", extensions: ["png"] }],
      });
      const filePath = result?.filePath;
      if (filePath) {
        await screenshotApi.save(imageData, filePath);
        set({ toast: `✓ 已保存` });
        setTimeout(() => get().close(), 800);
      }
    } catch (error) {
      console.error("Save failed:", error);
      set({ toast: "✗ 保存失败" });
    } finally {
      set({ saving: false });
    }
  },
}));
