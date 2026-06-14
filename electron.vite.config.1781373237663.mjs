// electron.vite.config.ts
import { resolve } from "path";
import { defineConfig, externalizeDepsPlugin } from "electron-vite";
import react from "@vitejs/plugin-react";
var __electron_vite_injected_dirname = "D:\\coding-harness\\CoSurf";
var electron_vite_config_default = defineConfig({
  // ===== 主进程配置 =====
  main: {
    plugins: [externalizeDepsPlugin()],
    build: {
      outDir: "out/main",
      rollupOptions: {
        input: {
          main: resolve(__electron_vite_injected_dirname, "electron/main.ts")
        }
      }
    },
    resolve: {
      alias: {
        "@electron": resolve(__electron_vite_injected_dirname, "electron")
      }
    }
  },
  // ===== Preload 脚本配置 =====
  preload: {
    plugins: [externalizeDepsPlugin()],
    build: {
      outDir: "out/preload",
      rollupOptions: {
        input: {
          preload: resolve(__electron_vite_injected_dirname, "electron/preload.ts"),
          "content-preload": resolve(__electron_vite_injected_dirname, "electron/content-preload.ts")
        }
      }
    }
  },
  // ===== 渲染进程配置 (复用现有 src-web) =====
  renderer: {
    root: resolve(__electron_vite_injected_dirname, "src-web"),
    build: {
      outDir: resolve(__electron_vite_injected_dirname, "out/renderer"),
      rollupOptions: {
        input: {
          index: resolve(__electron_vite_injected_dirname, "src-web/index.html")
        }
      }
    },
    resolve: {
      alias: {
        "@": resolve(__electron_vite_injected_dirname, "src-web/src")
      }
    },
    plugins: [react()],
    server: {
      port: 1420,
      strictPort: true
    },
    // 定义环境变量
    define: {
      "import.meta.env.ELECTRON": "true"
    }
  }
});
export {
  electron_vite_config_default as default
};
