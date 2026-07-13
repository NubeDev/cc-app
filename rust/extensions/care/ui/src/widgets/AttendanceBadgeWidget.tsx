import { useT } from "../hooks/useT";

export function AttendanceBadgeWidget() {
  const t = useT();
  return (
    <span className="inline-flex items-center rounded-full border border-border bg-card px-2.5 py-0.5 text-xs font-medium text-foreground">
      {t("attendance.checkIn")}
    </span>
  );
}
