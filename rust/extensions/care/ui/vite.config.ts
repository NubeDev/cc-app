import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { defineExtConfig } from "@nube/ext-ui-sdk/vite";

// The care ext UI remote: lib-mode ESM `remoteEntry.js` with React externalised
// (so the host's single React resolves via its import map — no second copy /
// "Invalid hook call"). `defineExtConfig` is a CONFIG FRAGMENT spread into
// `defineConfig` (NOT a plugin) — it sets `build.lib`/`rollupOptions.external`/
// `cssCodeSplit`. Our entry is `remoteEntry.tsx`.
export default defineConfig({
  plugins: [react()],
  ...defineExtConfig({ entry: "src/remoteEntry.tsx" }),
});
