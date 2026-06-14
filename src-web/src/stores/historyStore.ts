import { create } from "zustand";
import { db } from "@/lib/api";

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
      const entries = await db.listHistory(limit, 0);
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
      const entries = await db.searchHistory(query, 100);
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
      await db.addHistory(title || url, url);
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
      await db.deleteHistoryEntry(id);
      set((state) => ({
        entries: state.entries.filter((e) => e.id !== id),
      }));
    } catch (error) {
      console.error("[HistoryStore] Failed to delete entry:", error);
    }
  },

  clearAll: async () => {
    try {
      await db.clearHistory();
      set({ entries: [] });
    } catch (error) {
      console.error("[HistoryStore] Failed to clear history:", error);
    }
  },
}));
