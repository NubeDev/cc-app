import { useState } from "react";
import { TabBar } from "../components/TabBar";
import type { TabKey } from "../components/TabBar";
import { useCareSession } from "../hooks/useCareSession";
import { AdminHomePage } from "./admin/AdminHomePage";
import { ChildrenListPage } from "./child/ChildrenListPage";
import { AttendancePage } from "./attendance/AttendancePage";
import { GuardianWeekPage } from "./menu/GuardianWeekPage";
import { ServingViewPage } from "./menu/ServingViewPage";
import { MenuPlannerPage } from "./menu/MenuPlannerPage";
import { FeedPage } from "./feed/FeedPage";
import { LogEntryPage } from "./feed/LogEntryPage";

export function HomePage() {
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

  // The Today tab is the milestone-08 home. Staff land on the two-tap logging
  // flow (their primary daily gesture — children multi-select → type → done);
  // guardians and admins land on the live feed (SSE append, child switcher, day
  // rollup, incident ack) filtered to their reached children by the chokepoint.
  function todaySurface() {
    if (isStaff) return <LogEntryPage />;
    return <FeedPage />;
  }

  return (
    <div>
      {tab === "today" && todaySurface()}
      {tab === "children" && isAdmin && <ChildrenListPage />}
      {tab === "attendance" && (isStaff || isAdmin) && <AttendancePage />}
      {tab === "menus" && menusSurface()}
      {tab === "admin" && isAdmin && <AdminHomePage />}

      <TabBar active={tab} onChange={setTab} showAdmin={isAdmin} />
    </div>
  );
}
