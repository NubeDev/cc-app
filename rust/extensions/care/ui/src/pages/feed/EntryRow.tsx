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
import { Button } from "../../components/ui/button";
import { useT } from "../../hooks/useT";
import { shortTime, thumbSrc, type DailyLog, type LogKind } from "./shared";

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

// One typed feed row — the photo/incident dominant, others compact (DESIGN.md
// §Components: "Feed entries are typed rows/cards with the photo dominant").
// Incident + medication stand out via the semantic destructive token; the rest
// sit on the calm card surface. Semantic tokens ONLY (rule 9).
export function EntryRow({
  entry,
  acknowledged,
  onAcknowledge,
}: {
  entry: DailyLog;
  acknowledged?: boolean;
  onAcknowledge?: () => void;
}) {
  const t = useT();
  const Icon = ICONS[entry.kind] ?? StickyNote;
  const isIncident = entry.kind === "incident";
  const emphatic = isIncident || entry.kind === "medication";

  return (
    <article
      className={
        "rounded-2xl border p-4 shadow-sm " +
        (emphatic ? "border-destructive/40 bg-destructive/5" : "border-border bg-card")
      }
    >
      <header className="flex items-center gap-3">
        <span
          className={
            "flex size-9 shrink-0 items-center justify-center rounded-full " +
            (emphatic ? "bg-destructive/15 text-destructive" : "bg-muted text-muted-foreground")
          }
          aria-hidden
        >
          <Icon className="size-5" />
        </span>
        <div className="min-w-0 flex-1">
          <p
            className={
              "text-[15px] font-semibold " +
              (emphatic ? "text-destructive" : "text-foreground")
            }
          >
            {t("log.type." + entry.kind)}
          </p>
        </div>
        <time className="shrink-0 text-xs tabular-nums text-muted-foreground">
          {shortTime(entry.at)}
        </time>
      </header>

      <EntryBody entry={entry} />

      {isIncident && (
        <IncidentActions
          entry={entry}
          acknowledged={acknowledged ?? entry.incident?.acknowledged ?? false}
          onAcknowledge={onAcknowledge}
        />
      )}
    </article>
  );
}

function EntryBody({ entry }: { entry: DailyLog }) {
  const t = useT();
  return (
    <div className="space-y-2 pt-2">
      {entry.kind === "meal" && entry.meal && (
        <p className="text-[15px] text-foreground">
          <span className="font-medium">{t("slot." + entry.meal.slot)}</span>
          {" · "}
          <span className="text-muted-foreground">{t("log.portion." + entry.meal.portion)}</span>
        </p>
      )}

      {entry.kind === "nap" && entry.nap && (
        <p className="text-[15px] text-foreground">
          {entry.nap.start ? shortTime(entry.nap.start) : t("log.nap.open")}
          {" – "}
          {entry.nap.end ? shortTime(entry.nap.end) : t("log.nap.ongoing")}
        </p>
      )}

      {entry.kind === "incident" && entry.incident && (
        <dl className="space-y-1.5 text-[15px]">
          <IncidentField label={t("log.incident.what")} value={entry.incident.what} />
          <IncidentField label={t("log.incident.where")} value={entry.incident.where} />
          <IncidentField label={t("log.incident.action")} value={entry.incident.action} />
        </dl>
      )}

      {entry.kind === "medication" && entry.medication && (
        <p className="text-[15px] text-foreground">
          <span className="font-medium">{entry.medication.dose}</span>
          {" · "}
          <span className="text-muted-foreground">
            {t("log.medication.witness_by", { name: entry.medication.witness })}
          </span>
        </p>
      )}

      {entry.note && <p className="text-[15px] text-foreground">{entry.note}</p>}

      {(entry.media_ids ?? []).length > 0 && (
        <div className="grid grid-cols-2 gap-2 pt-1 sm:grid-cols-3">
          {(entry.media_ids ?? []).map((id) => (
            <img
              key={id}
              src={thumbSrc(id)}
              alt={t("log.photo.alt")}
              loading="lazy"
              className="aspect-square w-full rounded-xl border border-border bg-muted object-cover"
            />
          ))}
        </div>
      )}
    </div>
  );
}

function IncidentField({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex gap-2">
      <dt className="shrink-0 text-xs font-semibold uppercase tracking-wide text-muted-foreground">
        {label}
      </dt>
      <dd className="text-foreground">{value}</dd>
    </div>
  );
}

function IncidentActions({
  acknowledged,
  onAcknowledge,
}: {
  entry: DailyLog;
  acknowledged: boolean;
  onAcknowledge?: () => void;
}) {
  const t = useT();
  if (acknowledged) {
    return (
      <p className="mt-3 flex items-center gap-1.5 text-[13px] font-medium text-success">
        <Check className="size-4" aria-hidden />
        {t("log.incident.acknowledged")}
      </p>
    );
  }
  if (!onAcknowledge) return null;
  return (
    <Button variant="destructive" size="sm" className="mt-3 w-full" onClick={onAcknowledge}>
      {t("log.incident.acknowledge")}
    </Button>
  );
}
