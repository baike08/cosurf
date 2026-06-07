import { type Browser, type BrowserContext, type Page, chromium } from "playwright";
import { v4 as uuidv4 } from "uuid";
import type { LaunchOptions, SessionInfo } from "./types.js";

interface ManagedSession {
  id: string;
  browser: Browser;
  context: BrowserContext;
  page: Page;
  createdAt: Date;
}

const DEFAULT_VIEWPORT = { width: 1280, height: 720 };

export class BrowserManager {
  private sessions = new Map<string, ManagedSession>();

  async launch(options: LaunchOptions = {}): Promise<{ sessionId: string; page: Page }> {
    const sessionId = uuidv4();

    const browser = await chromium.launch({
      headless: options.headless ?? true,
    });

    const context = await browser.newContext({
      viewport: options.viewport ?? DEFAULT_VIEWPORT,
      userAgent: options.userAgent,
      locale: options.locale ?? "zh-CN",
      timezoneId: options.timezoneId ?? "Asia/Shanghai",
      proxy: options.proxy,
    });

    const page = await context.newPage();

    this.sessions.set(sessionId, {
      id: sessionId,
      browser,
      context,
      page,
      createdAt: new Date(),
    });

    return { sessionId, page };
  }

  getSession(sessionId: string): ManagedSession | undefined {
    return this.sessions.get(sessionId);
  }

  getPage(sessionId: string): Page {
    const session = this.sessions.get(sessionId);
    if (!session) {
      throw new Error(`Session not found: ${sessionId}`);
    }
    return session.page;
  }

  getSessionInfo(sessionId: string): SessionInfo | null {
    const session = this.sessions.get(sessionId);
    if (!session) return null;

    const viewport = session.page.viewportSize() ?? DEFAULT_VIEWPORT;
    return {
      sessionId: session.id,
      url: session.page.url(),
      title: "",
      viewport,
      createdAt: session.createdAt.toISOString(),
    };
  }

  listSessions(): SessionInfo[] {
    const result: SessionInfo[] = [];
    for (const [id] of this.sessions) {
      const info = this.getSessionInfo(id);
      if (info) result.push(info);
    }
    return result;
  }

  async closeSession(sessionId: string): Promise<void> {
    const session = this.sessions.get(sessionId);
    if (!session) return;

    this.sessions.delete(sessionId);

    try {
      await session.context.close();
    } catch {
      // context may already be closed
    }
    try {
      await session.browser.close();
    } catch {
      // browser may already be closed
    }
  }

  async closeAll(): Promise<void> {
    const ids = [...this.sessions.keys()];
    await Promise.all(ids.map((id) => this.closeSession(id)));
  }

  get sessionCount(): number {
    return this.sessions.size;
  }
}
