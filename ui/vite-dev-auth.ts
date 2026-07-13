import type { Plugin, Connect } from "vite";
import type { IncomingMessage, ServerResponse } from "node:http";

// ─────────────────────────────────────────────────────────────────────────────
// Dev-only auth seam (the thin shell's BFF half).
//
// The browser shell only ever talks to its OWN origin over `/api/*` (see
// `src/api/gateway.ts`): it never holds a bearer token or knows the gateway URL.
// In production a real host process terminates the session and forwards to the
// lb gateway; in dev THIS middleware is that host. It:
//
//   - `/api/auth/login`  {handle, password} → lb `POST /login`
//        {user, workspace, secret}; on 200 it stores the minted token in an
//        httpOnly cookie and returns the UI `Session` shape.
//   - `/api/auth/logout` clears the cookie.
//   - `/api/me/workspaces` returns the caller's workspaces (dev: the one ws the
//        session was minted into, with the role folded from the token).
//   - `/api/mcp/call` + `/api/invites/accept` proxy to the gateway, attaching
//        the cookie's bearer token (`/invites/accept` is pre-auth, token optional).
//
// The gateway URL comes from env (VITE_GATEWAY_URL / CC_GATEWAY_URL), matching
// what `make dev` exports; it defaults to the dev gateway. This file is dev-only
// (wired via `configureServer`) — it never ships in the production bundle.
// ─────────────────────────────────────────────────────────────────────────────

const GATEWAY_URL =
  process.env.VITE_GATEWAY_URL || process.env.CC_GATEWAY_URL || "http://127.0.0.1:8080";

// The one dev workspace the shell logs into. The gateway's membership resolve is
// the real gate; this is only the default `workspace` we send with a login handle.
const DEV_WORKSPACE = process.env.CC_WORKSPACE || "acme";

const COOKIE = "cc_session";

// The lb token is a JWT carrying the FULL resolved cap set (~9KB here) — far over
// the ~4KB browser cookie limit, so the browser silently drops it if we cookie it
// directly (the bug this indirection fixes). Instead we mint a short opaque
// session id, keep the real token in this dev-process map, and cookie only the id.
// Dev-only + in-memory: it resets when the dev server restarts (re-login), which
// is exactly right for a dev seam.
const SESSIONS = new Map<string, string>(); // sid → gateway JWT
let sidCounter = 0;
function newSid(): string {
  // Not cryptographic — this is a dev-only server-side map key, never the token.
  sidCounter += 1;
  return `s${sidCounter}_${Date.now().toString(36)}`;
}

/** Read one cookie value from a request's Cookie header. */
function readCookie(req: IncomingMessage, name: string): string | null {
  const raw = req.headers.cookie;
  if (!raw) return null;
  for (const part of raw.split(";")) {
    const [k, ...v] = part.trim().split("=");
    if (k === name) return decodeURIComponent(v.join("="));
  }
  return null;
}

/** Resolve the gateway token for a request from its session-id cookie. */
function tokenFor(req: IncomingMessage): string | null {
  const sid = readCookie(req, COOKIE);
  if (!sid) return null;
  return SESSIONS.get(sid) ?? null;
}

/** Collect a JSON request body (dev traffic is tiny; no streaming needed). */
async function readJson(req: IncomingMessage): Promise<any> {
  const chunks: Buffer[] = [];
  for await (const c of req) chunks.push(c as Buffer);
  if (!chunks.length) return {};
  try {
    return JSON.parse(Buffer.concat(chunks).toString("utf8"));
  } catch {
    return {};
  }
}

function sendJson(res: ServerResponse, status: number, body: unknown) {
  const s = JSON.stringify(body);
  res.statusCode = status;
  res.setHeader("content-type", "application/json");
  res.end(s);
}

/** Decode a JWT payload without verifying (the gateway is authoritative; we only
 *  read `role`/`caps`/`ws` for the UI's convenience gating). */
function decodeJwt(token: string): Record<string, any> {
  try {
    const payload = token.split(".")[1];
    const json = Buffer.from(payload, "base64url").toString("utf8");
    return JSON.parse(json);
  } catch {
    return {};
  }
}

/** lb mints every session `role: member`; real authority rides caps. Fold the
 *  cap set into the coarse UI role the shell shows (admin controls vs guardian).
 *  Mirrors the care sidecar's caps-based admin signal (native-caller-identity). */
function roleFromCaps(caps: string[]): string {
  const ADMIN_MARKERS = ["mcp:members.manage:call", "mcp:members.add:call", "mcp:native.install:call"];
  return caps.some((c) => ADMIN_MARKERS.includes(c)) ? "admin" : "member";
}

/** POST helper to the gateway. */
async function gatewayPost(path: string, body: unknown, token?: string) {
  const headers: Record<string, string> = { "content-type": "application/json" };
  if (token) headers.authorization = `Bearer ${token}`;
  const r = await fetch(`${GATEWAY_URL}${path}`, {
    method: "POST",
    headers,
    body: JSON.stringify(body ?? {}),
  });
  const text = await r.text();
  let json: any;
  try {
    json = text ? JSON.parse(text) : {};
  } catch {
    json = { error: text };
  }
  return { status: r.status, json, text };
}

export function devAuth(): Plugin {
  return {
    name: "cc-dev-auth",
    configureServer(server) {
      const use = (path: string, handler: Connect.NextHandleFunction) =>
        server.middlewares.use(path, handler);

      // ── login: {handle|email, password} → gateway /login ──────────────────
      use("/api/auth/login", (req, res, next) => {
        if (req.method !== "POST") return next();
        void (async () => {
          const body = await readJson(req);
          // The form labels it "email", but lb logs in by HANDLE — `ada` → the
          // `user:ada` principal (the seeded dev admin). Accept either; the
          // gateway canonicalizes a bare handle to `user:<handle>`.
          const user = String(body.email ?? body.handle ?? body.user ?? "").trim();
          const secret = String(body.password ?? body.secret ?? "");
          if (!user) return sendJson(res, 400, { error: "handle required" });

          const { status, json } = await gatewayPost("/login", {
            user,
            workspace: DEV_WORKSPACE,
            secret,
          });
          if (status !== 200 || !json.token) {
            return sendJson(res, status === 200 ? 502 : status, {
              error: json?.error ?? "login failed",
            });
          }

          // Keep the (large) token server-side; cookie only a short session id.
          const sid = newSid();
          SESSIONS.set(sid, json.token);
          res.setHeader(
            "set-cookie",
            `${COOKIE}=${encodeURIComponent(sid)}; Path=/; HttpOnly; SameSite=Lax`,
          );
          const claims = decodeJwt(json.token);
          sendJson(res, 200, {
            sub: json.principal,
            workspaceId: json.workspace,
            role: roleFromCaps(json.caps ?? []),
            locale: claims.locale === "es" ? "es" : "en",
          });
        })();
      });

      // ── logout ────────────────────────────────────────────────────────────
      use("/api/auth/logout", (req, res, next) => {
        if (req.method !== "POST") return next();
        const sid = readCookie(req, COOKIE);
        if (sid) SESSIONS.delete(sid);
        res.setHeader("set-cookie", `${COOKIE}=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0`);
        sendJson(res, 200, {});
      });

      // ── the caller's workspaces (dev: the one the session was minted into) ──
      use("/api/me/workspaces", (req, res, next) => {
        if (req.method !== "GET") return next();
        const token = tokenFor(req);
        if (!token) return sendJson(res, 401, { error: "no session" });
        const claims = decodeJwt(token);
        sendJson(res, 200, [
          {
            id: claims.ws ?? DEV_WORKSPACE,
            name: claims.ws ?? DEV_WORKSPACE,
            role: roleFromCaps(claims.caps ?? []),
          },
        ]);
      });

      // ── mediated MCP bridge: attach the cookie's bearer, forward verbatim ──
      use("/api/mcp/call", (req, res, next) => {
        if (req.method !== "POST") return next();
        void (async () => {
          const token = tokenFor(req);
          if (!token) return sendJson(res, 401, { error: "no session" });
          const body = await readJson(req);
          const { status, json } = await gatewayPost("/mcp/call", body, token);
          sendJson(res, status, json);
        })();
      });

      // ── DEV-ONLY: recover the raw invite token for an email from the outbox ──
      // The invite verb never returns the raw token (it ships in the email effect
      // only). For a self-contained e2e we recover it from the durable outbox
      // `send_invite` effect (readable via the gateway's `/store` route with the
      // session bearer). Dev/test only — a production host reads the emailed link.
      // `GET /api/dev/invite-token?email=<email>` → { token } (or 404).
      use("/api/dev/invite-token", (req, res, next) => {
        if (req.method !== "GET") return next();
        void (async () => {
          const token = tokenFor(req);
          if (!token) return sendJson(res, 401, { error: "no session" });
          const email = new URL(req.url ?? "", "http://localhost").searchParams.get("email") ?? "";
          const r = await fetch(`${GATEWAY_URL}/store/tables/outbox/rows`, {
            headers: { authorization: `Bearer ${token}` },
          });
          if (!r.ok) return sendJson(res, r.status, { error: "outbox read failed" });
          const body = await r.json().catch(() => ({}) as any);
          const rows: any[] = body?.rows ?? [];
          let found: string | null = null;
          for (const row of rows) {
            const eff = row?.data?.data;
            if (eff?.action !== "send_invite") continue;
            let payload: any;
            try {
              payload = JSON.parse(eff.payload);
            } catch {
              continue;
            }
            if (payload?.email === email && payload?.token) found = payload.token; // last wins
          }
          if (!found) return sendJson(res, 404, { error: "no invite for email" });
          sendJson(res, 200, { token: found });
        })();
      });

      // ── pre-auth invite VERIFY (token preview: email/locale/redeemable) ────
      // The accept page fetches this BEFORE any session so it can render in the
      // invitee's language. lb: `GET /public/invite/verify?workspace=&token=`.
      use("/api/invites/verify", (req, res, next) => {
        if (req.method !== "GET") return next();
        void (async () => {
          const url = new URL(req.url ?? "", "http://localhost");
          const token = url.searchParams.get("token") ?? "";
          const workspace = url.searchParams.get("workspace") || DEV_WORKSPACE;
          const r = await fetch(
            `${GATEWAY_URL}/public/invite/verify?workspace=${encodeURIComponent(
              workspace,
            )}&token=${encodeURIComponent(token)}`,
          );
          const text = await r.text();
          let json: any;
          try {
            json = text ? JSON.parse(text) : {};
          } catch {
            json = { error: text };
          }
          sendJson(res, r.status, json);
        })();
      });

      // ── pre-auth invite ACCEPT (token + new password in the body) ──────────
      // lb: `POST /public/invite/accept` {token, workspace, secret, current_secret?}
      // → creates the identity, sets the argon2 credential, joins the workspace,
      // and mints a session. On 200 we hold the token server-side (same as login)
      // so the browser lands signed-in, ready for email+password login next time.
      use("/api/invites/accept", (req, res, next) => {
        if (req.method !== "POST") return next();
        void (async () => {
          const body = await readJson(req);
          // The shell posts {token, secret}; the gateway also needs the workspace.
          const accept = {
            token: body.token,
            workspace: body.workspace || DEV_WORKSPACE,
            secret: body.secret ?? body.password,
            current_secret: body.current_secret,
          };
          const { status, json } = await gatewayPost("/public/invite/accept", accept);
          if (status === 200 && json.token) {
            const sid = newSid();
            SESSIONS.set(sid, json.token);
            res.setHeader(
              "set-cookie",
              `${COOKIE}=${encodeURIComponent(sid)}; Path=/; HttpOnly; SameSite=Lax`,
            );
            const claims = decodeJwt(json.token);
            // Return the UI Session shape (same as login) so the accept page can
            // route straight into the app, already authenticated.
            return sendJson(res, 200, {
              sub: json.principal,
              workspaceId: json.workspace,
              role: roleFromCaps(json.caps ?? []),
              locale: claims.locale === "es" ? "es" : "en",
            });
          }
          sendJson(res, status, json);
        })();
      });
    },
  };
}
