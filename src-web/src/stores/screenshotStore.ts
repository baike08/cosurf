import { create } from "zustand";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@/lib/tauri";

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
    // 监听全屏截图完成事件
    const unlistenFullscreen = await listen<{
      image: string;
      width: number;
      height: number;
    }>("screenshot-fullscreen-captured", (event) => {
      const { image, width, height } = event.payload;
      set({ showSelector: true, fullScreenImage: image, screenWidth: width, screenHeight: height, toast: null });
    });

    // 监听裁剪后截图完成事件
    const unlistenCaptured = await listen<{
      image: string;
      width: number;
      height: number;
    }>("screenshot-captured", (event) => {
      const { image, width, height } = event.payload;
      set({ isOpen: true, showSelector: false, fullScreenImage: null, imageData: image, imageWidth: width, imageHeight: height, toast: null });
    });

    return () => {
      unlistenFullscreen();
      unlistenCaptured();
    };
  },

  close: () => {
    set({ isOpen: false, showSelector: false, fullScreenImage: null, imageData: null, toast: null });
  },

  captureRegion: (rect) => {
    const { fullScreenImage, screenWidth, screenHeight } = get();
    if (!fullScreenImage) return;

    // 调用后端裁剪截图
    invoke("capture_region_from_base64", {
      base64Data: fullScreenImage,
      x: Math.round(rect.x),
      y: Math.round(rect.y),
      width: Math.round(rect.width),
      height: Math.round(rect.height),
      screenWidth,
      screenHeight,
    }).catch((error) => {
      console.error("Capture failed:", error);
      set({ showSelector: false, fullScreenImage: null, toast: "✗ 截图失败" });
    });
  },

  copyToClipboard: async () => {
    const { imageData } = get();
    if (!imageData) return;
    set({ copying: true });
    try {
      await invoke("copy_screenshot_to_clipboard", {
        base64Data: imageData,
      });
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
      const { save } = await import("@tauri-apps/plugin-dialog");
      const filePath = await save({
        title: "保存截图",
        defaultPath: `screenshot_${Date.now()}.png`,
        filters: [{ name: "PNG 图片", extensions: ["png"] }],
      });
      if (filePath) {
        await invoke("save_screenshot", {
          base64Data: imageData,
          path: filePath,
        });
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
