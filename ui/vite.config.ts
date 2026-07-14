import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { VitePWA } from "vite-plugin-pwa";
import { fileURLToPath, URL } from "node:url";
import { devAuth } from "./vite-dev-auth";

export default defineConfig({
  // Dev server port — kept in sync with the Makefile's UI_PORT (5391). strictPort
  // so a clash fails loudly rather than silently hopping to another port (which
  // would break the seed/e2e URLs). Moved off Vite's default 5173 to avoid
  // clashing with other local projects.
  server: { port: 5391, strictPort: true },
  resolve: {
    // `@/…` → src/ (the shadcn import alias, mirrors tsconfig paths).
    alias: { "@": fileURLToPath(new URL("./src", import.meta.url)) },
    // The host imports the care ext's remoteEntry directly in dev (same-repo,
    // same React). Dedupe so Radix hooks (shadcn primitives) never bind to a
    // second React copy resolved from the ext's own node_modules.
    dedupe: ["react", "react-dom"],
  },
  plugins: [
    // Dev-only auth seam: terminates the shell's /api/* calls and forwards to
    // the lb gateway with a cookie-held token (see vite-dev-auth.ts). No-op in
    // the production build (configureServer only runs under `vite`/dev).
    devAuth(),
    react(),
    VitePWA({
      registerType: "autoUpdate",
      manifest: {
        name: "Childcare",
        short_name: "Childcare",
        theme_color: "#000000",
        background_color: "#ffffff",
        display: "standalone",
        orientation: "portrait",
        icons: [],
      },
    }),
  ],
});