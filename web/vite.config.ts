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
        manualChunks: {
          vendor: ["solid-js", "solid-js/web", "solid-js/store", "@solidjs/router"],
          kobalte: ["@kobalte/core"],
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
