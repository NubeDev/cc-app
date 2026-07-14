import { defineConfig, devices } from "@playwright/test";

// Playwright drives the REAL shell against the REAL node (mobile-shell-scope
// §"Rule 9": UI tests run against a real node — no mocks). The dev server on
// :5391 (with the vite-dev-auth seam) + the cc-node gateway on :8391 must be
// running: `make dev` (or the node + `pnpm dev`) then `make e2e-ui`. The base
// URL points at the running shell; specs never stub the gateway.
const BASE_URL = process.env.CC_UI_URL || "http://127.0.0.1:5391";

export default defineConfig({
  testDir: "./e2e",
  timeout: 30_000,
  expect: { timeout: 10_000 },
  fullyParallel: false,
  reporter: [["list"]],
  use: {
    baseURL: BASE_URL,
    trace: "retain-on-failure",
    // phone-first is the design target (DESIGN.md); test at the 360px viewport.
    viewport: { width: 390, height: 844 },
  },
  projects: [
    {
      name: "mobile-chromium",
      use: { ...devices["Pixel 5"] },
    },
  ],
});
