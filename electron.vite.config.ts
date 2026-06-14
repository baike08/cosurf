/**
 * CoSurf Electron-Vite 构建配置
 * 
 * 三段式构建:
 * - main:     Electron 主进程 (electron/*.ts -> out/main/*.js)
 * - preload:  Preload 脚本 (electron/preload.ts -> out/preload/*.js)
 * - renderer: React 前端 (src-web/ -> out/renderer/)
 */

import { resolve } from 'path';
import { defineConfig, externalizeDepsPlugin } from 'electron-vite';
import react from '@vitejs/plugin-react';

export default defineConfig({
  // ===== 主进程配置 =====
  main: {
    plugins: [externalizeDepsPlugin()],
    build: {
      outDir: 'out/main',
      rollupOptions: {
        input: {
          main: resolve(__dirname, 'electron/main.ts'),
        },
      },
    },
    resolve: {
      alias: {
        '@electron': resolve(__dirname, 'electron'),
      },
    },
  },

  // ===== Preload 脚本配置 =====
  preload: {
    plugins: [externalizeDepsPlugin()],
    build: {
      outDir: 'out/preload',
      rollupOptions: {
        input: {
          preload: resolve(__dirname, 'electron/preload.ts'),
          'content-preload': resolve(__dirname, 'electron/content-preload.ts'),
        },
      },
    },
  },

  // ===== 渲染进程配置 (复用现有 src-web) =====
  renderer: {
    root: resolve(__dirname, 'src-web'),
    build: {
      outDir: resolve(__dirname, 'out/renderer'),
      rollupOptions: {
        input: {
          index: resolve(__dirname, 'src-web/index.html'),
        },
      },
    },
    resolve: {
      alias: {
        '@': resolve(__dirname, 'src-web/src'),
      },
    },
    plugins: [react()],
    server: {
      port: 1420,
      strictPort: true,
    },
    // 定义环境变量
    define: {
      'import.meta.env.ELECTRON': 'true',
    },
  },
});
