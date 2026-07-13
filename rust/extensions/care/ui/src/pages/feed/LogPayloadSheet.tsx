import { useState } from "react";
import { X } from "lucide-react";
import { Button } from "../../components/ui/button";
import { Input } from "../../components/ui/input";
import { Textarea } from "../../components/ui/textarea";
import { Field } from "../../components/Field";
import { Segmented } from "../../components/ui/segmented";
import { useT } from "../../hooks/useT";
import { MEAL_SLOTS, PORTIONS, type LogKind, type MealSlot, type Portion } from "./shared";

// The payload-collection sheet for the four typed logs. Opens from the staff
// LogEntryPage after children + a type are picked. A bottom sheet on phones
// (DESIGN.md §Layout: "secondary tasks open as bottom sheets"). Photo v1
// collects media-id references entered by the (already-uploaded) media path;
// the upload flow itself is the lb media path (out of this surface's scope).
export function LogPayloadSheet({
  kind,
  count,
  onClose,
  onSubmit,
}: {
  kind: LogKind;
  count: number;
  onClose: () => void;
  onSubmit: (payload: Record<string, unknown>) => Promise<void>;
}) {
  const t = useT();
  const [busy, setBusy] = useState(false);
  const [err, setErr] = useState<string | null>(null);

  const [slot, setSlot] = useState<MealSlot>("lunch");
  const [portion, setPortion] = useState<Portion>("most");
  const [note, setNote] = useState("");
  const [what, setWhat] = useState("");
  const [where, setWhere] = useState("");
  const [action, setAction] = useState("");
  const [dose, setDose] = useState("");
  const [witness, setWitness] = useState("");
  const [mediaIds, setMediaIds] = useState("");

  function build(): Record<string, unknown> | null {
    switch (kind) {
      case "meal":
        return { meal: { slot, portion }, ...(note.trim() ? { note: note.trim() } : {}) };
      case "incident":
        if (!what.trim() || !where.trim() || !action.trim()) return null;
        return {
          incident: { what: what.trim(), where: where.trim(), action: action.trim() },
          ...(note.trim() ? { note: note.trim() } : {}),
        };
      case "medication":
        if (!dose.trim() || !witness.trim()) return null;
        return { medication: { dose: dose.trim(), witness: witness.trim() } };
      case "photo": {
        const ids = mediaIds.split(",").map((s) => s.trim()).filter(Boolean);
        if (!ids.length) return null;
        return { media_ids: ids, ...(note.trim() ? { note: note.trim() } : {}) };
      }
      default:
        return {};
    }
  }

  async function submit() {
    const payload = build();
    if (!payload) {
      setErr(t("log.required_fields"));
      return;
    }
    setBusy(true);
    setErr(null);
    try {
      await onSubmit(payload);
    } catch (e) {
      setErr((e as Error).message || t("common.error_generic"));
      setBusy(false);
    }
  }

  return (
    <div className="fixed inset-0 z-50 flex flex-col justify-end" role="dialog" aria-modal="true">
      <button
        className="absolute inset-0 bg-foreground/40 backdrop-blur-sm"
        aria-label={t("common.cancel")}
        onClick={onClose}
      />
      <div className="relative max-h-[85vh] overflow-y-auto rounded-t-3xl border-t border-border bg-card p-5 pb-8 shadow-2xl motion-safe:animate-in motion-safe:slide-in-from-bottom">
        <div className="mx-auto mb-4 h-1.5 w-10 rounded-full bg-muted" aria-hidden />
        <div className="mb-1 flex items-start justify-between gap-3">
          <h2 className="text-xl font-bold tracking-tight text-foreground">
            {t("log.type." + kind)}
          </h2>
          <Button variant="ghost" size="icon" onClick={onClose} aria-label={t("common.cancel")}>
            <X />
          </Button>
        </div>
        <p className="mb-4 text-[13px] text-muted-foreground">
          {t("log.for_children", { count })}
        </p>

        <div className="space-y-4">
          {kind === "meal" && (
            <>
              <Field label={t("log.meal.slot")}>
                <Segmented<MealSlot>
                  value={slot}
                  onChange={setSlot}
                  columns={2}
                  segments={MEAL_SLOTS.map((s) => ({ value: s, label: t("slot." + s) }))}
                />
              </Field>
              <Field label={t("log.meal.portion")}>
                <Segmented<Portion>
                  value={portion}
                  onChange={setPortion}
                  columns={4}
                  segments={PORTIONS.map((p) => ({ value: p, label: t("log.portion." + p) }))}
                />
              </Field>
            </>
          )}

          {kind === "incident" && (
            <>
              <Field label={t("log.incident.what")} required htmlFor="inc-what">
                <Input id="inc-what" value={what} onChange={(e) => setWhat(e.target.value)} className="h-12 text-base" />
              </Field>
              <Field label={t("log.incident.where")} required htmlFor="inc-where">
                <Input id="inc-where" value={where} onChange={(e) => setWhere(e.target.value)} className="h-12 text-base" />
              </Field>
              <Field label={t("log.incident.action")} required htmlFor="inc-action">
                <Input id="inc-action" value={action} onChange={(e) => setAction(e.target.value)} className="h-12 text-base" />
              </Field>
            </>
          )}

          {kind === "medication" && (
            <>
              <Field label={t("log.medication.dose")} required htmlFor="med-dose">
                <Input id="med-dose" value={dose} onChange={(e) => setDose(e.target.value)} className="h-12 text-base" />
              </Field>
              <Field label={t("log.medication.witness")} required htmlFor="med-witness">
                <Input id="med-witness" value={witness} onChange={(e) => setWitness(e.target.value)} className="h-12 text-base" />
              </Field>
            </>
          )}

          {kind === "photo" && (
            <Field label={t("log.photo.media_ids")} required hint={t("log.photo.media_hint")} htmlFor="photo-ids">
              <Input id="photo-ids" value={mediaIds} onChange={(e) => setMediaIds(e.target.value)} className="h-12 text-base" />
            </Field>
          )}

          {(kind === "meal" || kind === "incident" || kind === "photo") && (
            <Field label={t("log.note")} hint={t("common.optional")} htmlFor="log-note">
              <Textarea id="log-note" value={note} onChange={(e) => setNote(e.target.value)} rows={2} />
            </Field>
          )}
        </div>

        {err && <p className="mt-3 text-sm font-medium text-destructive">{err}</p>}

        <Button className="mt-5 w-full text-base" size="lg" disabled={busy} onClick={submit}>
          {t("log.save")}
        </Button>
      </div>
    </div>
  );
}
