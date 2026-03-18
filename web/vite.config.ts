import { defineConfig } from "vite";
import solidPlugin from "vite-plugin-solid";
import tailwindcss from "@tailwindcss/vite";

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
    target: "esnext",
    outDir: "dist",
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
