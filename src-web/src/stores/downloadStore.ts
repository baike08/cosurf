import { create } from "zustand";
import type { DownloadItem } from "@cosurf/shared";
import { generateId } from "@/lib/utils";

interface DownloadState {
  downloads: DownloadItem[];

  addDownload: (item: Omit<DownloadItem, "id" | "startTime">) => void;
  updateDownload: (id: string, updates: Partial<DownloadItem>) => void;
  removeDownload: (id: string) => void;
  clearCompleted: () => void;
  cancelDownload: (id: string) => void;

  // 计算属性
  activeDownloads: number;
  hasDownloads: boolean;
}

export const useDownloadStore = create<DownloadState>()((set, get) => ({
  downloads: [],

  addDownload: (item) => {
    set((state) => ({
      downloads: [
        {
          ...item,
          id: generateId(),
          startTime: new Date().toISOString(),
        },
        ...state.downloads,
      ],
    }));
  },

  updateDownload: (id, updates) => {
    set((state) => ({
      downloads: state.downloads.map((d) =>
        d.id === id ? { ...d, ...updates } : d,
      ),
    }));
  },

  removeDownload: (id) => {
    set((state) => ({
      downloads: state.downloads.filter((d) => d.id !== id),
    }));
  },

  clearCompleted: () => {
    set((state) => ({
      downloads: state.downloads.filter(
        (d) => d.state !== "completed" && d.state !== "cancelled",
      ),
    }));
  },

  cancelDownload: (id) => {
    set((state) => ({
      downloads: state.downloads.map((d) =>
        d.id === id ? { ...d, state: "cancelled" as const } : d,
      ),
    }));
  },

  get activeDownloads() {
    return get().downloads.filter((d) => d.state === "in_progress").length;
  },

  get hasDownloads() {
    return get().downloads.length > 0;
  },
}));
