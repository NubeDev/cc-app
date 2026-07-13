import { useEffect, useState } from "react";
import { useNavigate, useParams } from "react-router-dom";
import { authApi } from "../api/auth";
import { useT, useLocaleSwitch } from "../hooks/useT";
import { ThemeControls } from "../components/ThemeControls";
import { Button } from "../components/ui/button";
import { Input } from "../components/ui/input";
import { Label } from "../components/ui/label";

/// The pre-auth invite accept page (milestone 05 golden-path entry; real email+password login).
///
/// The admin mints an invite via `care.invite.create_guardian`; the lb side emails a link
/// (`<base>/invite/<raw-token>`); the invitee lands here. On mount the page previews the invite
/// (`GET /api/invites/verify` → `/public/invite/verify`) to read the email + locale, so a
/// Spanish-speaking Ana gets a Spanish accept page (CLAUDE.md rule 8). The invitee then SETS a
/// password and accepts (`POST /api/invites/accept` → `/public/invite/accept`), which creates the
/// account, writes the argon2 credential, joins the workspace, derives scoped reach grants from the
/// existing edges, and mints a session. Thereafter she signs in with email + THIS password (the
/// gateway runs PasswordHash — a wrong password 401s). Her first sign-in lands seeing ONLY the child
/// she holds a live edge to — the cross-family deny test proves rule 7.
const MIN_PASSWORD = 8;

export function InviteAcceptPage() {
  const t = useT();
  const nav = useNavigate();
  const { token = "" } = useParams();
  const { setLocale } = useLocaleSwitch();
  const [email, setEmail] = useState<string | null>(null);
  const [password, setPassword] = useState("");
  const [busy, setBusy] = useState(false);
  const [err, setErr] = useState<string | null>(null);

  // Preview the invite (email + locale) before any session exists.
  useEffect(() => {
    if (!token) {
      setErr(t("invite.accept.missing_token"));
      return;
    }
    let live = true;
    authApi
      .inviteVerify(token)
      .then((preview) => {
        if (!live) return;
        setEmail(preview.email);
        if (preview.locale === "en" || preview.locale === "es") setLocale(preview.locale);
      })
      .catch(() => {
        if (live) setErr(t("invite.accept.failed"));
      });
    return () => {
      live = false;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [token]);

  async function onAccept(e: React.FormEvent) {
    e.preventDefault();
    if (!token) {
      setErr(t("invite.accept.missing_token"));
      return;
    }
    if (password.length < MIN_PASSWORD) {
      setErr(t("invite.accept.password_too_short"));
      return;
    }
    setBusy(true);
    setErr(null);
    try {
      const session = await authApi.inviteAccept(token, password);
      if (session.locale === "en" || session.locale === "es") setLocale(session.locale);
      nav("/workspaces", { replace: true });
    } catch {
      setErr(t("invite.accept.failed"));
    } finally {
      setBusy(false);
    }
  }

  return (
    <main className="mx-auto flex min-h-[100dvh] max-w-md flex-col px-6">
      <header className="flex items-center justify-end pt-[max(1.5rem,env(safe-area-inset-top))]">
        <ThemeControls />
      </header>
      <div className="m-auto w-full pb-16">
        <h1 className="text-[2rem] font-bold leading-tight tracking-tight text-foreground">
          {t("invite.accept.title")}
        </h1>
        <p className="mb-8 mt-2 text-[15px] leading-relaxed text-muted-foreground">
          {t("invite.accept.subtitle")}
        </p>
        <form onSubmit={onAccept} className="space-y-4">
          {email && (
            <div className="space-y-1.5">
              <Label htmlFor="invite-email">{t("auth.email")}</Label>
              <Input id="invite-email" type="email" value={email} readOnly disabled />
            </div>
          )}
          <div className="space-y-1.5">
            <Label htmlFor="invite-password">{t("invite.accept.password_label")}</Label>
            <Input
              id="invite-password"
              type="password"
              autoComplete="new-password"
              required
              minLength={MIN_PASSWORD}
              value={password}
              onChange={(ev) => setPassword(ev.target.value)}
            />
            <p className="text-[13px] text-muted-foreground">
              {t("invite.accept.password_hint")}
            </p>
          </div>
          {err && (
            <p role="alert" className="text-sm text-destructive">
              {err}
            </p>
          )}
          <Button type="submit" disabled={busy} size="lg" className="mt-2 w-full">
            {busy ? t("invite.accept.accepting") : t("invite.accept.cta")}
          </Button>
        </form>
      </div>
    </main>
  );
}
