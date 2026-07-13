import { useEffect, useMemo, useState } from "react";
import { ChevronLeft, ChevronRight, Copy } from "lucide-react";
import { useCareApi } from "../../api/care";
import { LargeTitle } from "../../components/LargeTitle";
import { Field } from "../../components/Field";
import { Button } from "../../components/ui/button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "../../components/ui/select";
import { useT } from "../../hooks/useT";
import type { RoomRow } from "../child/ChildrenListPage";
import { Cell, CellEditor } from "./MenuPlannerCells";
import {
  SLOTS,
  isoDate,
  mondayOf,
  addDays,
  parseIso,
  weekDates,
  useAllergenLabel,
  type Menu,
  type Slot,
} from "./shared";

type CellKey = string; // `${date}|${slot}`
const cellKey = (date: string, slot: string): CellKey => `${date}|${slot}`;

// ADMIN planner — a real week × slot grid at laptop width, stacked on phone.
// The editor is a bottom sheet on phone / an inline overlay on desktop.
export function MenuPlannerPage() {
  const t = useT();
  const api = useCareApi();
  const label = useAllergenLabel();

  const [rooms, setRooms] = useState<RoomRow[] | null>(null);
  const [roomId, setRoomId] = useState("");
  const [weekStart, setWeekStart] = useState(() => isoDate(mondayOf(new Date())));
  const [menus, setMenus] = useState<Record<CellKey, Menu>>({});
  const [loading, setLoading] = useState(false);
  const [editing, setEditing] = useState<{ date: string; slot: Slot } | null>(null);
  const [copied, setCopied] = useState(false);

  const dates = useMemo(() => weekDates(weekStart), [weekStart]);

  useEffect(() => {
    api
      .list<RoomRow>("room")
      .then((r) => {
        setRooms(r);
        setRoomId((prev) => prev || r[0]?.id || "");
      })
      .catch(() => setRooms([]));
  }, []);

  async function loadWeek() {
    if (!roomId) return;
    setLoading(true);
    try {
      const pairs: Array<{ date: string; slot: Slot }> = [];
      for (const date of dates) for (const slot of SLOTS) pairs.push({ date, slot });
      const results = await Promise.all(
        pairs.map((p) =>
          api
            .run<Menu>("menu.get", { date: p.date, room_id: roomId, slot: p.slot })
            .catch(() => null),
        ),
      );
      const next: Record<CellKey, Menu> = {};
      pairs.forEach((p, i) => {
        const m = results[i];
        if (m && (m.items?.length || m.substitutions?.length))
          next[cellKey(p.date, p.slot)] = m;
      });
      setMenus(next);
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    loadWeek();
  }, [roomId, weekStart]);

  function shiftWeek(delta: number) {
    setWeekStart(isoDate(addDays(parseIso(weekStart), delta * 7)));
  }

  async function copyLastWeek() {
    if (!roomId) return;
    const from = isoDate(addDays(parseIso(weekStart), -7));
    await api.run("menu.copy_week", {
      room_id: roomId,
      from_week_start: from,
      to_week_start: weekStart,
    });
    setCopied(true);
    setTimeout(() => setCopied(false), 2500);
    await loadWeek();
  }

  function onSaved(m: Menu) {
    setMenus((prev) => ({ ...prev, [cellKey(m.date, m.slot)]: m }));
    setEditing(null);
  }

  return (
    <main className="pb-24">
      <LargeTitle>{t("menu.planner.title")}</LargeTitle>

      <div className="flex flex-wrap items-end gap-3 px-4 pb-4">
        <div className="min-w-[10rem] flex-1">
          <Field label={t("menu.pick_room")}>
            <Select value={roomId} onValueChange={setRoomId}>
              <SelectTrigger>
                <SelectValue placeholder="—" />
              </SelectTrigger>
              <SelectContent>
                {(rooms ?? []).map((r) => (
                  <SelectItem key={r.id} value={r.id}>
                    {r.name}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </Field>
        </div>
        <div className="flex items-center gap-1">
          <Button variant="outline" size="icon" onClick={() => shiftWeek(-1)} aria-label={t("menu.prev_week")}>
            <ChevronLeft />
          </Button>
          <span className="min-w-[7rem] text-center text-[13px] font-medium text-foreground">
            {t("menu.week_of", { date: weekStart })}
          </span>
          <Button variant="outline" size="icon" onClick={() => shiftWeek(1)} aria-label={t("menu.next_week")}>
            <ChevronRight />
          </Button>
        </div>
        <Button variant="secondary" onClick={copyLastWeek} disabled={!roomId}>
          <Copy /> {copied ? t("menu.copied") : t("menu.copy_week")}
        </Button>
      </div>

      {!roomId ? (
        <p className="px-4 py-16 text-center text-[15px] text-muted-foreground">{t("room.empty")}</p>
      ) : (
        <div className="px-4">
          {/* Desktop grid ≥1024px */}
          <div className="hidden overflow-x-auto lg:block">
            <table className="w-full border-separate border-spacing-1">
              <thead>
                <tr>
                  <th className="w-24" />
                  {SLOTS.map((slot) => (
                    <th
                      key={slot}
                      className="pb-1 text-left text-xs font-semibold uppercase tracking-wide text-muted-foreground"
                    >
                      {t("slot." + slot)}
                    </th>
                  ))}
                </tr>
              </thead>
              <tbody>
                {dates.map((date) => (
                  <tr key={date}>
                    <th className="pr-2 align-top text-left text-xs font-medium text-muted-foreground">
                      {new Date(date + "T00:00:00").toLocaleDateString(undefined, {
                        weekday: "short",
                        day: "numeric",
                      })}
                    </th>
                    {SLOTS.map((slot) => (
                      <td key={slot} className="align-top">
                        <Cell
                          menu={menus[cellKey(date, slot)]}
                          loading={loading}
                          label={label}
                          onClick={() => setEditing({ date, slot })}
                        />
                      </td>
                    ))}
                  </tr>
                ))}
              </tbody>
            </table>
          </div>

          {/* Phone: stacked by day */}
          <div className="space-y-6 lg:hidden">
            {dates.map((date) => (
              <section key={date}>
                <h2 className="px-1 pb-2 text-xs font-semibold uppercase tracking-wide text-muted-foreground">
                  {new Date(date + "T00:00:00").toLocaleDateString(undefined, {
                    weekday: "long",
                    month: "short",
                    day: "numeric",
                  })}
                </h2>
                <div className="space-y-2">
                  {SLOTS.map((slot) => (
                    <div key={slot}>
                      <div className="px-1 pb-1 text-[11px] font-semibold uppercase tracking-wide text-muted-foreground">
                        {t("slot." + slot)}
                      </div>
                      <Cell
                        menu={menus[cellKey(date, slot)]}
                        loading={loading}
                        label={label}
                        onClick={() => setEditing({ date, slot })}
                      />
                    </div>
                  ))}
                </div>
              </section>
            ))}
          </div>
        </div>
      )}

      {editing && (
        <CellEditor
          date={editing.date}
          slot={editing.slot}
          roomId={roomId}
          initial={menus[cellKey(editing.date, editing.slot)]}
          onClose={() => setEditing(null)}
          onSaved={onSaved}
        />
      )}
    </main>
  );
}

