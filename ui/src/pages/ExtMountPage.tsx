import { useEffect, useRef, useState } from "react";
import { useNavigate, useParams } from "react-router-dom";
import { Home, LogOut } from "lucide-react";
import { useT } from "../hooks/useT";
import { gateway } from "../api/gateway";
import { authApi } from "../api/auth";
import { ThemeControls } from "../components/ThemeControls";
import { Button } from "../components/ui/button";

// Load the care remote's federation entry. In production the host fetches the
// signed, built bundle via the gateway; in dev both live in this repo against
// the same React, so we dynamic-import the remote's `defineRemote(...)` result
// directly. Either way we get the SDK's `{ mount }` contract.
const loadCareRemote = () =>
  import("../../../rust/extensions/care/ui/src/remoteEntry").then((m) => m.default);

// The leashed bridge: a page's ONLY reach to the platform is a host-mediated,
// caps-checked MCP call. Route it through the host gateway client.
const bridge = {
  call: <T,>(tool: string, args?: Record<string, unknown>): Promise<T> =>
    gateway<T>("/api/mcp/call", {
      method: "POST",
      body: JSON.stringify({ tool, args: args ?? {} }),
    }),
};

export function ExtMountPage() {
  const t = useT();
  const nav = useNavigate();
  const { workspaceId = "" } = useParams();
  const hostRef = useRef<HTMLDivElement>(null);
  // Loading until the remote's `mount` has run. IMPORTANT: the SDK's `mountScoped`
  // APPENDS a scoped `[data-ext-root]` into the host el (it never clears it), so a
  // skeleton placed INSIDE the mount target would never be removed — it must be a
  // SEPARATE sibling we hide once `mount` completes (the "perpetual skeleton" bug).
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    if (!hostRef.current) return;
    const el = hostRef.current;
    let teardown: void | (() => void);
    let cancelled = false;

    setLoading(true);
    loadCareRemote()
      .then((remote) => {
        if (cancelled || !hostRef.current) return;
        teardown = remote.mount(el, { workspace: workspaceId }, bridge);
        setLoading(false);
      })
      .catch(() => {
        // A failed remote load must not spin forever — drop the skeleton so the
        // (empty) mount host shows rather than an eternal loading state.
        if (!cancelled) setLoading(false);
      });

    return () => {
      cancelled = true;
      if (typeof teardown === "function") teardown();
    };
  }, [workspaceId]);

  // Switch workspace: back to the picker (the shell's home). Tears down happens
  // via the effect cleanup when this route unmounts.
  function goWorkspaces() {
    nav("/workspaces");
  }

  // Sign out: clear the server-side session, then land on /login.
  async function signOut() {
    try {
      await authApi.logout();
    } finally {
      nav("/login", { replace: true });
    }
  }

  return (
    <div className="relative min-h-[100dvh]">
      {/* Shell chrome — the ONE way out of the mounted ext (switch workspace /
          sign out). It belongs to the host, not the extension, so it lives here
          above the mount, not in the ext's own UI. Sticky + translucent so it
          stays reachable while the ext scrolls under it. */}
      <header className="sticky top-0 z-30 flex items-center justify-between gap-2 border-b border-border/70 bg-background/80 px-3 py-2 backdrop-blur-xl pt-[max(0.5rem,env(safe-area-inset-top))]">
        <Button variant="ghost" size="sm" onClick={goWorkspaces} className="gap-1.5">
          <Home className="size-4" aria-hidden />
          {t("shell.workspaces")}
        </Button>
        <div className="flex items-center gap-2">
          <ThemeControls />
          <Button
            variant="outline"
            size="icon"
            onClick={signOut}
            aria-label={t("auth.signOut")}
            title={t("auth.signOut")}
          >
            <LogOut className="size-4" aria-hidden />
          </Button>
        </div>
      </header>

      <main className="relative min-h-[calc(100dvh-49px)]">
        {/* The SDK appends the ext's scoped root here; this stays empty until then. */}
        <div ref={hostRef} className="min-h-[calc(100dvh-49px)]" />
        {loading && (
          <div
            aria-busy="true"
            aria-label={t("shell.loading")}
            className="pointer-events-none absolute inset-x-0 top-0"
          >
            <div className="mx-auto max-w-2xl space-y-4 px-4 pt-14">
              <div className="h-8 w-40 animate-pulse rounded-lg bg-muted" />
              <div className="h-24 animate-pulse rounded-2xl bg-muted" />
              <div className="h-24 animate-pulse rounded-2xl bg-muted" />
            </div>
          </div>
        )}
      </main>
    </div>
  );
}
