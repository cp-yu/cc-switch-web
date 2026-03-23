import path from "node:path";
import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
      "@platform/bootstrap": path.resolve(__dirname, "./src/platform/bootstrap.web.ts"),
      "@platform/transport-impl": path.resolve(
        __dirname,
        "./src/lib/transport/transport.impl.tauri.ts",
      ),
      "@platform/updater-impl": path.resolve(__dirname, "./src/lib/updater.web.ts"),
      "@platform/platform-paths-impl": path.resolve(
        __dirname,
        "./src/lib/platform-paths.web.ts",
      ),
    },
  },
  test: {
    environment: "jsdom",
    setupFiles: ["./tests/setupGlobals.ts", "./tests/setupTests.ts"],
    globals: true,
    coverage: {
      reporter: ["text", "lcov"],
    },
  },
});
