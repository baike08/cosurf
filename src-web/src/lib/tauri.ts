/**
 * @deprecated This module is no longer used.
 * All IPC communication has been migrated to Electron.
 * See: @/lib/api.ts, @/lib/events.ts, @/lib/electronBridge.ts
 */
export function isTauri(): boolean {
  return false;
}

export async function invoke<T>(_cmd: string, _args?: Record<string, unknown>): Promise<T> {
  throw new Error('Tauri is no longer supported. Use Electron IPC instead.');
}

export async function listen<T>(
  _event: string,
  _handler: (payload: T) => void,
): Promise<() => void> {
  return () => {};
}
