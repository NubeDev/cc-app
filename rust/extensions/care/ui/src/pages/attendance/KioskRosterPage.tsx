import { useEffect, useState } from "react";
import { Check, LogOut, X, ShieldAlert } from "lucide-react";
import { useCareApi } from "../../api/care";
import { LargeTitle } from "../../components/LargeTitle";
import { Button } from "../../components/ui/button";
import { Input } from "../../components/ui/input";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "../../components/ui/select";
import { useT } from "../../hooks/useT";
import { useCareSession } from "../../hooks/useCareSession";
import type { ChildRow, RoomRow } from "../child/ChildrenListPage";

interface AttendanceEvent {
  event_id: string;
  child_id?: string;
  room_id: string;
  kind: "check_in" | "check_out";
  at: string;
}

type ChildState = "present" | "out";

function todayStartIso(): string {
  const d = new Date();
  d.setHours(0, 0, 0, 0);
  return d.toISOString();
}

// Latest event per child → present/out. Absence of any event = out.
function deriveStates(events: AttendanceEvent[]): Record<string, ChildState> {
  const latest: Record<string, AttendanceEvent> = {};
  for (const e of events) {
    if (!e.child_id) continue;
    const prev = latest[e.child_id];
    if (!prev || e.at > prev.at) latest[e.child_id] = e;
  }
  const out: Record<string, ChildState> = {};
  for (const [id, e] of Object.entries(latest)) out[id] = e.kind === "check_in" ? "present" : "out";
  return out;
}

export function KioskRosterPage({ embedded }: { embedded?: boolean } = {}) {
  const t = useT();
  const api = useCareApi();
  const session = useCareSession();
  const isAdmin = session?.role === "admin";

  const [rooms, setRooms] = useState<RoomRow[] | null>(null);
  const [roomId, setRoomId] = useState<string>("");
  const [children, setChildren] = useState<ChildRow[] | null>(null);
  const [states, setStates] = useState<Record<string, ChildState>>({});
  const [collecting, setCollecting] = useState<ChildRow | null>(null);
  const [toast, setToast] = useState<string | null>(null);

  useEffect(() => {
    api.list<RoomRow>("room").then((r) => {
      setRooms(r);
      if (r[0] && !roomId) setRoomId(r[0].id);
    }).catch(() => setRooms([]));
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  async function loadRoom(id: string) {
    if (!id) return;
    setChildren(null);
    const [allChildren, events] = await Promise.all([
      api.list<ChildRow>("child"),
      api.run<AttendanceEvent[]>("attendance.list", { room_id: id, since: todayStartIso() }),
    ]);
    setChildren(allChildren.filter((c) => !c.archived && c.room_id === id));
    setStates(deriveStates(events));
  }

  useEffect(() => { loadRoom(roomId).catch(() => setChildren([])); /* eslint-disable-next-line react-hooks/exhaustive-deps */ }, [roomId]);

  function flash(msg: string) {
    setToast(msg);
    setTimeout(() => setToast((cur) => (cur === msg ? null : cur)), 2500);
  }

  async function checkIn(child: ChildRow) {
    await api.run("attendance.check_in", {
      event_id: crypto.randomUUID(),
      child_id: child.id,
      room_id: roomId,
      at: new Date().toISOString(),
    });
    setStates((s) => ({ ...s, [child.id]: "present" }));
    flash(t("attendance.checked_in", { name: child.name }));
  }

  function onRow(child: ChildRow) {
    if (states[child.id] === "present") setCollecting(child);
    else checkIn(child).catch((e) => flash((e as Error).message));
  }

  return (
    <main className={embedded ? "" : "pb-24"}>
      {!embedded && <LargeTitle>{t("attendance.roster.title")}</LargeTitle>}

      <div className="px-4">
        {rooms === null ? (
          <div className="h-11 animate-pulse rounded-xl bg-muted" />
        ) : !rooms.length ? (
          <p className="py-16 text-center text-[15px] text-muted-foreground">{t("attendance.no_rooms")}</p>
        ) : (
          <Select value={roomId} onValueChange={setRoomId}>
            <SelectTrigger className="h-12 text-base"><SelectValue placeholder={t("attendance.select_room")} /></SelectTrigger>
            <SelectContent>
              {rooms.map((r) => <SelectItem key={r.id} value={r.id}>{r.name}</SelectItem>)}
            </SelectContent>
          </Select>
        )}

        {rooms?.length ? (
          <>
            <p className="pt-3 text-[13px] text-muted-foreground">{t("attendance.tap_to_check_in")}</p>
            {children === null ? (
              <ul className="space-y-2 pt-3" aria-busy="true">
                {[0, 1, 2, 3].map((i) => <li key={i} className="h-20 animate-pulse rounded-2xl bg-muted" />)}
              </ul>
            ) : !children.length ? (
              <p className="py-16 text-center text-[15px] text-muted-foreground">{t("attendance.empty_room")}</p>
            ) : (
              <ul className="space-y-2 pt-3">
                {children.map((c) => {
                  const present = states[c.id] === "present";
                  return (
                    <li key={c.id}>
                      <button
                        onClick={() => onRow(c)}
                        className="flex min-h-20 w-full items-center gap-4 rounded-2xl border border-border bg-card p-4 text-left shadow-sm transition-colors hover:bg-accent focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring active:scale-[0.99]"
                      >
                        <span
                          className={
                            "flex size-12 shrink-0 items-center justify-center rounded-full " +
                            (present ? "bg-success text-success-foreground" : "bg-muted text-muted-foreground")
                          }
                          aria-hidden
                        >
                          {present ? <Check className="size-6" /> : <LogOut className="size-6" />}
                        </span>
                        <span className="min-w-0 flex-1">
                          <span className="block truncate text-lg font-semibold text-foreground">{c.name}</span>
                          <span className={"block text-sm font-medium " + (present ? "text-success" : "text-muted-foreground")}>
                            {present ? t("attendance.present") : t("attendance.out")}
                          </span>
                        </span>
                        <span className="rounded-full bg-muted px-3 py-1.5 text-sm font-semibold text-foreground">
                          {present ? t("attendance.checkOut") : t("attendance.check_in_action")}
                        </span>
                      </button>
                    </li>
                  );
                })}
              </ul>
            )}
          </>
        ) : null}
      </div>

      {toast && (
        <div className="pointer-events-none fixed inset-x-0 bottom-24 z-40 flex justify-center px-4">
          <div className="rounded-full bg-success px-5 py-2.5 text-sm font-semibold text-success-foreground shadow-lg">
            {toast}
          </div>
        </div>
      )}

      {collecting && (
        <CheckOutSheet
          child={collecting}
          roomId={roomId}
          isAdmin={isAdmin}
          onClose={() => setCollecting(null)}
          onDone={(name) => {
            setStates((s) => ({ ...s, [collecting.id]: "out" }));
            setCollecting(null);
            flash(t("attendance.checked_out", { name }));
          }}
        />
      )}
    </main>
  );
}

function CheckOutSheet({
  child, roomId, isAdmin, onClose, onDone,
}: {
  child: ChildRow;
  roomId: string;
  isAdmin: boolean;
  onClose: () => void;
  onDone: (childName: string) => void;
}) {
  const t = useT();
  const api = useCareApi();
  const [full, setFull] = useState<ChildRow | null>(null);
  const [name, setName] = useState("");
  const [busy, setBusy] = useState(false);
  const [denied, setDenied] = useState<string | null>(null);

  useEffect(() => {
    api.get<ChildRow>("child", child.id).then(setFull).catch(() => setFull(child));
  }, [child, api]);

  const candidates = full?.authorized_pickups ?? [];

  async function submit(override: boolean) {
    if (!name.trim()) return;
    setBusy(true);
    setDenied(null);
    try {
      await api.run("attendance.check_out", {
        event_id: crypto.randomUUID(),
        child_id: child.id,
        room_id: roomId,
        at: new Date().toISOString(),
        collector_name: name.trim(),
        ...(override ? { pickup_override: true } : {}),
      });
      onDone(child.name);
    } catch (e) {
      setDenied((e as Error).message || t("attendance.pickup_denied"));
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="fixed inset-0 z-50 flex flex-col justify-end" role="dialog" aria-modal="true">
      <button className="absolute inset-0 bg-foreground/40 backdrop-blur-sm" aria-label={t("common.cancel")} onClick={onClose} />
      <div className="relative max-h-[85vh] overflow-y-auto rounded-t-3xl border-t border-border bg-card p-5 pb-8 shadow-2xl motion-safe:animate-in motion-safe:slide-in-from-bottom">
        <div className="mx-auto mb-4 h-1.5 w-10 rounded-full bg-muted" aria-hidden />
        <div className="mb-4 flex items-start justify-between gap-3">
          <h2 className="text-xl font-bold tracking-tight text-foreground">{t("attendance.who_collecting", { name: child.name })}</h2>
          <Button variant="ghost" size="icon" onClick={onClose} aria-label={t("common.cancel")}><X /></Button>
        </div>

        {denied && (
          <div className="mb-4 rounded-2xl border border-destructive bg-destructive/10 p-4">
            <div className="flex items-center gap-2 text-destructive">
              <ShieldAlert className="size-5 shrink-0" aria-hidden />
              <p className="text-base font-bold">{t("attendance.pickup_denied")}</p>
            </div>
            <p className="mt-1.5 text-sm font-medium text-destructive">{denied}</p>
            {isAdmin && (
              <Button
                variant="destructive"
                className="mt-3 w-full"
                disabled={busy}
                onClick={() => submit(true)}
              >
                {t("attendance.override_confirm")}
              </Button>
            )}
          </div>
        )}

        <label className="mb-1.5 block text-sm font-medium text-foreground" htmlFor="collector">{t("attendance.collector_name")}</label>
        <Input
          id="collector"
          value={name}
          onChange={(e) => setName(e.target.value)}
          placeholder={t("attendance.collector_placeholder")}
          className="h-12 text-base"
        />

        {full === null ? (
          <div className="mt-3 h-12 animate-pulse rounded-xl bg-muted" />
        ) : candidates.length > 0 ? (
          <ul className="mt-3 space-y-2">
            {candidates.map((p, i) => {
              const active = name.trim() === p.name;
              return (
                <li key={`${p.name}-${i}`}>
                  <button
                    type="button"
                    onClick={() => setName(p.name)}
                    className={
                      "flex w-full items-center justify-between gap-3 rounded-2xl border p-4 text-left transition-colors " +
                      (active ? "border-primary bg-accent" : "border-border bg-card hover:bg-accent")
                    }
                  >
                    <span className="min-w-0">
                      <span className="block truncate text-base font-semibold text-foreground">{p.name}</span>
                      <span className="block text-xs text-muted-foreground">{t("attendance.on_pickup_list")}</span>
                    </span>
                    {active && <Check className="size-5 shrink-0 text-primary" aria-hidden />}
                  </button>
                </li>
              );
            })}
          </ul>
        ) : null}

        <Button
          className="mt-5 w-full text-base"
          size="lg"
          disabled={busy || !name.trim()}
          onClick={() => submit(false)}
        >
          {t("attendance.confirm_pickup")}
        </Button>
      </div>
    </div>
  );
}
