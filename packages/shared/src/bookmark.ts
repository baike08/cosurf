export interface Bookmark {
  id: string;
  title: string;
  url: string;
  favicon?: string;
  folderId?: string;
  order: number;
  createdAt: string;
}

export interface BookmarkFolder {
  id: string;
  name: string;
  parentId?: string;
  order: number;
  children: (Bookmark | BookmarkFolder)[];
}

export interface HistoryEntry {
  id: string;
  title: string;
  url: string;
  visitedAt: string;
}
