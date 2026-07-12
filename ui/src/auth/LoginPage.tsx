import { useState } from "react";
import { useNavigate, useLocation } from "react-router-dom";
import { authApi } from "../api/auth";
import { useT, useLocaleSwitch, useTheme } from "../hooks/useT";

export function LoginPage() {
  const t = useT();
  const { locale, setLocale } = useLocaleSwitch();
  const { resolved, setTheme } = useTheme();
  const nav = useNavigate();
  const loc = useLocation();
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [err, setErr] = useState<string | null>(null);

  async function onSubmit(e: React.FormEvent) {
    e.preventDefault();
    setErr(null);
    try {
      await authApi.login({ email, password });
      const next = (loc.state as { from?: string } | null)?.from ?? "/workspaces";
      nav(next, { replace: true });
    } catch {
      setErr(t("error.denied"));
    }
  }

  return (
    <main className="mx-auto flex min-h-screen max-w-md flex-col px-6">
      <header className="flex items-center justify-end gap-2 pt-6">
        <div className="flex overflow-hidden rounded-full border border-border bg-card text-xs">
          <button aria-pressed={locale === "en"} onClick={() => setLocale("en")} className={`px-3 py-1.5 font-medium ${locale === "en" ? "bg-primary text-primary-foreground" : "text-muted-foreground"}`}>EN</button>
          <button aria-pressed={locale === "es"} onClick={() => setLocale("es")} className={`px-3 py-1.5 font-medium ${locale === "es" ? "bg-primary text-primary-foreground" : "text-muted-foreground"}`}>ES</button>
        </div>
        <button onClick={() => setTheme(resolved === "dark" ? "light" : "dark")} className="flex h-8 w-8 items-center justify-center rounded-full border border-border bg-card text-sm" aria-label="Toggle theme">
          {resolved === "dark" ? "☼" : "☾"}
        </button>
      </header>
      <div className="m-auto w-full pb-12">
        <h1 className="mb-1 text-3xl font-semibold tracking-tight">{t("app.title")}</h1>
        <p className="mb-8 text-sm text-muted-foreground">{t("auth.welcome")}</p>
        <form onSubmit={onSubmit} className="space-y-3">
          <label className="block">
            <span className="block pb-1.5 text-sm text-foreground">{t("auth.email")}</span>
            <input type="email" required value={email} onChange={(e) => setEmail(e.target.value)} className="block w-full rounded-xl border border-border bg-card px-4 py-3 text-base outline-none focus:border-primary" />
          </label>
          <label className="block">
            <span className="block pb-1.5 text-sm text-foreground">{t("auth.password")}</span>
            <input type="password" required value={password} onChange={(e) => setPassword(e.target.value)} className="block w-full rounded-xl border border-border bg-card px-4 py-3 text-base outline-none focus:border-primary" />
          </label>
          {err && <p className="text-sm text-destructive">{err}</p>}
          <button type="submit" className="mt-2 w-full rounded-xl bg-primary px-4 py-3.5 text-base font-medium text-primary-foreground">
            {t("auth.signIn")}
          </button>
        </form>
      </div>
    </main>
  );
}