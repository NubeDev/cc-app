import { useState } from "react";
import { LargeTitle } from "../../components/LargeTitle";
import { Segmented } from "../../components/ui/segmented";
import { useT } from "../../hooks/useT";
import { KioskRosterPage } from "./KioskRosterPage";
import { OccupancyDashboardPage } from "./OccupancyDashboardPage";

type Tab = "roster" | "dashboard";

// The single entry the nav mounts: a segmented control over the staff/kiosk
// roster and the admin occupancy dashboard.
export function AttendancePage() {
  const t = useT();
  const [tab, setTab] = useState<Tab>("roster");

  return (
    <main className="pb-24">
      <LargeTitle>{t("attendance.title")}</LargeTitle>
      <div className="px-4 pb-4">
        <Segmented<Tab>
          columns={2}
          value={tab}
          onChange={setTab}
          segments={[
            { value: "roster", label: t("attendance.segment.roster") },
            { value: "dashboard", label: t("attendance.segment.dashboard") },
          ]}
        />
      </div>
      {tab === "roster" ? <KioskRosterPage embedded /> : <OccupancyDashboardPage embedded />}
    </main>
  );
}

export default AttendancePage;
