import Fastify, { type FastifyInstance } from "fastify";
import { BrowserManager } from "./browser-manager.js";
import * as actions from "./actions.js";
import type { ApiResponse } from "./types.js";

export function createServer(browserManager: BrowserManager): FastifyInstance {
  const app = Fastify({ logger: true });

  function withSession(fn: (page: import("playwright").Page, body: any, params: any) => Promise<any>) {
    return async (request: any, reply: any) => {
      const { sessionId } = request.params as { sessionId: string };
      try {
        const page = browserManager.getPage(sessionId);
        const result = await fn(page, request.body, request.params);
        const response: ApiResponse = { success: true, data: result };
        return reply.send(response);
      } catch (err: any) {
        const response: ApiResponse = { success: false, error: err.message };
        return reply.status(err.message.includes("not found") ? 404 : 500).send(response);
      }
    };
  }

  app.get("/health", async () => {
    return {
      success: true,
      data: {
        status: "ok",
        sessions: browserManager.sessionCount,
        uptime: process.uptime(),
      },
    };
  });

  app.post("/browser/launch", async (request) => {
    const options = (request.body ?? {}) as Record<string, unknown>;
    const { sessionId } = await browserManager.launch(options);
    const response: ApiResponse = { success: true, data: { sessionId } };
    return response;
  });

  app.get("/browser/sessions", async () => {
    const sessions = browserManager.listSessions();
    const response: ApiResponse = { success: true, data: sessions };
    return response;
  });

  app.get("/browser/:sessionId/info", async (request) => {
    const { sessionId } = request.params as { sessionId: string };
    const info = browserManager.getSessionInfo(sessionId);
    if (!info) {
      const response: ApiResponse = { success: false, error: `Session not found: ${sessionId}` };
      return response;
    }
    const response: ApiResponse = { success: true, data: info };
    return response;
  });

  app.post("/browser/:sessionId/navigate", withSession(async (page, body) => {
    return actions.navigate(page, body);
  }));

  app.post("/browser/:sessionId/click", withSession(async (page, body) => {
    await actions.click(page, body);
    return { url: page.url() };
  }));

  app.post("/browser/:sessionId/type", withSession(async (page, body) => {
    await actions.type(page, body);
    return { ok: true };
  }));

  app.post("/browser/:sessionId/fill", withSession(async (page, body) => {
    await actions.fill(page, body);
    return { ok: true };
  }));

  app.post("/browser/:sessionId/select", withSession(async (page, body) => {
    const selected = await actions.select(page, body);
    return { selected };
  }));

  app.post("/browser/:sessionId/scroll", withSession(async (page, body) => {
    await actions.scroll(page, body);
    return { ok: true };
  }));

  app.post("/browser/:sessionId/screenshot", withSession(async (page, body) => {
    return actions.screenshot(page, body ?? {});
  }));

  app.post("/browser/:sessionId/evaluate", withSession(async (page, body) => {
    const result = await actions.evaluate(page, body);
    return { result };
  }));

  app.post("/browser/:sessionId/wait", withSession(async (page, body) => {
    await actions.waitFor(page, body);
    return { ok: true };
  }));

  app.post("/browser/:sessionId/content", withSession(async (page, body) => {
    return actions.getContent(page, body);
  }));

  app.get("/browser/:sessionId/cookies", withSession(async (page) => {
    return actions.getCookies(page);
  }));

  app.post("/browser/:sessionId/cookies", withSession(async (page, body) => {
    await actions.setCookies(page, body);
    return { ok: true };
  }));

  app.post("/browser/:sessionId/elements", withSession(async (page, body) => {
    const { selector } = body as { selector: string };
    return actions.getElements(page, selector);
  }));

  app.get("/browser/:sessionId/accessibility", withSession(async (page) => {
    return actions.getAccessibilityTree(page);
  }));

  app.post("/browser/:sessionId/press", withSession(async (page, body) => {
    const { key } = body as { key: string };
    await actions.pressKey(page, key);
    return { ok: true };
  }));

  app.post("/browser/:sessionId/hover", withSession(async (page, body) => {
    const { selector } = body as { selector: string };
    await actions.hover(page, selector);
    return { ok: true };
  }));

  app.post("/browser/:sessionId/drag", withSession(async (page, body) => {
    const { source, target } = body as { source: string; target: string };
    await actions.dragAndDrop(page, source, target);
    return { ok: true };
  }));

  app.post("/browser/:sessionId/upload", withSession(async (page, body) => {
    const { selector, filePaths } = body as { selector: string; filePaths: string[] };
    await actions.uploadFile(page, selector, filePaths);
    return { ok: true };
  }));

  app.post("/browser/:sessionId/back", withSession(async (page) => {
    return actions.goBack(page);
  }));

  app.post("/browser/:sessionId/forward", withSession(async (page) => {
    return actions.goForward(page);
  }));

  app.post("/browser/:sessionId/reload", withSession(async (page) => {
    return actions.reload(page);
  }));

  app.post("/browser/:sessionId/close", async (request) => {
    const { sessionId } = request.params as { sessionId: string };
    await browserManager.closeSession(sessionId);
    const response: ApiResponse = { success: true, data: { ok: true } };
    return response;
  });

  app.post("/browser/close-all", async () => {
    await browserManager.closeAll();
    const response: ApiResponse = { success: true, data: { ok: true } };
    return response;
  });

  return app;
}
