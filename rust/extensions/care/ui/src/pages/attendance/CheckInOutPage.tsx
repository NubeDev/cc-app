import { useCareSession } from "../../hooks/useCareSession";
import { KioskRosterPage } from "./KioskRosterPage";
import { OccupancyDashboardPage } from "./OccupancyDashboardPage";

// Role-routed landing: admins land on the occupancy dashboard, everyone else
// (staff / kiosk) on the check-in/out roster. AttendancePage is the preferred
// nav entry (segmented control over both); this is the by-role alternative.
export function CheckInOutPage() {
  const session = useCareSession();
  if (session?.role === "admin") return <OccupancyDashboardPage />;
  return <KioskRosterPage />;
}

export default CheckInOutPage;
