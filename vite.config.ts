import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// Tauri 前端构建配置：固定端口，注入防抖环境变量
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
    watch: {
      // 关键：不监听 Rust 构建目录，否则 Windows 下 Cargo 写 target/ 时
      // Vite 监听到被锁定的 .exe 会抛 EBUSY 直接崩溃
      ignored: ["**/src-tauri/**", "**/dist/**", "**/node_modules/**"],
    },
  },
  build: {
    target: "chrome105",
    minify: "esbuild",
    sourcemap: false,
  },
});
