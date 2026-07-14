import { test, expect } from "@playwright/test";

// The care extension UI renders (federated mount) at /ext/<ws> — a real browser
// against the real node (no mocks — mobile-shell-scope §"Rule 9"). Guards the
// regression where `remoteEntry.tsx` passed lazy `() => import(...)` thunks to
// `defineRemote` (which resolve to Promises → React threw "Objects are not valid
// as a React child (found: [object Promise])" and the page rendered only skeleton
// placeholders). The SDK contract is render functions `(ctx, bridge) => ReactNode`
// + a CSS-string `styles` — this asserts the mount actually produces DOM.
//
// Prereq: `make dev` (node :8391 + shell :5391) + `make seed`.

const ADMIN_HANDLE = process.env.CC_ADMIN_HANDLE || "ada";
const ADMIN_PASSWORD = process.env.CC_ADMIN_PASSWORD || "cc-admin-1234";
const WORKSPACE = process.env.CC_WORKSPACE || "acme";

test("the care extension mounts and renders content at /ext/<ws>", async ({ page }) => {
  const pageErrors: string[] = [];
  page.on("pageerror", (e) => pageErrors.push(e.message));

  // sign in as the seeded admin and enter the workspace
  await page.goto("/login");
  await page.locator("#email").fill(ADMIN_HANDLE);
  await page.locator("#password").fill(ADMIN_PASSWORD);
  await page.getByRole("button", { name: /sign in/i }).click();
  await expect(page).toHaveURL(/\/workspaces$/);
  await page.getByRole("button", { name: new RegExp(WORKSPACE, "i") }).click();
  await expect(page).toHaveURL(new RegExp(`/ext/${WORKSPACE}`));

  // The SDK mounts the ext under a scoped root; it must exist AND carry content.
  const extRoot = page.locator("[data-ext-root='care']");
  await expect(extRoot).toBeVisible();
  // The care Home page renders its bottom-tab nav — real DOM, not skeletons.
  await expect(page.getByRole("button", { name: /today|hoy/i })).toBeVisible();
  await expect(page.getByRole("button", { name: /children|niños/i })).toBeVisible();

  // The loading skeletons are REMOVED once the remote mounts (the "perpetual
  // skeleton" bug: they were placed inside the SDK's append-only mount target,
  // so they never went away). After mount there must be zero pulsing placeholders.
  await expect(page.locator(".animate-pulse")).toHaveCount(0, { timeout: 5000 });

  // No React render error escaped (the [object Promise] crash would land here).
  expect(pageErrors, `page errors: ${pageErrors.join(" | ")}`).toHaveLength(0);
});

test("the shell header switches workspace and signs out", async ({ page }) => {
  await page.goto("/login");
  await page.locator("#email").fill(ADMIN_HANDLE);
  await page.locator("#password").fill(ADMIN_PASSWORD);
  await page.getByRole("button", { name: /sign in/i }).click();
  await expect(page).toHaveURL(/\/workspaces$/);
  await page.getByRole("button", { name: new RegExp(WORKSPACE, "i") }).click();
  await expect(page).toHaveURL(new RegExp(`/ext/${WORKSPACE}`));

  // The host chrome — the ONE way out of the mounted ext — is present.
  const wsBtn = page.getByRole("button", { name: /workspaces|espacios/i });
  const outBtn = page.getByRole("button", { name: /sign out|cerrar sesión|salir/i });
  await expect(wsBtn).toBeVisible();
  await expect(outBtn).toBeVisible();

  // Home/workspaces → back to the picker (switch workspace).
  await wsBtn.click();
  await expect(page).toHaveURL(/\/workspaces$/);

  // Re-enter, then sign out → /login, and the server session is really cleared.
  await page.getByRole("button", { name: new RegExp(WORKSPACE, "i") }).click();
  await expect(page).toHaveURL(new RegExp(`/ext/${WORKSPACE}`));
  await page.getByRole("button", { name: /sign out|cerrar sesión|salir/i }).click();
  await expect(page).toHaveURL(/\/login$/);
  const afterLogout = await page.request.post("/api/mcp/call", {
    data: { tool: "care.center.list", args: {} },
  });
  expect(afterLogout.status()).toBe(401);
});
