import { useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { ChevronRight } from "lucide-react";
import { useWorkspaces } from "../hooks/useWorkspaces";
import { useT } from "../hooks/useT";
import { ThemeControls } from "../components/ThemeControls";

export function WorkspacePickerPage() {
  const t = useT();
  const { data, error } = useWorkspaces();
  const nav = useNavigate();
  useEffect(() => {
    if (error) nav("/login");
  }, [error, nav]);

  const loading = !data && !error;

  return (
    <main className="mx-auto max-w-2xl px-6 pb-16">
      <header className="flex items-center justify-end pt-[max(1.5rem,env(safe-area-inset-top))]">
        <ThemeControls />
      </header>
      <h1 className="mt-4 text-[2rem] font-bold leading-tight tracking-tight text-foreground">
        {t("workspace.pick")}
      </h1>
      <p className="mb-6 mt-1 text-[15px] text-muted-foreground">{t("app.title")}</p>

      {loading ? (
        <ul className="divide-y divide-border overflow-hidden rounded-2xl border border-border bg-card">
          {[0, 1].map((i) => (
            <li key={i} className="flex items-center gap-3 px-4 py-4">
              <div className="h-4 w-40 animate-pulse rounded-md bg-muted" />
            </li>
          ))}
        </ul>
      ) : (
        <ul className="divide-y divide-border overflow-hidden rounded-2xl border border-border bg-card shadow-sm">
          {(data ?? []).map((w) => (
            <li key={w.id}>
              <button
                onClick={() => nav(`/ext/${w.id}`)}
                className="flex w-full items-center justify-between gap-3 px-4 py-4 text-left transition-colors hover:bg-accent focus-visible:bg-accent focus-visible:outline-none"
              >
                <span className="min-w-0">
                  <span className="block truncate text-base font-medium text-foreground">{w.name}</span>
                  <span className="block truncate text-[13px] text-muted-foreground">{w.role}</span>
                </span>
                <ChevronRight className="size-5 shrink-0 text-muted-foreground" aria-hidden />
              </button>
            </li>
          ))}
        </ul>
      )}
    </main>
  );
}
