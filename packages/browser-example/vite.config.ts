import tailwindcss from "@tailwindcss/vite";
import { defineConfig } from "vite";

const { PORT } = process.env;

function parseInteger(value: string | undefined, defaultValue: number): number {
  const parsed = Number.parseInt(value ?? "", 10);
  return Number.isNaN(parsed) ? defaultValue : parsed;
}

const port = parseInteger(PORT, 8080);

export default defineConfig({
  root: ".",
  publicDir: "public",
  plugins: [tailwindcss()],
  build: {
    outDir: "dist",
    emptyOutDir: true,
    modulePreload: { polyfill: false },
  },
  worker: {
    format: "es",
  },
  server: {
    host: "127.0.0.1",
    port,
    strictPort: true,
  },
  preview: {
    host: "127.0.0.1",
    port,
    strictPort: true,
  },
});
