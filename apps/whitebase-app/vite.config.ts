import { defineConfig } from "vite";

// @ts-expect-error process is a Node.js global
const host = process.env.TAURI_DEV_HOST;

// GitHub Pages buildでは "/Whitebase/" を指定する。
// 通常のWeb開発・Tauriでは "/" を使用する。
// @ts-expect-error process is a Node.js global
const base = process.env.WHITEBASE_WEB_BASE || "/";

// https://vite.dev/config/
export default defineConfig(async () => ({
  base,

  // Vite options tailored for Tauri development and only applied
  // in `tauri dev` or `tauri build`
  clearScreen: false,

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
      ignored: ["**/src-tauri/**"],
    },
  },
}));