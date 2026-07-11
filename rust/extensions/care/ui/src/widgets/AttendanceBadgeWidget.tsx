import { useT } from "../hooks/useT";

export function AttendanceBadgeWidget() {
  const t = useT();
  return (
    <span className="rounded-full border px-2 py-0.5 text-xs">
      {t("attendance.checkIn")}
    </span>
  );
}