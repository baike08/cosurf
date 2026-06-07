export interface DownloadItem {
  id: string;
  url: string;
  filename: string;
  mimeType: string;
  totalBytes: number;
  receivedBytes: number;
  startTime: string;
  endTime?: string;
  state: "in_progress" | "completed" | "interrupted" | "cancelled";
  savePath: string;
  error?: string;
}

export interface DownloadState {
  downloads: DownloadItem[];
  isDownloading: boolean;

  addDownload: (item: Omit<DownloadItem, "id" | "startTime">) => void;
  updateDownload: (id: string, updates: Partial<DownloadItem>) => void;
  removeDownload: (id: string) => void;
  clearCompleted: () => void;
  pauseDownload: (id: string) => void;
  resumeDownload: (id: string) => void;
  cancelDownload: (id: string) => void;
  openFile: (id: string) => void;
  showInFolder: (id: string) => void;
}
