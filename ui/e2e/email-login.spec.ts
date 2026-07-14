import { test, expect } from "@playwright/test";

// The REAL email + password login golden path, in a real browser against the real
// node running in PasswordHash mode (no mocks — mobile-shell-scope §"Rule 9").
// A person signs in with an email + a password THEY set at invite-accept, not the
// dev bootstrap handle. Proves the whole chain end to end:
//   admin provisions a guardian + mints an invite (care.* over the bridge)
//   → the raw token is recovered from the outbox effect (dev seam)
//   → /invite/<token> accept page: preview (email/locale) + SET a password
//   → /public/invite/accept creates the account + argon2 credential + membership
//   → sign out → /login with email + password → lands on /workspaces
//   → a WRONG password is rejected (PasswordHash actually bites).
//
// Prereq: `make dev` (node :8391 in PasswordHash mode + shell :5391) + `make seed`.
// The guardian is provisioned FRESH per run (a unique id/email) so the invite is
// always redeemable and the test is re-runnable without a store reset.

const ADMIN_HANDLE = process.env.CC_ADMIN_HANDLE || "ada";
const ADMIN_PASSWORD = process.env.CC_ADMIN_PASSWORD || "cc-admin-1234";
const WORKSPACE = process.env.CC_WORKSPACE || "acme";

/** Sign in as the seeded workspace-admin (needed to provision + invite over the bridge). */
async function loginAdmin(page: import("@playwright/test").Page) {
  await page.goto("/login");
  await page.locator("#email").fill(ADMIN_HANDLE);
  await page.locator("#password").fill(ADMIN_PASSWORD);
  await page.getByRole("button", { name: /sign in/i }).click();
  await expect(page).toHaveURL(/\/workspaces$/);
}

test("a guardian accepts an invite, sets a password, and logs in by email + password", async ({
  page,
}) => {
  // A run-unique guardian so the invite is always fresh + redeemable (re-runnable).
  const uniq = `${Date.now()}`;
  const guardianId = `e2e-${uniq}`;
  const email = `e2e-${uniq}@familia.test`;
  const password = "guardian-secret-123";

  // ── admin provisions the guardian record, then mints the invite ─────────────
  await loginAdmin(page);
  const create = await page.request.post("/api/mcp/call", {
    data: {
      tool: "care.guardian.create",
      args: { id: guardianId, name: "E2E Guardian", email, locale: "en" },
    },
  });
  expect(create.status()).toBe(200);
  const mint = await page.request.post("/api/mcp/call", {
    data: { tool: "care.invite.create_guardian", args: { guardian_id: guardianId, locale: "en" } },
  });
  expect(mint.status()).toBe(200);

  // Recover the raw invite token from the outbox effect (dev-only seam).
  const tok = await page.request.get(`/api/dev/invite-token?email=${encodeURIComponent(email)}`);
  expect(tok.status()).toBe(200);
  const { token } = await tok.json();
  expect(token).toMatch(/^lbi_/);

  // Sign the admin out so the accept happens pre-auth (a fresh session).
  await page.request.post("/api/auth/logout");

  // ── the accept page: preview shows the invitee's email; set a password ──────
  await page.goto(`/invite/${token}`);
  await expect(page.locator("#invite-email")).toHaveValue(email);
  await page.locator("#invite-password").fill(password);
  await page.getByRole("button", { name: /accept invite|aceptar/i }).click();

  // Accept mints a session → the app lands on the workspace picker.
  await expect(page).toHaveURL(/\/workspaces$/);

  // Sign out to prove the CREDENTIAL (not the accept session) authenticates next.
  await page.request.post("/api/auth/logout");

  // ── the real thing: email + the password the guardian set → signed in ───────
  await page.goto("/login");
  await page.locator("#email").fill(email);
  await page.locator("#password").fill(password);
  await page.getByRole("button", { name: /sign in/i }).click();
  await expect(page).toHaveURL(/\/workspaces$/);
  await expect(page.getByRole("button", { name: new RegExp(WORKSPACE, "i") })).toBeVisible();

  // ── and a WRONG password for that same account is rejected ──────────────────
  await page.request.post("/api/auth/logout");
  await page.goto("/login");
  await page.locator("#email").fill(email);
  await page.locator("#password").fill("definitely-not-the-password");
  await page.getByRole("button", { name: /sign in/i }).click();
  // If this had 200'd, the node would be running password-less — the exact
  // regression this whole change fixes. It must 401 → the shell stays on /login.
  await expect(page.getByRole("alert")).toBeVisible();
  await expect(page).toHaveURL(/\/login$/);
});
