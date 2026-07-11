import { useState } from "react";
import { useNavigate, useLocation } from "react-router-dom";
import { authApi } from "../api/auth";
import { useT } from "../hooks/useT";

export function LoginPage() {
  const t = useT();
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
    <main className="mx-auto flex min-h-screen max-w-sm flex-col justify-center px-6">
      <h1 className="mb-6 text-2xl font-semibold">{t("auth.welcome")}</h1>
      <form onSubmit={onSubmit} className="space-y-3">
        <label className="block">
          <span className="text-sm">{t("auth.email")}</span>
          <input
            type="email" required value={email}
            onChange={(e) => setEmail(e.target.value)}
            className="mt-1 block w-full rounded border px-3 py-2"
          />
        </label>
        <label className="block">
          <span className="text-sm">{t("auth.password")}</span>
          <input
            type="password" required value={password}
            onChange={(e) => setPassword(e.target.value)}
            className="mt-1 block w-full rounded border px-3 py-2"
          />
        </label>
        {err && <p className="text-sm text-red-600">{err}</p>}
        <button type="submit" className="w-full rounded bg-black py-3 text-white">
          {t("auth.signIn")}
        </button>
      </form>
    </main>
  );
}