import tailwindcss from "@tailwindcss/vite";
import { defineConfig } from "vite";

const { SERVER_WS_ADDR, PORT } = process.env;

function parseInteger(value: string | undefined, defaultValue: number): number {
  const parsed = Number.parseInt(value ?? "", 10);
  return Number.isNaN(parsed) ? defaultValue : parsed;
}

const port = parseInteger(PORT, 8080);
const websocketAddress = SERVER_WS_ADDR ?? "127.0.0.1:8081";
const websocketProxy = { target: `ws://${websocketAddress}`, ws: true };

export default defineConfig({
  root: ".",
  publicDir: "public",
  plugins: [tailwindcss()],
  build: {
    outDir: "dist",
    emptyOutDir: true,
    modulePreload: { polyfill: false },
    rolldownOptions: {
      output: {
        codeSplitting: {
          groups: [
            {
              name: "a2ui-renderer",
              test: /node_modules[\\/]@a2ui[\\/]lit[\\/]/,
            },
            {
              name: "a2ui-core",
              test: /node_modules[\\/]@a2ui[\\/]web_core[\\/]/,
            },
          ],
        },
      },
    },
  },
  worker: {
    format: "es",
  },
  server: {
    host: "127.0.0.1",
    port,
    strictPort: true,
    proxy: { "/ws": websocketProxy },
  },
  preview: {
    host: "127.0.0.1",
    port,
    strictPort: true,
    proxy: { "/ws": websocketProxy },
  },
});
