import { useEffect, useMemo, useState } from "react";
import {
  Utensils,
  Moon,
  Baby,
  Blocks,
  Camera,
  StickyNote,
  TriangleAlert,
  Pill,
  Check,
} from "lucide-react";
import type { LucideIcon } from "lucide-react";
import { useCareApi } from "../../api/care";
import { LargeTitle } from "../../components/LargeTitle";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "../../components/ui/select";
import { useT } from "../../hooks/useT";
import type { ChildRow, RoomRow } from "../child/ChildrenListPage";
import { LogPayloadSheet } from "./LogPayloadSheet";
import { LOG_KINDS, type LogKind } from "./shared";

const ICONS: Record<LogKind, LucideIcon> = {
  meal: Utensils,
  nap: Moon,
  diaper: Baby,
  activity: Blocks,
  photo: Camera,
  note: StickyNote,
  incident: TriangleAlert,
  medication: Pill,
};

// STAFF two-tap logging (the highest-frequency staff flow): pick a room, TAP the
// children (multi-select), TAP a type. Simple types (nap/diaper/activity/note)
// fire on the type tap — that's the two-tap path. Types with required/optional
// payload (meal/incident/medication/photo) open a bottom sheet to collect it.
export function LogEntryPage({ embedded }: { embedded?: boolean } = {}) {
  const t = useT();
  const api = useCareApi();

  const [rooms, setRooms] = useState<RoomRow[] | null>(null);
  const [roomId, setRoomId] = useState<string>("");
  const [children, setChildren] = useState<ChildRow[] | null>(null);
  const [picked, setPicked] = useState<Set<string>>(new Set());
  const [sheetKind, setSheetKind] = useState<LogKind | null>(null);
  const [toast, setToast] = useState<string | null>(null);

  useEffect(() => {
    api
      .list<RoomRow>("room")
      .then((r) => {
        setRooms(r);
        if (r[0]) setRoomId(r[0].id);
      })
      .catch(() => setRooms([]));
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  useEffect(() => {
    if (!roomId) return;
    setChildren(null);
    setPicked(new Set());
    api
      .list<ChildRow>("child")
      .then((c) => setChildren(c.filter((x) => !x.archived && x.room_id === roomId)))
      .catch(() => setChildren([]));
  }, [api, roomId]);

  const pickedIds = useMemo(() => [...picked], [picked]);

  function toggle(id: string) {
    setPicked((s) => {
      const next = new Set(s);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  }

  function flash(msg: string) {
    setToast(msg);
    setTimeout(() => setToast((cur) => (cur === msg ? null : cur)), 2500);
  }

  // The type tap. Simple types log immediately (two taps done); typed payloads
  // open the sheet.
  const NEEDS_SHEET: LogKind[] = ["meal", "incident", "medication", "photo"];
  function onKind(kind: LogKind) {
    if (!pickedIds.length) return;
    if (NEEDS_SHEET.includes(kind)) {
      setSheetKind(kind);
      return;
    }
    logEntry(kind, {}).catch((e) => flash((e as Error).message));
  }

  async function logEntry(kind: LogKind, payload: Record<string, unknown>) {
    await api.run("log.add", {
      entry_id: crypto.randomUUID(),
      child_ids: pickedIds,
      room_id: roomId,
      kind,
      at: new Date().toISOString(),
      ...payload,
    });
    flash(t("log.saved", { count: pickedIds.length, type: t("log.type." + kind) }));
    setPicked(new Set());
    setSheetKind(null);
  }

  return (
    <main className={embedded ? "" : "pb-24"}>
      {!embedded && <LargeTitle>{t("log.add.title")}</LargeTitle>}

      <div className="px-4">
        {rooms === null ? (
          <div className="h-11 animate-pulse rounded-xl bg-muted" />
        ) : !rooms.length ? (
          <p className="py-16 text-center text-[15px] text-muted-foreground">
            {t("attendance.no_rooms")}
          </p>
        ) : (
          <Select value={roomId} onValueChange={setRoomId}>
            <SelectTrigger className="h-12 text-base">
              <SelectValue placeholder={t("attendance.select_room")} />
            </SelectTrigger>
            <SelectContent>
              {rooms.map((r) => (
                <SelectItem key={r.id} value={r.id}>
                  {r.name}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        )}
      </div>

      {rooms?.length ? (
        <>
          <p className="px-4 pb-1 pt-3 text-[13px] text-muted-foreground">
            {pickedIds.length
              ? t("log.selected_count", { count: pickedIds.length })
              : t("log.select_children")}
          </p>
          <ChildGrid children={children} picked={picked} onToggle={toggle} />
          <KindBar disabled={!pickedIds.length} onPick={onKind} />
        </>
      ) : null}

      {toast && (
        <div className="pointer-events-none fixed inset-x-0 bottom-24 z-40 flex justify-center px-4">
          <div className="rounded-full bg-success px-5 py-2.5 text-sm font-semibold text-success-foreground shadow-lg">
            {toast}
          </div>
        </div>
      )}

      {sheetKind && (
        <LogPayloadSheet
          kind={sheetKind}
          count={pickedIds.length}
          onClose={() => setSheetKind(null)}
          onSubmit={(payload) => logEntry(sheetKind, payload)}
        />
      )}
    </main>
  );
}

function ChildGrid({
  children,
  picked,
  onToggle,
}: {
  children: ChildRow[] | null;
  picked: Set<string>;
  onToggle: (id: string) => void;
}) {
  const t = useT();
  if (children === null) {
    return (
      <div className="grid grid-cols-2 gap-2 px-4 pt-1 sm:grid-cols-3" aria-busy="true">
        {[0, 1, 2, 3].map((i) => (
          <div key={i} className="h-16 animate-pulse rounded-2xl bg-muted" />
        ))}
      </div>
    );
  }
  if (!children.length) {
    return (
      <p className="px-4 py-16 text-center text-[15px] text-muted-foreground">
        {t("attendance.empty_room")}
      </p>
    );
  }
  return (
    <div className="grid grid-cols-2 gap-2 px-4 pt-1 sm:grid-cols-3">
      {children.map((c) => {
        const on = picked.has(c.id);
        return (
          <button
            key={c.id}
            type="button"
            aria-pressed={on}
            onClick={() => onToggle(c.id)}
            className={
              "flex min-h-16 items-center gap-2 rounded-2xl border p-3 text-left transition-colors active:scale-[0.99] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring " +
              (on ? "border-primary bg-accent" : "border-border bg-card hover:bg-accent")
            }
          >
            <span
              className={
                "flex size-6 shrink-0 items-center justify-center rounded-full border " +
                (on ? "border-primary bg-primary text-primary-foreground" : "border-border")
              }
              aria-hidden
            >
              {on && <Check className="size-4" />}
            </span>
            <span className="min-w-0 truncate text-[15px] font-semibold text-foreground">
              {c.name}
            </span>
          </button>
        );
      })}
    </div>
  );
}

// The type row — a fixed, translucent bar above the tab bar so a tap is always
// in the thumb zone (the second of the two taps).
function KindBar({ disabled, onPick }: { disabled: boolean; onPick: (k: LogKind) => void }) {
  const t = useT();
  return (
    <nav className="fixed inset-x-0 bottom-[calc(52px+env(safe-area-inset-bottom))] z-10 border-t border-border/70 bg-background/80 backdrop-blur-xl">
      <div className="mx-auto flex max-w-2xl gap-1 overflow-x-auto px-3 py-2">
        {LOG_KINDS.map((k) => {
          const Icon = ICONS[k];
          return (
            <button
              key={k}
              type="button"
              disabled={disabled}
              onClick={() => onPick(k)}
              className="flex min-w-[64px] flex-col items-center gap-1 rounded-xl px-2 py-1.5 text-[11px] font-medium text-muted-foreground transition-colors hover:bg-accent hover:text-foreground disabled:opacity-40 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-ring"
            >
              <Icon className="size-6" aria-hidden />
              {t("log.type." + k)}
            </button>
          );
        })}
      </div>
    </nav>
  );
}
