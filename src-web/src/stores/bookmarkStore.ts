import { create } from "zustand";
import { db } from "@/lib/api";

interface Bookmark {
  id: string;
  title: string;
  url: string;
  favicon?: string;
  folderId?: string;
  order: number;
  createdAt: string;
}

interface BookmarkFolder {
  id: string;
  name: string;
  parentId?: string;
  order: number;
}

interface BookmarkState {
  bookmarks: Bookmark[];
  folders: BookmarkFolder[];
  currentFolderId: string | null; // null = 根目录
  loading: boolean;
  searchQuery: string;

  // Actions
  loadBookmarks: (folderId?: string | null) => Promise<void>;
  loadFolders: () => Promise<void>;
  setCurrentFolder: (folderId: string | null) => void;
  setSearchQuery: (query: string) => void;
  addBookmark: (title: string, url: string, favicon?: string, folderId?: string) => Promise<Bookmark | null>;
  deleteBookmark: (id: string) => Promise<void>;
  addFolder: (name: string, parentId?: string) => Promise<BookmarkFolder | null>;
  deleteFolder: (id: string) => Promise<void>;
  isBookmarked: (url: string) => boolean;
  removeBookmarkByUrl: (url: string) => Promise<void>;
}

export const useBookmarkStore = create<BookmarkState>((set, get) => ({
  bookmarks: [],
  folders: [],
  currentFolderId: null,
  loading: false,
  searchQuery: "",

  loadBookmarks: async (folderId?: string | null) => {
    set({ loading: true });
    try {
      const fid = folderId !== undefined ? folderId : get().currentFolderId;
      const bookmarks = await db.listBookmarks(fid || null);
      set({ bookmarks, loading: false });
    } catch (error) {
      console.error("[BookmarkStore] Failed to load bookmarks:", error);
      set({ loading: false });
    }
  },

  loadFolders: async () => {
    try {
      const folders = await db.listBookmarkFolders(null);
      set({ folders });
    } catch (error) {
      console.error("[BookmarkStore] Failed to load folders:", error);
    }
  },

  setCurrentFolder: (folderId) => {
    set({ currentFolderId: folderId });
    get().loadBookmarks(folderId);
  },

  setSearchQuery: (query) => {
    set({ searchQuery: query });
  },

  addBookmark: async (title, url, favicon, folderId) => {
    try {
      const bookmark = await db.createBookmark(title, url, favicon || null, folderId || null);
      // 重新加载当前文件夹的书签
      await get().loadBookmarks();
      return bookmark;
    } catch (error) {
      console.error("[BookmarkStore] Failed to add bookmark:", error);
      return null;
    }
  },

  deleteBookmark: async (id) => {
    try {
      await db.deleteBookmark(id);
      set((state) => ({
        bookmarks: state.bookmarks.filter((b) => b.id !== id),
      }));
    } catch (error) {
      console.error("[BookmarkStore] Failed to delete bookmark:", error);
    }
  },

  addFolder: async (name, parentId) => {
    try {
      const folder = await db.createBookmarkFolder(name, parentId || null);
      await get().loadFolders();
      return folder;
    } catch (error) {
      console.error("[BookmarkStore] Failed to add folder:", error);
      return null;
    }
  },

  deleteFolder: async (id) => {
    try {
      await db.deleteBookmarkFolder(id);
      set((state) => ({
        folders: state.folders.filter((f) => f.id !== id),
      }));
      // 如果当前在删除的文件夹中，切回根目录
      if (get().currentFolderId === id) {
        get().setCurrentFolder(null);
      }
    } catch (error) {
      console.error("[BookmarkStore] Failed to delete folder:", error);
    }
  },

  isBookmarked: (url) => {
    return get().bookmarks.some((b) => b.url === url);
  },

  removeBookmarkByUrl: async (url) => {
    const bookmark = get().bookmarks.find((b) => b.url === url);
    if (bookmark) {
      await get().deleteBookmark(bookmark.id);
    }
  },
}));
