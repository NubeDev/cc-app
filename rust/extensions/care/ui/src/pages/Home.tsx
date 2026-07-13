import { useState } from "react";
import { LargeTitle } from "../components/LargeTitle";
import { TabBar } from "../components/TabBar";
import type { TabKey } from "../components/TabBar";
import { useT } from "../hooks/useT";
import { useCareSession } from "../hooks/useCareSession";
import { AdminHomePage } from "./admin/AdminHomePage";
import { ChildrenListPage } from "./child/ChildrenListPage";
import { AttendancePage } from "./attendance/AttendancePage";
import { GuardianWeekPage } from "./menu/GuardianWeekPage";
import { ServingViewPage } from "./menu/ServingViewPage";
import { MenuPlannerPage } from "./menu/MenuPlannerPage";

export function HomePage() {
  const t = useT();
  const session = useCareSession();
  const [tab, setTab] = useState<TabKey>("today");
  const role = session?.role ?? "guardian";
  const isAdmin = role === "admin";
  const isStaff = role === "staff" || role === "kiosk";

  // The Menus tab is universal but role-specific: a guardian sees their child's
  // week (with their child's substitutions), staff see the serving view (red
  // allergen flags), an admin gets the week × slot planner.
  function menusSurface() {
    if (isAdmin) return <MenuPlannerPage />;
    if (isStaff) return <ServingViewPage />;
    return <GuardianWeekPage />;
  }

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
      {tab === "children" && isAdmin && <ChildrenListPage />}
      {tab === "attendance" && (isStaff || isAdmin) && <AttendancePage />}
      {tab === "menus" && menusSurface()}
      {tab === "admin" && isAdmin && <AdminHomePage />}

      <TabBar active={tab} onChange={setTab} showAdmin={isAdmin} />
    </div>
  );
}
