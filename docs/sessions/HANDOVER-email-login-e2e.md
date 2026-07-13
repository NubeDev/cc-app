# Handover — get EMAIL/PASSWORD login working end to end (browser)

> **RESOLVED 2026-07-13 — email + password login is LIVE and Playwright-proven.** The prompt
> below was executed. Outcome:
> - **The invite→accept→set-password→email-login chain needed NO lb change** for
>   membership/credential — lb `invite_accept` already creates the identity, sets the argon2
>   credential, joins the workspace, mints a session (route: `POST /public/invite/accept`, NOT
>   `/invites/accept`). Verified live.
> - **The ONE real lb gap:** `boot_full` hardwired the password-less `DevTrustAny` check
>   (`Gateway::new_live`), so an embedded node accepted ANY password — the `PasswordHash` check
>   was unreachable below the embed seam. **Fixed in lb `node-v0.4.2`** (local tag): additive
>   `BootConfig::credential_mode` (applied via `Gateway::with_credential_check`) +
>   `BootConfig::seed_credential` (boot seed argon2-sets the dev admin's password — bootstrap
>   paradox fix). Scope: `lb/docs/scope/auth-caps/embedder-credential-mode-scope.md`; test
>   `lb/rust/node/tests/credential_mode_test.rs` (4 green).
> - **cc-app wired it in:** `CC_PASSWORD_LOGIN` + `CC_SEED_PASSWORD` (`rust/node/src/boot.rs`);
>   `make dev` now defaults to real PasswordHash (`PASSWORD_LOGIN=1`,
>   `ADMIN_PASSWORD=cc-admin-1234`); the shell accept page sets a password + the dev-auth seam
>   was fixed to `/public/invite/accept` (+ a verify + dev token-recovery seam); `seed.sh`
>   prints working email-login instructions + a ready accept link.
> - **Proof:** `ui/e2e/email-login.spec.ts` (green) — guardian accepts → sets password →
>   email+password login lands on `/workspaces`; WRONG password → 401 → stays on `/login`.
>   `make e2e-ui` 4/4; `cargo test --workspace` green (rule 7 `live_node.rs` intact).
> - **Still local:** lb + cc-app build via the `.cargo/config.toml` `[patch]` → sibling
>   checkouts; `node-v0.4.2` is a LOCAL tag, not pushed (per the operator's "keep it local"
>   instruction this round). Release ritual (push tags, drop patch) pending.
> - **ALSO FIXED this session — the care ext blank render at `/ext/acme`.** It was NOT a
>   session-provider gap (as guessed below): `remoteEntry.tsx` passed lazy `() => import(...)`
>   thunks to `defineRemote`, which the SDK renders directly → "Objects are not valid as a React
>   child (found: [object Promise])". Rewrote it to the SDK contract (static imports; `?inline` CSS
>   string; page wrapped in the ext's `LocaleProvider`; widgets as a keyed map) + fixed
>   `vite.config.ts` (`extUiSdk` → `defineExtConfig`, unblocking `pnpm build`/`make pack`).
>   Guard: `ui/e2e/ext-mount.spec.ts` (green). Debug doc:
>   `docs/debugging/ui/care-ext-blank-render-remoteentry-promise-child.md`.

---

Copy the block below into a new session. Everything above the line is context for you (the human);
the fenced block is the prompt.

---

```
GOAL: make real EMAIL + PASSWORD login work end to end in the browser shell (ui/), proven by a
Playwright test — a person signs in with an email + a password they set, not the dev bootstrap
handle. Today only the dev bootstrap handle `ada` logs in (password-less, LB_DEV_LOGIN). Read this
whole prompt before touching anything, then VERIFY each claim against the live system before acting —
prior sessions have been wrong about lb internals, so drive the real flow, don't trust summaries.

REPO FAMILY (read CLAUDE.md + docs/WORKFLOW-LB.md first): cc-app embeds lb (NubeDev/lb) via lb-node.
Sibling checkouts: ~/code/rust/lb, ~/code/rust/lb-ext-sdk. A change in the login/membership/gateway
belongs in lb (then tag + bump the pin), NOT a cc-app workaround (rule 10). A UI/shell/seed change is
cc-app. Decide the owning repo BEFORE coding.

CURRENT STATE (verified 2026-07-13 — RE-VERIFY, don't assume):
  - Browser login for the dev handle NOW WORKS. A prior session added a Vite dev auth seam
    (ui/vite-dev-auth.ts, wired in ui/vite.config.ts) that terminates the shell's /api/* calls and
    forwards to the cc-node gateway, holding the lb token in a SERVER-SIDE session map (the JWT is
    ~9KB — over the 4KB cookie limit — so the cookie carries only a short session id). Login form →
    /api/auth/login → gateway POST /login {user,workspace,secret} → session cookie → /workspaces →
    /api/mcp/call (bearer from the cookie's session). Playwright proof: ui/e2e/login.spec.ts (3
    green), run via `make e2e-ui` (needs `make dev` + `make seed` running).
  - The login field accepts a bare HANDLE (ui/src/auth/LoginPage.tsx, type="text"): typing `ada`
    logs in as user:ada, the seeded workspace-admin. dev-login is password-less.

WHY EMAIL LOGIN DOESN'T WORK YET (the two gaps — CONFIRM both against the running node):
  1. MEMBERSHIP. The gateway /login runs membership_login_resolve: a ws that already has members
     (acme has user:ada) refuses a DIFFERENT sub ("not a member"). `admin@acme.test` canonicalizes to
     `user:admin@acme.test` — a distinct principal that was never made a member. The seed sets a
     CREDENTIAL for it (identity.set_credential, which IS routed on /mcp/call) but NOT membership.
     Verified: `membership.add`/`members.add` are NOT routed on /mcp/call at node-v0.4.0
     (call_membership_tool is defined in lb but never invoked in tool_call.rs) — so cc-app cannot
     provision a member over the callback. Confirm by grepping lb `rust/crates/host/src/tool_call.rs`
     at the pinned tag and by trying the verb live.
  2. CREDENTIAL MODE. Login credential-check is env-selected (lb role/gateway session/credential.rs):
     LB_DEV_LOGIN=1 → DevTrustAny (password-less, what the dev node runs); unset → PasswordHash
     (argon2 against the per-(ws,user) credential identity.set_credential writes). So even once a
     member exists, REAL password login only bites when the node runs WITHOUT LB_DEV_LOGIN. Confirm
     how cc-node sets it (rust/node/src/boot.rs / the Makefile dev target env).

THE LIKELY NO-LB-CHANGE PATH (evaluate FIRST — it may already work):
  The INVITE→ACCEPT flow already exists end to end and PROVISIONS MEMBERSHIP:
    - cc-app: care.invite.create_guardian / create_staff mint a real lb invite over the callback
      (rust/extensions/care/src/invite/*.rs → SidecarClient invite.create). The shell has the
      /invite/:token route + InviteAcceptPage + the /api/invites/accept seam (already proxied by
      vite-dev-auth.ts).
    - lb: gateway /invites/accept verifies the token and calls membership_add_raw → provisions
      membership + returns a session (rust/crates/host/src/invites/accept.rs).
  So the real-email path is probably: admin mints an invite for a person → they open /invite/<token>
  → accept (membership + first session) → they SET a password → thereafter email+password /login works
  (under PasswordHash mode). DETERMINE whether this fully works today, and exactly where it breaks
  (does accept set a credential? does the invite carry the email as the sub? does /login by email
  resolve to the accepted member?). Prefer this path — it needs no lb change if it already closes.

DO, in order:
  A. Bring the system up + reproduce: `make dev` (node:8080 + shell:5173), `make seed`, `make e2e-ui`
     (dev-handle login green). Then try email login by hand through :5173 and capture the EXACT
     failure (curl the gateway /login and the /api/* seam; a prior session's notes on shapes are in
     ui/vite-dev-auth.ts).
  B. Trace the invite→accept→set-password→email-login chain end to end against the live node. Find the
     first broken link. Decide: does this close with cc-app + shell work only, or is an lb gap real
     (e.g. no routed member-provisioning verb; accept doesn't set a credential; PasswordHash login
     can't find the credential)?
  C. If an lb change IS needed, STOP and write a scope FIRST per ~/code/rust/lb/docs/SCOPE-WRITTING.md
     under lb docs/scope/ (auth-caps/ or membership/ area — mirror the existing
     native-caller-identity-scope.md shape: goals / non-goals / intent / testing / rule-10 check).
     Then implement additively, tag (node-v0.4.x / sdk-v0.4.x as needed), and bump the cc-app pins
     (rust/Cargo.toml + Makefile LB_TAG) + drop any local [patch]. NOTE: cc-app currently builds
     lb/sdk via a LOCAL [patch] in the git-ignored .cargo/config.toml (pins say node-v0.4.0/sdk-v0.4.0
     but redirect to the sibling checkouts which carry LOCALLY-tagged node-v0.4.1/sdk-v0.4.1 — the
     native-caller `admin` marker, not yet pushed). Respect/extend that pattern; don't fight it.
  D. Prove it with Playwright: extend ui/e2e/login.spec.ts (or a new spec) with a REAL email+password
     login that lands on /workspaces and reads the seeded roster over the bridge. `make e2e-ui` green.
     Update the seed message (scripts/seed.sh) so it prints working email-login instructions (it
     currently tells people to use the `ada` handle because email login didn't work).

CONSTRAINTS:
  - No mocks (rule 4): tests drive the real node + real gateway + real spawned care sidecar.
  - Rule 7 is sacred (guardian isolation) — don't regress it. rule 10 — fix lb generically, no cc-app
    special-casing of the platform.
  - One responsibility per file, <=400 lines (docs/FILE-LAYOUT.md). Keep the authz-fence green.
  - The vite-dev-auth.ts seam is DEV-ONLY; a production host terminates the session for real. Don't
    over-invest it into a prod BFF unless the scope says so.

KNOWN SEPARATE ISSUE (out of scope unless it blocks you): the care EXTENSION UI renders BLANK at
/ext/acme — the federated remote loads all its modules but produces no visible DOM (likely the
@nube/ext-ui-sdk mount needs a session provider the shell's bridge doesn't pass). Login itself works;
this is the ext's own render. Flag it, don't rabbit-hole into it.

REPORT HONESTLY: if the shipped API differs from these notes, or a "quick fix" hides a platform gap,
STOP and say so. Verify at each step by driving the real browser + node, not the type-checker.
```

---

## Notes for the human (not part of the prompt)

- Nothing has been committed. cc-app has the vite-dev-auth seam, the Playwright spec, the login-field
  relax, the seed-message fix, the `make e2e-ui` target, and `.gitignore` entries. lb + lb-ext-sdk
  have local commits + local tags (`node-v0.4.1` / `sdk-v0.4.1`) on branch `native-caller-admin-marker`
  — NOT pushed. cc-app builds them via a local `[patch]` in the git-ignored `.cargo/config.toml`.
- The dev node + shell may be down (I started them in the background; they exit when this session
  ends). The next session's step A (`make dev`) brings them back.
- My read: the invite→accept path is the most promising no-lb-change route. If it turns out accept
  doesn't set a password credential (so PasswordHash login still can't verify), THAT is the real lb
  gap to scope — a "set your password after accept" verb, or accept taking an initial credential.
```
