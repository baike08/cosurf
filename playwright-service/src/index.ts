import { BrowserManager } from "./browser-manager.js";
import { createServer } from "./server.js";

const PORT = parseInt(process.env.COSURF_PLAYWRIGHT_PORT ?? "3100", 10);
const HOST = process.env.COSURF_PLAYWRIGHT_HOST ?? "127.0.0.1";

async function main() {
  const browserManager = new BrowserManager();
  const app = createServer(browserManager);

  const shutdown = async (signal: string) => {
    app.log.info(`Received ${signal}, shutting down gracefully...`);
    await app.close();
    await browserManager.closeAll();
    process.exit(0);
  };

  process.on("SIGINT", () => shutdown("SIGINT"));
  process.on("SIGTERM", () => shutdown("SIGTERM"));

  try {
    await app.listen({ port: PORT, host: HOST });
    app.log.info(`Playwright sidecar service running on http://${HOST}:${PORT}`);
  } catch (err) {
    app.log.error(err);
    process.exit(1);
  }
}

main();
