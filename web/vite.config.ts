import { defineConfig, type Plugin } from "vite";
import solidPlugin from "vite-plugin-solid";
import tailwindcss from "@tailwindcss/vite";
import path from "path";

// Read at build time so Capacitor live-reload works without changing source.
const isCapacitorDev = process.env.CAPACITOR_DEV_SERVER_URL !== undefined;
const isDemoMode = process.env.VITE_DEMO_MODE === "true";

const MOCK_INDEX = path.resolve(__dirname, "src/api/mock/index.ts");

/**
 * In demo mode, redirect every import of the real API client
 * (src/api/client.ts) to the in-memory mock barrel.
 * Using a plugin resolveId hook is more reliable than path aliases
 * in Vite 8 (rolldown) for intercepting relative/aliased imports.
 */
function demoModePlugin(): Plugin {
  return {
    name: "rustvault-demo-mode",
    enforce: "pre",
    resolveId(id, importer) {
      if (!isDemoMode) return null;
      // Case 1: absolute-like alias  ~/api/client
      if (id === "~/api/client") return MOCK_INDEX;
      // Case 2: ends with the file path (absolute resolution)
      if (id.endsWith("/src/api/client") || id.endsWith("/src/api/client.ts")) return MOCK_INDEX;
      // Case 3: relative import "./client" from src/api/index.ts
      if (id === "./client" && importer && importer.includes("/src/api/")) return MOCK_INDEX;
      return null;
    },
  };
}

export default defineConfig({
  plugins: [demoModePlugin(), solidPlugin(), tailwindcss()],
  define: {
    // Boolean flag available at compile time: `if (__DEMO_MODE__) { ... }`
    // Tree-shaking eliminates the unused branch in production builds.
    __DEMO_MODE__: JSON.stringify(isDemoMode),
  },
  server: {
    port: 3000,
    proxy: isDemoMode
      ? {} // No proxy in demo mode — all API calls go to the in-memory mock.
      : {
          "/api": {
            target: "http://localhost:8080",
            changeOrigin: true,
          },
        },
  },
  build: {
    // Capacitor webview supports modern JS — no need to target legacy browsers.
    target: "esnext",
    outDir: "dist",
    // Ensure asset paths are relative so Capacitor can load them from file://.
    assetsInlineLimit: isCapacitorDev ? 0 : 4096,
    rollupOptions: {
      output: {
        manualChunks(id) {
          if (id.includes("node_modules/echarts") || id.includes("node_modules/zrender")) {
            return "charts";
          }
          if (id.includes("node_modules/@kobalte")) {
            return "kobalte";
          }
          if (
            id.includes("node_modules/solid-js") ||
            id.includes("node_modules/@solidjs")
          ) {
            return "vendor";
          }
          if (id.includes("node_modules/@capacitor")) {
            return "capacitor";
          }
        },
      },
    },
  },
  resolve: {
    alias: {
      "~": "/src",
    },
  },
});
