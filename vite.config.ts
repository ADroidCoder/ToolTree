import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

// Tauri 开发服务器固定端口（GitTree、ShellTree 各占了一个，这里换一个，几个项目可以同时开发）
export default defineConfig({
  plugins: [svelte()],
  clearScreen: false,
  server: { port: 1423, strictPort: true, watch: { ignored: ["**/src-tauri/**", "**/target/**"] } },
  build: { target: "es2022", chunkSizeWarningLimit: 2000 },
});
