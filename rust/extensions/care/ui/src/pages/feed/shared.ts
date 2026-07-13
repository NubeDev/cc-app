// Shared shapes + helpers for the daily-feed surfaces (guardian FeedPage +
// staff LogEntryPage + the feed widgets). Mirrors the Rust `daily_log` schema
// (care/src/log/records.rs + payload.rs) — the ONE place the wire shape lives
// for the UI, so a field rename lands in one file.

/** The eight entry types (LogKind) — the stable enum keys the record + catalog
 *  share; each maps to a `log.type.<kind>` catalog key. */
export type LogKind =
  | "meal"
  | "nap"
  | "diaper"
  | "activity"
  | "photo"
  | "note"
  | "incident"
  | "medication";

/** The four meal-portion enum keys (`log.portion.<key>`). */
export type Portion = "all" | "most" | "some" | "none";

/** The meal-slot keys the feed shares with the menu (`slot.<key>`). */
export type MealSlot = "breakfast" | "am_snack" | "lunch" | "pm_snack";

export interface MealPayload {
  slot: string;
  portion: string;
}
export interface NapPayload {
  start?: string;
  end?: string;
}
export interface IncidentPayload {
  what: string;
  where: string;
  action: string;
  acknowledged?: boolean;
}
export interface MedicationPayload {
  dose: string;
  witness: string;
}

/** One `daily_log` entry, as returned by `care.log.list` / `care.log.day`.
 *  Always for exactly one child (per-child rows). */
export interface DailyLog {
  kind: LogKind;
  child_id: string;
  room_id: string;
  author: string;
  at: string;
  note?: string;
  media_ids?: string[];
  nap?: NapPayload;
  meal?: MealPayload;
  incident?: IncidentPayload;
  medication?: MedicationPayload;
  correction_of?: string;
}

/** `care.log.list` reply — cursor-paged. */
export interface LogListReply {
  entries: DailyLog[];
  next_cursor?: string;
}

/** `care.log.day` reply — single-child rollup with a sparse per-kind tally. */
export interface LogDayReply {
  child_id: string;
  date: string;
  entries: DailyLog[];
  summary: Record<string, number>;
}

/** The type-order shown in the staff picker + the summary chips. Incident +
 *  medication last (they open extra fields; the high-frequency taps sit first). */
export const LOG_KINDS: LogKind[] = [
  "meal",
  "nap",
  "diaper",
  "activity",
  "photo",
  "note",
  "incident",
  "medication",
];

export const PORTIONS: Portion[] = ["all", "most", "some", "none"];
export const MEAL_SLOTS: MealSlot[] = ["breakfast", "am_snack", "lunch", "pm_snack"];

/** Today's date as `YYYY-MM-DD` (the `log.day` date key). Local wall date. */
export function isoDay(d: Date = new Date()): string {
  const y = d.getFullYear();
  const m = String(d.getMonth() + 1).padStart(2, "0");
  const day = String(d.getDate()).padStart(2, "0");
  return [y, m, day].join("-");
}

/** Start-of-today ISO instant — the `since` bound for "today's" feed. */
export function todayStartIso(): string {
  const d = new Date();
  d.setHours(0, 0, 0, 0);
  return d.toISOString();
}

/** A short local time (e.g. "11:30 AM") for an ISO instant — the feed timestamp. */
export function shortTime(iso: string): string {
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return iso;
  return d.toLocaleTimeString(undefined, { hour: "numeric", minute: "2-digit" });
}

/** The relative media serve path for a photo `media_id` — the lb media route,
 *  reach-checked SERVER-side (never at render). Kept relative so the browser
 *  resolves it against the host origin the shell already runs on. */
export function thumbSrc(mediaId: string): string {
  return "/media/" + encodeURIComponent(mediaId) + "?variant=thumb";
}

/** A stable de-dupe key for an entry (no id in the body). `(at, kind, child)`
 *  plus the correction target is unique enough to merge live-poll pages. */
export function entryKey(e: DailyLog): string {
  return [e.at, e.kind, e.child_id, e.correction_of ?? ""].join("|");
}

/** Merge a freshly-polled page into the known set, newest-first, de-duped.
 *  Corrections supersede their target only inside `log.day` server-side; the
 *  live list shows the raw ledger, so we simply de-dupe by `entryKey`. */
export function mergeEntries(prev: DailyLog[], incoming: DailyLog[]): DailyLog[] {
  const seen = new Set(prev.map(entryKey));
  const merged = [...prev];
  for (const e of incoming) {
    const k = entryKey(e);
    if (!seen.has(k)) {
      seen.add(k);
      merged.push(e);
    }
  }
  // Newest first (iOS feed convention) — ISO `at` sorts lexically = chronologically.
  merged.sort((a, b) => (a.at < b.at ? 1 : a.at > b.at ? -1 : 0));
  return merged;
}
