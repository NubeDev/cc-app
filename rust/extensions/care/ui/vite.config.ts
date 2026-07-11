import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { extUiSdk } from "@nube/ext-ui-sdk/vite";

export default defineConfig({
  plugins: [react(), extUiSdk()],
  build: { lib: { entry: "src/remoteEntry.tsx", formats: ["es"] } },
});