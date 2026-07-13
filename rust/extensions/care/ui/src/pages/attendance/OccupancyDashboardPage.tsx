import { useEffect, useState } from "react";
import { RefreshCw, TriangleAlert, Users } from "lucide-react";
import { useCareApi } from "../../api/care";
import { LargeTitle } from "../../components/LargeTitle";
import { Button } from "../../components/ui/button";
import { useT } from "../../hooks/useT";
import type { RoomRow } from "../child/ChildrenListPage";

const RATIO_WARN_THRESHOLD = 8;

interface Occupancy {
  room_id: string;
  children: number;
  staff: number;
  ratio?: number;
}

export function OccupancyDashboardPage({ embedded }: { embedded?: boolean } = {}) {
  const t = useT();
  const api = useCareApi();
  const [rows, setRows] = useState<Occupancy[] | null>(null);
  const [roomNames, setRoomNames] = useState<Record<string, string>>({});
  const [busy, setBusy] = useState(false);

  async function refresh() {
    setBusy(true);
    try {
      const [now, rooms] = await Promise.all([
        api.run<Occupancy[]>("attendance.now", {}),
        api.list<RoomRow>("room"),
      ]);
      setRoomNames(Object.fromEntries(rooms.map((r) => [r.id, r.name])));
      setRows(now);
    } catch {
      setRows([]);
    } finally {
      setBusy(false);
    }
  }

  useEffect(() => { refresh().catch(() => {}); /* eslint-disable-next-line react-hooks/exhaustive-deps */ }, []);

  return (
    <main className={embedded ? "" : "pb-24"}>
      {!embedded && (
        <LargeTitle
          trailing={
            <Button size="pill" variant="outline" onClick={() => refresh()} disabled={busy}>
              <RefreshCw className={busy ? "motion-safe:animate-spin" : ""} /> {t("attendance.refresh")}
            </Button>
          }
        >
          {t("attendance.dashboard.title")}
        </LargeTitle>
      )}

      <div className="px-4">
        {embedded && (
          <div className="flex justify-end pb-3">
            <Button size="pill" variant="outline" onClick={() => refresh()} disabled={busy}>
              <RefreshCw className={busy ? "motion-safe:animate-spin" : ""} /> {t("attendance.refresh")}
            </Button>
          </div>
        )}

        {rows === null ? (
          <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3" aria-busy="true">
            {[0, 1, 2].map((i) => <div key={i} className="h-32 animate-pulse rounded-2xl bg-muted" />)}
          </div>
        ) : !rows.length ? (
          <p className="py-16 text-center text-[15px] text-muted-foreground">{t("attendance.dashboard_empty")}</p>
        ) : (
          <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3">
            {rows.map((r) => <RoomTile key={r.room_id} row={r} name={roomNames[r.room_id] ?? r.room_id} />)}
          </div>
        )}
      </div>
    </main>
  );
}

function RoomTile({ row, name }: { row: Occupancy; name: string }) {
  const t = useT();
  const noStaff = row.ratio === undefined || row.staff === 0;
  const ratioHigh = row.ratio !== undefined && row.ratio > RATIO_WARN_THRESHOLD;
  const warn = noStaff || ratioHigh;

  return (
    <div
      className={
        "rounded-2xl border p-4 shadow-sm " +
        (warn ? "border-warning bg-warning/10" : "border-border bg-card")
      }
    >
      <div className="flex items-start justify-between gap-2">
        <h2 className="min-w-0 truncate text-lg font-semibold text-foreground">{name}</h2>
        {warn && <TriangleAlert className="size-5 shrink-0 text-warning" aria-hidden />}
      </div>

      <div className="mt-3 flex items-end gap-4">
        <div>
          <p className="text-3xl font-bold leading-none text-foreground">{row.children}</p>
          <p className="pt-1 text-xs font-medium text-muted-foreground">{t("attendance.children")}</p>
        </div>
        <div>
          <p className={"text-3xl font-bold leading-none " + (noStaff ? "text-warning" : "text-foreground")}>{row.staff}</p>
          <p className="pt-1 text-xs font-medium text-muted-foreground">{t("attendance.staff")}</p>
        </div>
        <div className="ml-auto text-right">
          <p className={"inline-flex items-center gap-1 text-2xl font-bold leading-none " + (warn ? "text-warning" : "text-foreground")}>
            <Users className="size-5" aria-hidden />
            {row.ratio !== undefined ? `1:${row.ratio}` : "—"}
          </p>
          <p className="pt-1 text-xs font-medium text-muted-foreground">{t("attendance.ratio")}</p>
        </div>
      </div>

      {warn && (
        <p className="mt-3 text-sm font-semibold text-warning">
          {noStaff ? t("attendance.no_staff") : t("attendance.ratio_high")}
        </p>
      )}
    </div>
  );
}
