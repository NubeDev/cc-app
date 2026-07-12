import { useState } from "react";
import { PageTitle } from "../components/PageTitle";
import { TabBar } from "../components/TabBar";
import { useT } from "../hooks/useT";
import { useCareSession } from "../hooks/useCareSession";
import { AdminHomePage } from "./admin/AdminHomePage";
import { ChildrenListPage } from "./child/ChildrenListPage";

type Tab = "today" | "children" | "admin";

export function HomePage() {
  const t = useT();
  const session = useCareSession();
  const [tab, setTab] = useState<Tab>("today");
  const isAdmin = session?.role === "admin";

  return (
    <div>
      {tab === "today" && (
        <main className="pb-24">
          <PageTitle>{t("app.title")}</PageTitle>
          <div className="px-4">
            {session ? (
              <div className="space-y-1">
                <p className="text-sm text-muted-foreground">{session.role} · {session.workspaceId}</p>
                <p className="text-xs opacity-60">{session.locale}</p>
              </div>
            ) : (
              <p className="py-6 text-sm opacity-60">{t("auth.signIn")}</p>
            )}
            <p className="mt-8 py-10 text-center text-sm opacity-50">{t("feed.empty")}</p>
          </div>
        </main>
      )}
      {tab === "children" && <ChildrenListPage />}
      {tab === "admin" && isAdmin && <AdminHomePage />}

      <TabBar active={tab} onChange={setTab} showAdmin={isAdmin} />
    </div>
  );
}