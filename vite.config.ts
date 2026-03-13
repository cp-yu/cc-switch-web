import path from "node:path";
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { codeInspectorPlugin } from "code-inspector-plugin";

const backendPort = Number(process.env.CC_SWITCH_PORT || 17666);

export default defineConfig(({ command }) => ({
  root: "src",
  plugins: [
    command === "serve" &&
      codeInspectorPlugin({
        bundler: "vite",
      }),
    react(),
  ].filter(Boolean),
  base: "./",
  build: {
    outDir: "../dist",
    emptyOutDir: true,
  },
  server: {
    port: 3000,
    strictPort: true,
    proxy: {
      "/api": {
        target: `http://127.0.0.1:${backendPort}`,
        changeOrigin: true,
        ws: true,
      },
    },
  },
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  clearScreen: false,
  envPrefix: ["VITE_", "TAURI_"],
}));
