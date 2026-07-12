import { useEffect, useState } from "react";
import { useCareApi } from "../../api/care";
import { PageTitle } from "../../components/PageTitle";
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

export function EnrollmentListPage() {
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
        <PageTitle>{t("enrollment.waitlist.title")}</PageTitle>
        <div className="px-4">
          <button onClick={() => setWaitlistRoom(null)} className="mb-4 text-sm text-primary">← {t("common.back")}</button>
          <h2 className="pb-3 text-base font-semibold">{rooms.find((r) => r.id === waitlistRoom)?.name}</h2>
          {!filtered.length ? (
            <p className="py-6 text-center text-sm opacity-60">{t("enrollment.waitlist_empty")}</p>
          ) : (
            <ol className="space-y-2">
              {filtered.map((r, i) => {
                const child = children.find((c) => c.id === r.child_id);
                return (
                  <li key={r.id ?? `${r.child_id}-${i}`} className="flex items-center gap-3 rounded-2xl border border-border bg-card p-4">
                    <span className="flex h-8 w-8 items-center justify-center rounded-full bg-primary/10 text-sm font-semibold text-primary">{i + 1}</span>
                    <span className="flex-1 font-medium">{child?.name ?? r.child_id}</span>
                    <span className="text-xs opacity-60">{r.waitlist_seq ? t("enrollment.position", { position: String(r.waitlist_seq) }) : ""}</span>
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
    <main className="pb-24">
      <PageTitle>{t("enrollment.list.title")}</PageTitle>
      <div className="px-4">
        <button onClick={() => setCreating(true)} className="mb-4 w-full rounded-2xl bg-primary px-4 py-3 font-medium text-primary-foreground">
          + {t("enrollment.editor.title.new")}
        </button>
        {rooms.map((room) => {
          const list = (byRoom.get(room.id) ?? []).filter((r) => r.status !== "withdrawn");
          const wait = list.filter((r) => r.status === "waitlist").sort((a, b) => (a.waitlist_seq ?? 0) - (b.waitlist_seq ?? 0));
          const enrolled = list.filter((r) => r.status === "enrolled");
          return (
            <section key={room.id} className="mb-4 rounded-2xl border border-border bg-card p-4">
              <header className="flex items-baseline justify-between">
                <h2 className="text-base font-semibold">{room.name}</h2>
                <span className="text-xs opacity-60">{enrolled.length} {t("enrollment.status.enrolled")}</span>
              </header>
              {wait.length > 0 && (
                <button onClick={() => setWaitlistRoom(room.id)} className="mt-2 block w-full rounded-xl bg-muted/60 px-3 py-2 text-left text-sm">
                  {wait.length} {t("enrollment.status.waitlist")} →
                </button>
              )}
              {enrolled.length > 0 && (
                <ul className="mt-3 space-y-1">
                  {enrolled.map((r) => {
                    const child = children.find((c) => c.id === r.child_id);
                    return (
                      <li key={r.id ?? `${r.child_id}-${r.room_id}`}>
                        <button onClick={() => setEditing(r)} className="block w-full text-left text-sm">
                          {child?.name ?? r.child_id}
                          {r.schedule && r.schedule.length > 0 && (
                            <span className="ml-2 text-xs opacity-60">{r.schedule.map((d) => t(`enrollment.day.${d}`)).join(" ")}</span>
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
      <PageTitle>{initial ? t("enrollment.editor.title.edit") : t("enrollment.editor.title.new")}</PageTitle>
      <form onSubmit={(e) => { e.preventDefault(); save(); }} className="space-y-4 px-4">
        <Field label={t("enrollment.child")}>
          <select value={childId} onChange={(e) => setChildId(e.target.value)} disabled={!!initial} className="w-full rounded-xl border border-border bg-card px-3 py-2.5 disabled:opacity-60">
            {children.map((c) => <option key={c.id} value={c.id}>{c.name}</option>)}
          </select>
        </Field>
        <Field label={t("enrollment.room")}>
          <select value={roomId} onChange={(e) => setRoomId(e.target.value)} disabled={!!initial} className="w-full rounded-xl border border-border bg-card px-3 py-2.5 disabled:opacity-60">
            {rooms.map((r) => <option key={r.id} value={r.id}>{r.name}</option>)}
          </select>
        </Field>
        <Field label={t("enrollment.status")}>
          <div className="grid grid-cols-3 gap-2">
            {(["enrolled", "waitlist", "withdrawn"] as const).map((s) => (
              <button key={s} type="button" onClick={() => setStatus(s)} className={`rounded-xl border px-2 py-2.5 text-xs ${status === s ? "border-primary bg-primary text-primary-foreground" : "border-border bg-card"}`}>
                {t(`enrollment.status.${s}`)}
              </button>
            ))}
          </div>
        </Field>
        <Field label={t("enrollment.schedule")}>
          <div className="grid grid-cols-7 gap-1">
            {DAYS.map((d) => (
              <button key={d} type="button" onClick={() => toggleDay(d)} className={`rounded-lg border px-1 py-2 text-xs ${schedule.includes(d) ? "border-primary bg-primary text-primary-foreground" : "border-border bg-card"}`}>
                {t(`enrollment.day.${d}`)}
              </button>
            ))}
          </div>
        </Field>
        <Field label={t("enrollment.start_date")}>
          <input type="date" value={startDate} onChange={(e) => setStartDate(e.target.value)} className="w-full rounded-xl border border-border bg-card px-3 py-2.5" />
        </Field>
        <div className="flex gap-2 pt-4">
          <button type="button" onClick={onDone} className="flex-1 rounded-xl border border-border px-4 py-3">{t("common.cancel")}</button>
          <button type="submit" disabled={busy} className="flex-1 rounded-xl bg-primary px-4 py-3 font-medium text-primary-foreground disabled:opacity-50">{t("common.save")}</button>
        </div>
      </form>
    </main>
  );
}

function Field({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <label className="block">
      <span className="block pb-1.5 text-sm text-muted-foreground">{label}</span>
      {children}
    </label>
  );
}