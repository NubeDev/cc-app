import { Utensils, Moon, Baby, Blocks, Camera, StickyNote, TriangleAlert, Pill } from "lucide-react";
import type { LucideIcon } from "lucide-react";
import { useT } from "../../hooks/useT";
import type { LogKind } from "./shared";
import { LOG_KINDS } from "./shared";

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

// The day-rollup header (from care.log.day's sparse `summary` tally): a compact
// chip row of the day's counts. Only non-zero kinds render (the tally is
// sparse). Incident/medication chips carry the destructive token so a parent
// scanning the top of the feed sees a flagged day at a glance.
export function DaySummary({ summary }: { summary: Record<string, number> }) {
  const t = useT();
  const chips = LOG_KINDS.filter((k) => (summary[k] ?? 0) > 0);

  if (!chips.length) {
    return (
      <p className="px-4 pb-2 text-[13px] text-muted-foreground">{t("feed.no_summary")}</p>
    );
  }

  return (
    <div className="flex flex-wrap gap-2 px-4 pb-3">
      {chips.map((k) => {
        const Icon = ICONS[k];
        const emphatic = k === "incident" || k === "medication";
        return (
          <span
            key={k}
            className={
              "inline-flex items-center gap-1.5 rounded-full px-3 py-1 text-[13px] font-medium " +
              (emphatic
                ? "bg-destructive/10 text-destructive"
                : "bg-muted text-foreground")
            }
          >
            <Icon className="size-3.5" aria-hidden />
            {t("log.type." + k)}
            <span className="tabular-nums opacity-70">{summary[k]}</span>
          </span>
        );
      })}
    </div>
  );
}
