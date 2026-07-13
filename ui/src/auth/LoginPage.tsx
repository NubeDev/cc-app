import { useState } from "react";
import { useNavigate, useLocation } from "react-router-dom";
import { authApi } from "../api/auth";
import { useT } from "../hooks/useT";
import { ThemeControls } from "../components/ThemeControls";
import { Button } from "../components/ui/button";
import { Input } from "../components/ui/input";
import { Label } from "../components/ui/label";

export function LoginPage() {
  const t = useT();
  const nav = useNavigate();
  const loc = useLocation();
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [err, setErr] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  async function onSubmit(e: React.FormEvent) {
    e.preventDefault();
    setErr(null);
    setBusy(true);
    try {
      await authApi.login({ email, password });
      const next = (loc.state as { from?: string } | null)?.from ?? "/workspaces";
      nav(next, { replace: true });
    } catch {
      setErr(t("error.denied"));
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
          {t("app.title")}
        </h1>
        <p className="mb-8 mt-1 text-[15px] text-muted-foreground">{t("auth.welcome")}</p>
        <form onSubmit={onSubmit} className="space-y-4">
          <div className="space-y-1.5">
            <Label htmlFor="email">{t("auth.email")}</Label>
            <Input
              id="email"
              // `text`, not `email`: lb logs in by HANDLE (`ada` → `user:ada`,
              // the seeded dev admin) as well as by email, and the browser's
              // email validation would reject a bare handle. The dev-auth seam
              // (vite-dev-auth.ts) accepts either.
              type="text"
              autoComplete="username"
              required
              value={email}
              onChange={(e) => setEmail(e.target.value)}
            />
          </div>
          <div className="space-y-1.5">
            <Label htmlFor="password">{t("auth.password")}</Label>
            <Input
              id="password"
              type="password"
              autoComplete="current-password"
              required
              value={password}
              onChange={(e) => setPassword(e.target.value)}
            />
          </div>
          {err && (
            <p role="alert" className="text-sm text-destructive">
              {err}
            </p>
          )}
          <Button type="submit" size="lg" disabled={busy} className="mt-2 w-full">
            {t("auth.signIn")}
          </Button>
        </form>
      </div>
    </main>
  );
}
