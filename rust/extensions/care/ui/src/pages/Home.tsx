import { useState } from "react";
import { LargeTitle } from "../components/LargeTitle";
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
          <LargeTitle>{t("app.title")}</LargeTitle>
          <div className="px-4">
            {session && (
              <p className="text-[13px] capitalize text-muted-foreground">
                {session.role} · {session.workspaceId}
              </p>
            )}
            <div className="flex flex-col items-center gap-2 py-20 text-center">
              <p className="text-[15px] text-muted-foreground">{t("feed.empty")}</p>
            </div>
          </div>
        </main>
      )}
      {tab === "children" && <ChildrenListPage />}
      {tab === "admin" && isAdmin && <AdminHomePage />}

      <TabBar active={tab} onChange={setTab} showAdmin={isAdmin} />
    </div>
  );
}
