import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import path from "path";

const host = process.env.TAURI_DEV_HOST || process.env.ELECTRON_DEV_HOST;

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      ignored: ["**/src-tauri/**", "**/native/**", "**/electron/**"],
    },
  },
  envPrefix: ["VITE_", "TAURI_", "ELECTRON_"],
  build: {
    target: "chrome120",
    minify: "esbuild",
    sourcemap: !!process.env.VITE_DEBUG,
  },
});
