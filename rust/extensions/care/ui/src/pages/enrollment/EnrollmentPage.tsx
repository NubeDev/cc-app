import { useEffect, useState } from "react";
import { Plus, ChevronLeft, ChevronRight } from "lucide-react";
import { useCareApi } from "../../api/care";
import { LargeTitle } from "../../components/LargeTitle";
import { Field } from "../../components/Field";
import { Button } from "../../components/ui/button";
import { Input } from "../../components/ui/input";
import { Segmented } from "../../components/ui/segmented";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "../../components/ui/select";
import { cn } from "../../lib/cn";
import { useT } from "../../hooks/useT";

export interface EnrollmentRow {
  id?: string;
  child_id: string;
  room_id: string;
  status: "enrolled" | "waitlist" | "withdrawn";
  schedule?: string[];
  waitlist_seq?: number;
  start_date?: string;
}
export interface ChildRow { id: string; name: string; }
export interface RoomRow { id: string; name: string; }

const DAYS = ["mon", "tue", "wed", "thu", "fri", "sat", "sun"] as const;
const STATUSES = ["enrolled", "waitlist", "withdrawn"] as const;

export function EnrollmentListPage({ embedded }: { embedded?: boolean } = {}) {
  const t = useT();
  const api = useCareApi();
  const [rows, setRows] = useState<EnrollmentRow[] | null>(null);
  const [rooms, setRooms] = useState<RoomRow[]>([]);
  const [children, setChildren] = useState<ChildRow[]>([]);
  const [editing, setEditing] = useState<EnrollmentRow | null>(null);
  const [creating, setCreating] = useState(false);
  const [waitlistRoom, setWaitlistRoom] = useState<string | null>(null);

  async function refresh() {
    const [r, ro, c] = await Promise.all([
      api.list<EnrollmentRow>("enrollment"),
      api.list<RoomRow>("room"),
      api.list<ChildRow>("child"),
    ]);
    setRows(r);
    setRooms(ro);
    setChildren(c);
  }
  useEffect(() => { refresh().catch(() => {}); }, []);

  if (editing || creating) {
    return <EnrollmentEditor initial={editing} rooms={rooms} children={children} onDone={() => { setEditing(null); setCreating(false); refresh(); }} />;
  }
  if (waitlistRoom) {
    const filtered = (rows ?? []).filter((r) => r.room_id === waitlistRoom && r.status === "waitlist").sort((a, b) => (a.waitlist_seq ?? 0) - (b.waitlist_seq ?? 0));
    return (
      <main className="pb-24">
        <LargeTitle>{t("enrollment.waitlist.title")}</LargeTitle>
        <div className="px-4 pt-1">
          <Button variant="ghost" size="sm" className="mb-3 -ml-2 text-primary" onClick={() => setWaitlistRoom(null)}>
            <ChevronLeft /> {t("common.back")}
          </Button>
          <h2 className="pb-3 text-base font-semibold text-foreground">{rooms.find((r) => r.id === waitlistRoom)?.name}</h2>
          {!filtered.length ? (
            <p className="py-16 text-center text-[15px] text-muted-foreground">{t("enrollment.waitlist_empty")}</p>
          ) : (
            <ol className="space-y-2">
              {filtered.map((r, i) => {
                const child = children.find((c) => c.id === r.child_id);
                return (
                  <li key={r.id ?? `${r.child_id}-${i}`} className="flex items-center gap-3 rounded-2xl border border-border bg-card p-4 shadow-sm">
                    <span className="flex size-8 items-center justify-center rounded-full bg-primary/10 text-sm font-semibold text-primary">{i + 1}</span>
                    <span className="flex-1 font-medium text-foreground">{child?.name ?? r.child_id}</span>
                    <span className="text-xs text-muted-foreground">{r.waitlist_seq ? t("enrollment.position", { position: String(r.waitlist_seq) }) : ""}</span>
                  </li>
                );
              })}
            </ol>
          )}
        </div>
      </main>
    );
  }

  const byRoom = new Map<string, EnrollmentRow[]>();
  for (const r of rows ?? []) {
    const list = byRoom.get(r.room_id) ?? [];
    list.push(r);
    byRoom.set(r.room_id, list);
  }

  return (
    <main className={embedded ? "" : "pb-24"}>
      {!embedded && <LargeTitle>{t("enrollment.list.title")}</LargeTitle>}
      <div className="px-4 pt-1">
        <Button className="mb-4 w-full" onClick={() => setCreating(true)}>
          <Plus /> {t("enrollment.editor.title.new")}
        </Button>
        {rooms.map((room) => {
          const list = (byRoom.get(room.id) ?? []).filter((r) => r.status !== "withdrawn");
          const wait = list.filter((r) => r.status === "waitlist").sort((a, b) => (a.waitlist_seq ?? 0) - (b.waitlist_seq ?? 0));
          const enrolled = list.filter((r) => r.status === "enrolled");
          return (
            <section key={room.id} className="mb-4 rounded-2xl border border-border bg-card p-4 shadow-sm">
              <header className="flex items-baseline justify-between">
                <h2 className="text-base font-semibold text-foreground">{room.name}</h2>
                <span className="text-xs text-muted-foreground">{enrolled.length} {t("enrollment.status.enrolled")}</span>
              </header>
              {wait.length > 0 && (
                <button onClick={() => setWaitlistRoom(room.id)} className="mt-3 flex w-full items-center justify-between rounded-xl bg-muted px-3 py-2.5 text-left text-sm text-foreground transition-colors hover:bg-accent">
                  <span>{wait.length} {t("enrollment.status.waitlist")}</span>
                  <ChevronRight className="size-4 text-muted-foreground" aria-hidden />
                </button>
              )}
              {enrolled.length > 0 && (
                <ul className="mt-3 space-y-1">
                  {enrolled.map((r) => {
                    const child = children.find((c) => c.id === r.child_id);
                    return (
                      <li key={r.id ?? `${r.child_id}-${r.room_id}`}>
                        <button onClick={() => setEditing(r)} className="flex w-full items-center gap-2 rounded-lg px-1 py-1.5 text-left text-sm text-foreground transition-colors hover:bg-accent">
                          <span className="font-medium">{child?.name ?? r.child_id}</span>
                          {r.schedule && r.schedule.length > 0 && (
                            <span className="text-xs text-muted-foreground">{r.schedule.map((d) => t(`enrollment.day.${d}`)).join(" ")}</span>
                          )}
                        </button>
                      </li>
                    );
                  })}
                </ul>
              )}
            </section>
          );
        })}
      </div>
    </main>
  );
}

function EnrollmentEditor({ initial, rooms, children, onDone }: { initial: EnrollmentRow | null; rooms: RoomRow[]; children: ChildRow[]; onDone: () => void }) {
  const t = useT();
  const api = useCareApi();
  const [childId, setChildId] = useState(initial?.child_id ?? children[0]?.id ?? "");
  const [roomId, setRoomId] = useState(initial?.room_id ?? rooms[0]?.id ?? "");
  const [status, setStatus] = useState<"enrolled" | "waitlist" | "withdrawn">(initial?.status ?? "enrolled");
  const [schedule, setSchedule] = useState<string[]>(initial?.schedule ?? []);
  const [startDate, setStartDate] = useState(initial?.start_date ?? "");
  const [busy, setBusy] = useState(false);

  async function save() {
    setBusy(true);
    try {
      if (initial?.child_id && initial.room_id) {
        await api.run("enrollment.update", {
          child_id: initial.child_id,
          room_id: initial.room_id,
          status,
          schedule,
          start_date: startDate || undefined,
        });
      } else {
        await api.run("enrollment.create", {
          child_id: childId,
          room_id: roomId,
          status,
          schedule,
          start_date: startDate || undefined,
        });
      }
      onDone();
    } finally { setBusy(false); }
  }

  function toggleDay(d: string) {
    setSchedule((s) => s.includes(d) ? s.filter((x) => x !== d) : [...s, d]);
  }

  return (
    <main className="pb-24">
      <LargeTitle>{initial ? t("enrollment.editor.title.edit") : t("enrollment.editor.title.new")}</LargeTitle>
      <form onSubmit={(e) => { e.preventDefault(); save(); }} className="space-y-4 px-4 pt-1">
        <Field label={t("enrollment.child")}>
          <Select value={childId} onValueChange={setChildId} disabled={!!initial}>
            <SelectTrigger><SelectValue /></SelectTrigger>
            <SelectContent>
              {children.map((c) => <SelectItem key={c.id} value={c.id}>{c.name}</SelectItem>)}
            </SelectContent>
          </Select>
        </Field>
        <Field label={t("enrollment.room")}>
          <Select value={roomId} onValueChange={setRoomId} disabled={!!initial}>
            <SelectTrigger><SelectValue /></SelectTrigger>
            <SelectContent>
              {rooms.map((r) => <SelectItem key={r.id} value={r.id}>{r.name}</SelectItem>)}
            </SelectContent>
          </Select>
        </Field>
        <Field label={t("enrollment.status")}>
          <Segmented
            columns={STATUSES.length}
            value={status}
            onChange={setStatus}
            segments={STATUSES.map((s) => ({ value: s, label: t(`enrollment.status.${s}`) }))}
          />
        </Field>
        <Field label={t("enrollment.schedule")}>
          <div className="grid grid-cols-7 gap-1.5">
            {DAYS.map((d) => {
              const on = schedule.includes(d);
              return (
                <button
                  key={d}
                  type="button"
                  aria-pressed={on}
                  onClick={() => toggleDay(d)}
                  className={cn(
                    "rounded-xl border py-2.5 text-xs font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
                    on ? "border-primary bg-primary text-primary-foreground" : "border-border bg-card text-muted-foreground hover:text-foreground",
                  )}
                >
                  {t(`enrollment.day.${d}`)}
                </button>
              );
            })}
          </div>
        </Field>
        <Field label={t("enrollment.start_date")} htmlFor="en-start">
          <Input id="en-start" type="date" value={startDate} onChange={(e) => setStartDate(e.target.value)} />
        </Field>
        <div className="flex gap-2 pt-2">
          <Button type="button" variant="outline" className="flex-1" onClick={onDone}>{t("common.cancel")}</Button>
          <Button type="submit" className="flex-1" disabled={busy}>{t("common.save")}</Button>
        </div>
      </form>
    </main>
  );
}
