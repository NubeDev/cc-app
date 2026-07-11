import { useNavigate } from "react-router-dom";
import { useWorkspaces } from "../hooks/useWorkspaces";
import { useT } from "../hooks/useT";
import { useEffect } from "react";

export function WorkspacePickerPage() {
  const t = useT();
  const { data, error } = useWorkspaces();
  const nav = useNavigate();
  useEffect(() => { if (error) nav("/login"); }, [error, nav]);

  return (
    <main className="mx-auto max-w-sm px-6 py-10">
      <h1 className="mb-4 text-xl font-semibold">{t("workspace.pick")}</h1>
      <ul className="space-y-2">
        {(data ?? []).map((w) => (
          <li key={w.id}>
            <button
              onClick={() => nav(`/ext/${w.id}`)}
              className="w-full rounded border px-4 py-3 text-left"
            >
              <div className="font-medium">{w.name}</div>
              <div className="text-xs opacity-70">{w.role}</div>
            </button>
          </li>
        ))}
      </ul>
    </main>
  );
}