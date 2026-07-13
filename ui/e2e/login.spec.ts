import { test, expect } from "@playwright/test";

// The login golden path, in a REAL browser against the REAL node (no mocks —
// mobile-shell-scope §"Rule 9"). Proves the whole shell auth seam end to end:
//   login form → vite-dev-auth `/api/auth/login` → cc-node `/login` → token
//   cookie → `/workspaces` (folded from the session) → mount the care ext.
//
// Prereq: the node (:8080) + the shell dev server (:5173) are running and the
// roster is seeded (`make dev` + `make seed`). The dev admin handle is `ada`
// (→ `user:ada`, the seeded workspace-admin). The node runs in the default
// PasswordHash mode, so the admin logs in with the seeded password (the node
// seeds `ada`'s credential at boot from CC_SEED_PASSWORD == ADMIN_PASSWORD).

const ADMIN_HANDLE = process.env.CC_ADMIN_HANDLE || "ada";
const ADMIN_PASSWORD = process.env.CC_ADMIN_PASSWORD || "cc-admin-1234";
const WORKSPACE = process.env.CC_WORKSPACE || "acme";

test("admin logs in and reaches the care app", async ({ page }) => {
  // ── the login screen ──────────────────────────────────────────────────────
  await page.goto("/login");
  await expect(page.getByRole("button", { name: /sign in/i })).toBeVisible();

  // Fill the handle (the field accepts a bare handle, not just an email) and the
  // password (ignored by the dev-login, but the form requires a value).
  await page.locator("#email").fill(ADMIN_HANDLE);
  await page.locator("#password").fill(ADMIN_PASSWORD);
  await page.getByRole("button", { name: /sign in/i }).click();

  // ── the workspace picker (session minted, folded to the caller's ws) ───────
  await expect(page).toHaveURL(/\/workspaces$/);
  await expect(page.getByRole("heading", { name: /choose a workspace/i })).toBeVisible();
  const wsButton = page.getByRole("button", { name: new RegExp(WORKSPACE, "i") });
  await expect(wsButton).toBeVisible();
  // The role folded from the token's caps (admin markers) shows on the card.
  await expect(page.getByText("admin", { exact: false })).toBeVisible();

  // ── enter the workspace → the care ext page route ─────────────────────────
  await wsButton.click();
  await expect(page).toHaveURL(new RegExp(`/ext/${WORKSPACE}`));
});

// The mediated bridge works with the cookie session: an authenticated
// `/api/mcp/call` reaches the live care sidecar and returns the seeded roster.
// (Asserted at the API layer the shell's bridge uses — the care ext's federated
// render is proven separately; this pins that LOGIN yields a usable session.)
test("the logged-in session can read the seeded roster over the bridge", async ({ page, request }) => {
  await page.goto("/login");
  await page.locator("#email").fill(ADMIN_HANDLE);
  await page.locator("#password").fill(ADMIN_PASSWORD);
  await page.getByRole("button", { name: /sign in/i }).click();
  await expect(page).toHaveURL(/\/workspaces$/);

  // The browser context now holds the session cookie; drive the same mediated
  // bridge the ext uses. A seeded admin sees the seeded center (rule-7 admin pass).
  const resp = await page.request.post("/api/mcp/call", {
    data: { tool: "care.center.list", args: {} },
  });
  expect(resp.status()).toBe(200);
  const centers = await resp.json();
  expect(Array.isArray(centers)).toBeTruthy();
  expect(JSON.stringify(centers)).toContain("Sunshine Childcare");
});

test("a bad handle is rejected at the login screen", async ({ page }) => {
  await page.goto("/login");
  await page.locator("#email").fill("nobody-here");
  await page.locator("#password").fill("x");
  await page.getByRole("button", { name: /sign in/i }).click();

  // The gateway denies a non-member; the shell shows the denied error and stays
  // on /login (never mints a session).
  await expect(page.getByRole("alert")).toBeVisible();
  await expect(page).toHaveURL(/\/login$/);
});
