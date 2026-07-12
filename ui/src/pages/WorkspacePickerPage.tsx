import { useNavigate } from "react-router-dom";
import { useWorkspaces } from "../hooks/useWorkspaces";
import { useT, useLocaleSwitch, useTheme } from "../hooks/useT";
import { useEffect } from "react";

export function WorkspacePickerPage() {
  const t = useT();
  const { locale, setLocale } = useLocaleSwitch();
  const { resolved, setTheme } = useTheme();
  const { data, error } = useWorkspaces();
  const nav = useNavigate();
  useEffect(() => { if (error) nav("/login"); }, [error, nav]);

  return (
    <main className="mx-auto max-w-2xl px-6 py-10">
      <header className="flex items-center justify-end gap-2 pb-6">
        <div className="flex overflow-hidden rounded-full border border-border bg-card text-xs">
          <button aria-pressed={locale === "en"} onClick={() => setLocale("en")} className={`px-3 py-1.5 font-medium ${locale === "en" ? "bg-primary text-primary-foreground" : "text-muted-foreground"}`}>EN</button>
          <button aria-pressed={locale === "es"} onClick={() => setLocale("es")} className={`px-3 py-1.5 font-medium ${locale === "es" ? "bg-primary text-primary-foreground" : "text-muted-foreground"}`}>ES</button>
        </div>
        <button onClick={() => setTheme(resolved === "dark" ? "light" : "dark")} className="flex h-8 w-8 items-center justify-center rounded-full border border-border bg-card text-sm" aria-label="Toggle theme">
          {resolved === "dark" ? "☼" : "☾"}
        </button>
      </header>
      <h1 className="mb-2 text-3xl font-semibold tracking-tight">{t("workspace.pick")}</h1>
      <p className="mb-6 text-sm text-muted-foreground">{t("app.title")}</p>
      <ul className="space-y-2">
        {(data ?? []).map((w) => (
          <li key={w.id}>
            <button onClick={() => nav(`/ext/${w.id}`)} className="w-full rounded-2xl border border-border bg-card p-4 text-left transition hover:border-primary">
              <div className="text-base font-medium">{w.name}</div>
              <div className="text-xs text-muted-foreground">{w.role}</div>
            </button>
          </li>
        ))}
      </ul>
    </main>
  );
}