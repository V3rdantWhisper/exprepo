import { defineConfig } from "vite";
import solid from "vite-plugin-solid";

// @tauri-apps/cli runs this; Tauri expects a fixed dev port and no auto-clear so
// Rust compiler errors stay visible in the terminal.
const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [solid()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? { protocol: "ws", host, port: 1421 }
      : undefined,
    watch: {
      // Don't watch the Rust side; tauri handles rebuilds.
      ignored: ["**/src-tauri/**"],
    },
  },
  build: {
    target: "esnext",
    outDir: "dist",
    emptyOutDir: true,
  },
});
