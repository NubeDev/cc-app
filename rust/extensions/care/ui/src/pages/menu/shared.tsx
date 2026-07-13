import { cn } from "../../lib/cn";
import { useT } from "../../hooks/useT";

// The fixed top-9 allergen enum (menus-scope §Intent). `other:<label>` and
// `untaggable` are handled by the label helper below. Semantic tokens only —
// red/destructive is reserved for allergy flags per DESIGN.md.
export const ALLERGENS = [
  "peanut",
  "tree_nut",
  "milk",
  "egg",
  "wheat",
  "soy",
  "fish",
  "shellfish",
  "sesame",
] as const;

export const SLOTS = ["breakfast", "am_snack", "lunch", "pm_snack"] as const;
export type Slot = (typeof SLOTS)[number];

// ---- verb payload shapes (see UI preamble §Verb contracts) ----
export interface MenuItem {
  name: string;
  allergens?: string[];
}
export interface MenuSubstitution {
  restriction: string;
  substitute?: string;
}
export interface Menu {
  date: string;
  room_id: string;
  slot: string;
  items?: MenuItem[];
  substitutions?: MenuSubstitution[];
}
// care.menu.week (guardian) — derived per-child rows
export interface WeekSub {
  item: string;
  reason: string; // allergen key or "untaggable"
  substitute?: string;
  resolved: boolean;
}
export interface WeekSlot {
  slot: string;
  items?: { name: string }[];
  substitutions?: WeekSub[];
}
export interface WeekDay {
  date: string;
  slots?: WeekSlot[];
}
export interface MenuWeek {
  child_id: string;
  room_id?: string;
  days?: WeekDay[];
}

// Render an allergen/reason key. Handles `other:<label>` free-text and the
// `untaggable` sentinel; everything else routes through the catalog.
export function useAllergenLabel(): (key: string) => string {
  const t = useT();
  return (key: string) => {
    if (key.startsWith("other:")) return key.slice("other:".length);
    if (key === "untaggable") return t("allergen.untaggable");
    return t("allergen." + key);
  };
}

// A small allergen pill. `tone="flag"` = the loud red safety flag (item carries
// an allergen); `tone="muted"` = a neutral tag chip in the planner.
export function AllergenBadge({
  label,
  tone = "muted",
  className,
}: {
  label: string;
  tone?: "flag" | "muted";
  className?: string;
}) {
  return (
    <span
      className={cn(
        "inline-flex items-center rounded-full px-2 py-0.5 text-xs font-semibold",
        tone === "flag"
          ? "bg-destructive/10 text-destructive"
          : "bg-muted text-muted-foreground",
        className,
      )}
    >
      {label}
    </span>
  );
}

// A toggle chip for the planner allergen picker (selected = destructive-tinted).
export function AllergenToggle({
  label,
  active,
  onToggle,
}: {
  label: string;
  active: boolean;
  onToggle: () => void;
}) {
  return (
    <button
      type="button"
      aria-pressed={active}
      onClick={onToggle}
      className={cn(
        "rounded-full border px-3 py-1 text-xs font-medium transition-colors",
        "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
        active
          ? "border-destructive/30 bg-destructive/10 text-destructive"
          : "border-border bg-card text-muted-foreground hover:bg-accent",
      )}
    >
      {label}
    </button>
  );
}

// ---- week/date math (local time; ISO yyyy-mm-dd keys) ----
export function isoDate(d: Date): string {
  const y = d.getFullYear();
  const m = String(d.getMonth() + 1).padStart(2, "0");
  const day = String(d.getDate()).padStart(2, "0");
  return `${y}-${m}-${day}`;
}

// Monday of the week containing `d`.
export function mondayOf(d: Date): Date {
  const x = new Date(d.getFullYear(), d.getMonth(), d.getDate());
  const dow = (x.getDay() + 6) % 7; // 0 = Monday
  x.setDate(x.getDate() - dow);
  return x;
}

export function addDays(d: Date, n: number): Date {
  const x = new Date(d);
  x.setDate(x.getDate() + n);
  return x;
}

// Parse an ISO yyyy-mm-dd key into a local Date at midnight.
export function parseIso(key: string): Date {
  const parts = key.split("-").map(Number);
  return new Date(parts[0] ?? 1970, (parts[1] ?? 1) - 1, parts[2] ?? 1);
}

export function weekDates(weekStart: string): string[] {
  const start = parseIso(weekStart);
  return Array.from({ length: 5 }, (_, i) => isoDate(addDays(start, i)));
}

// Client-side unresolved derivation for the planner: an item allergen with no
// matching substitution entry (menus-scope §Safety surface). Conservative — an
// `untaggable`/`other:` item with no substitute also counts as unresolved.
export function unresolvedRestrictions(menu: Menu): string[] {
  const items = menu.items ?? [];
  const subs = menu.substitutions ?? [];
  const filled = new Set(
    subs.filter((s) => (s.substitute ?? "").trim()).map((s) => s.restriction),
  );
  const restrictions = new Set<string>();
  for (const it of items) for (const a of it.allergens ?? []) restrictions.add(a);
  return [...restrictions].filter((r) => !filled.has(r));
}
