export interface LaunchOptions {
  headless?: boolean;
  viewport?: { width: number; height: number };
  userAgent?: string;
  locale?: string;
  timezoneId?: string;
  proxy?: { server: string; username?: string; password?: string };
}

export interface NavigateRequest {
  url: string;
  timeout?: number;
  waitUntil?: "load" | "domcontentloaded" | "networkidle" | "commit";
}

export interface ClickRequest {
  selector?: string;
  position?: { x: number; y: number };
  button?: "left" | "right" | "middle";
  clickCount?: number;
  timeout?: number;
}

export interface TypeRequest {
  selector: string;
  text: string;
  delay?: number;
  timeout?: number;
}

export interface FillRequest {
  selector: string;
  value: string;
  timeout?: number;
}

export interface SelectRequest {
  selector: string;
  values: string | string[];
  timeout?: number;
}

export interface ScrollRequest {
  direction: "up" | "down" | "left" | "right";
  amount?: number;
  selector?: string;
}

export interface ScreenshotOptions {
  fullPage?: boolean;
  selector?: string;
  type?: "png" | "jpeg";
  quality?: number;
}

export interface EvaluateRequest {
  expression: string;
  timeout?: number;
}

export interface WaitForRequest {
  selector?: string;
  url?: string;
  timeout?: number;
  state?: "attached" | "detached" | "visible" | "hidden";
}

export interface GetContentRequest {
  format: "text" | "html" | "markdown";
  selector?: string;
}

export interface CookieRequest {
  urls?: string[];
}

export interface SetCookieEntry {
  name: string;
  value: string;
  url?: string;
  domain?: string;
  path?: string;
  expires?: number;
  httpOnly?: boolean;
  secure?: boolean;
  sameSite?: "Strict" | "Lax" | "None";
}

export interface SetCookiesRequest {
  cookies: SetCookieEntry[];
}

export interface ApiResponse<T = unknown> {
  success: boolean;
  data?: T;
  error?: string;
}

export interface SessionInfo {
  sessionId: string;
  url: string;
  title: string;
  viewport: { width: number; height: number };
  createdAt: string;
}

export interface LaunchResponse {
  sessionId: string;
}

export interface NavigateResponse {
  url: string;
  title: string;
  status: number | null;
}

export interface ScreenshotResponse {
  data: string;
  mimeType: string;
}

export interface ContentResponse {
  content: string;
  format: string;
  url: string;
  title: string;
}

export interface ElementInfo {
  tagName: string;
  text: string;
  attributes: Record<string, string>;
  boundingBox: { x: number; y: number; width: number; height: number } | null;
}

export interface AccessibilityNode {
  role: string;
  name: string;
  value?: string;
  description?: string;
  children?: AccessibilityNode[];
}
