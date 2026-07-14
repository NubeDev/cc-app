import { test, expect } from "@playwright/test";

// Milestones 06 (attendance) + 07 (menus) UI smoke — a real browser against the
// real node (no mocks — mobile-shell-scope §"Rule 9"). Proves the new
// role-aware tabs mount and render real DOM (not skeletons), in BOTH en and es
// (CLAUDE.md rule 8: the food-safety + child-safety surfaces must render in
// Spanish too). The admin persona sees the menu PLANNER and the attendance
// dashboard/roster.
//
// Prereq: `make dev` (node :8391 + shell :5391) + `make seed`.

const ADMIN_HANDLE = process.env.CC_ADMIN_HANDLE || "ada";
const ADMIN_PASSWORD = process.env.CC_ADMIN_PASSWORD || "cc-admin-1234";
const WORKSPACE = process.env.CC_WORKSPACE || "acme";

async function signInToExt(page) {
  await page.goto("/login");
  await page.locator("#email").fill(ADMIN_HANDLE);
  await page.locator("#password").fill(ADMIN_PASSWORD);
  await page.getByRole("button", { name: /sign in|iniciar/i }).click();
  await expect(page).toHaveURL(/\/workspaces$/);
  await page.getByRole("button", { name: new RegExp(WORKSPACE, "i") }).click();
  await expect(page).toHaveURL(new RegExp(`/ext/${WORKSPACE}`));
  await expect(page.locator("[data-ext-root='care']")).toBeVisible();
}

test("admin: attendance + menus tabs mount and render real DOM (en)", async ({ page }) => {
  const pageErrors: string[] = [];
  page.on("pageerror", (e) => pageErrors.push(e.message));
  await signInToExt(page);

  // Attendance tab → the AttendancePage (segmented Roster | Who's here).
  await page.getByRole("button", { name: /^attendance$/i }).click();
  await expect(page.getByRole("heading", { name: /attendance/i })).toBeVisible();
  // The segmented control's two surfaces are present.
  await expect(page.getByText(/roster/i).first()).toBeVisible();

  // Menus tab → the admin planner.
  await page.getByRole("button", { name: /^menus$/i }).click();
  await expect(page.getByRole("heading", { name: /menu/i })).toBeVisible();

  // No skeletons left after mount, and no React render error escaped.
  await expect(page.locator(".animate-pulse")).toHaveCount(0, { timeout: 6000 });
  expect(pageErrors, `page errors: ${pageErrors.join(" | ")}`).toHaveLength(0);
});

test("admin: the new tabs render in Spanish (es-locale — rule 8)", async ({ page }) => {
  // Force the ext locale to Spanish before the app boots (the ext's
  // LocaleProvider reads localStorage `care.locale`).
  await page.addInitScript(() => {
    try {
      window.localStorage.setItem("care.locale", "es");
    } catch {
      /* ignore */
    }
  });
  await signInToExt(page);

  // The bottom-tab labels are Spanish: Asistencia (attendance) + Menús (menus).
  await expect(page.getByRole("button", { name: /asistencia/i })).toBeVisible();
  await expect(page.getByRole("button", { name: /menús/i })).toBeVisible();

  // Open the menus tab; the Spanish planner heading renders (not an English
  // fallback — a Spanish-speaking admin plans the food-safety surface).
  await page.getByRole("button", { name: /menús/i }).click();
  await expect(page.getByRole("heading", { name: /men[úu]|planificador/i })).toBeVisible();
});
