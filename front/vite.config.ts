import { svelte } from "@sveltejs/vite-plugin-svelte";
import { defineConfig } from "vitest/config";

// Tauri loads the dev server at a fixed port (src-tauri/tauri.conf.json, devUrl).
export default defineConfig({
  plugins: [svelte()],
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
    // The fonts live in crates/export/fonts, shared with the timeline export.
    fs: { allow: [".", "../crates/export/fonts"] },
  },
  build: {
    target: ["safari16", "chrome110"],
    // Fonts are files, never inlined as data: URLs.
    assetsInlineLimit: 0,
  },
  test: {
    include: ["src/**/*.test.ts"],
  },
});
