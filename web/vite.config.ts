import { defineConfig } from "vite";
import solidPlugin from "vite-plugin-solid";
import tailwindcss from "@tailwindcss/vite";

// Read at build time so Capacitor live-reload works without changing source.
const isCapacitorDev = process.env.CAPACITOR_DEV_SERVER_URL !== undefined;

export default defineConfig({
  plugins: [solidPlugin(), tailwindcss()],
  server: {
    port: 3000,
    proxy: {
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
