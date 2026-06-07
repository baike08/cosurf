/**
 * WebView2Container - 使用 iframe 显示网页
 * 
 * 注意: Tauri 2.x 不支持动态创建多个 WebView 实例,
 * 因此我们使用 iframe 方案,但已经修复了 shell.open 权限问题
 */

import { WebContentView } from "./WebContentView";

export function WebView2Container() {
  return <WebContentView />;
}
