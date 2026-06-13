import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";

interface HistoryEntry {
  id: string;
  title: string;
  url: string;
  visitedAt: string;
}

interface HistoryState {
  entries: HistoryEntry[];
  loading: boolean;
  searchQuery: string;
  setSearchQuery: (query: string) => void;
  loadHistory: (limit?: number) => Promise<void>;
  searchHistory: (query: string) => Promise<void>;
  addHistory: (title: string, url: string) => Promise<void>;
  deleteEntry: (id: string) => Promise<void>;
  clearAll: () => Promise<void>;
}

export const useHistoryStore = create<HistoryState>((set, get) => ({
  entries: [],
  loading: false,
  searchQuery: "",

  setSearchQuery: (query) => {
    set({ searchQuery: query });
    if (query.trim()) {
      get().searchHistory(query);
    } else {
      get().loadHistory();
    }
  },

  loadHistory: async (limit = 100) => {
    set({ loading: true });
    try {
      const entries = await invoke<HistoryEntry[]>("list_history", {
        limit,
        offset: 0,
      });
      set({ entries, loading: false });
    } catch (error) {
      console.error("[HistoryStore] Failed to load history:", error);
      set({ loading: false });
    }
  },

  searchHistory: async (query: string) => {
    if (!query.trim()) {
      get().loadHistory();
      return;
    }
    set({ loading: true });
    try {
      const entries = await invoke<HistoryEntry[]>("search_history", {
        query,
        limit: 100,
      });
      set({ entries, loading: false });
    } catch (error) {
      console.error("[HistoryStore] Failed to search history:", error);
      set({ loading: false });
    }
  },

  addHistory: async (title: string, url: string) => {
    // 跳过内部页面和空白页
    if (!url || url === "about:blank" || url.startsWith("cosurf://")) return;
    try {
      await invoke("add_history", {
        request: { title: title || url, url },
      });
      // 重新加载历史
      const { searchQuery } = get();
      if (searchQuery.trim()) {
        get().searchHistory(searchQuery);
      } else {
        get().loadHistory();
      }
    } catch (error) {
      console.error("[HistoryStore] Failed to add history:", error);
    }
  },

  deleteEntry: async (id: string) => {
    try {
      await invoke("delete_history_entry", { id });
      set((state) => ({
        entries: state.entries.filter((e) => e.id !== id),
      }));
    } catch (error) {
      console.error("[HistoryStore] Failed to delete entry:", error);
    }
  },

  clearAll: async () => {
    try {
      await invoke("clear_history");
      set({ entries: [] });
    } catch (error) {
      console.error("[HistoryStore] Failed to clear history:", error);
    }
  },
}));
