import { useEffect, useState } from "react";
import { Plus, TriangleAlert, ChevronRight } from "lucide-react";
import { useCareApi } from "../../api/care";
import { LargeTitle } from "../../components/LargeTitle";
import { Field } from "../../components/Field";
import { Button } from "../../components/ui/button";
import { Input } from "../../components/ui/input";
import { Textarea } from "../../components/ui/textarea";
import { Switch } from "../../components/ui/switch";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "../../components/ui/select";
import { useT } from "../../hooks/useT";

export interface ChildRow {
  id: string;
  name: string;
  dob: string;
  room_id?: string;
  allergies?: string[];
  medical_notes?: string;
  immunizations?: string[];
  emergency_contacts?: Array<{ name: string; phone: string; relationship?: string }>;
  authorized_pickups?: Array<{ name: string; phone?: string }>;
  photo_consent?: boolean;
  archived?: boolean;
}
export interface RoomRow { id: string; name: string; }

export function ChildrenListPage({ embedded }: { embedded?: boolean } = {}) {
  const t = useT();
  const api = useCareApi();
  const [children, setChildren] = useState<ChildRow[] | null>(null);
  const [rooms, setRooms] = useState<RoomRow[]>([]);
  const [editing, setEditing] = useState<ChildRow | null>(null);
  const [creating, setCreating] = useState(false);

  async function refresh() {
    const [c, r] = await Promise.all([api.list<ChildRow>("child"), api.list<RoomRow>("room")]);
    setChildren(c.filter((x) => !x.archived));
    setRooms(r);
  }
  useEffect(() => { refresh().catch(() => {}); }, []);

  if (editing || creating) {
    return <ChildEditor initial={editing} rooms={rooms} onDone={() => { setEditing(null); setCreating(false); refresh(); }} />;
  }

  return (
    <main className={embedded ? "" : "pb-24"}>
      {!embedded && (
        <LargeTitle
          trailing={
            <Button size="pill" onClick={() => setCreating(true)}>
              <Plus /> {t("common.add")}
            </Button>
          }
        >
          {t("nav.children")}
        </LargeTitle>
      )}
      <div className="px-4">
        {embedded && (
          <Button className="mb-4 w-full" onClick={() => setCreating(true)}>
            <Plus /> {t("child.editor.title.new")}
          </Button>
        )}
        {children === null ? (
          <ul className="space-y-2" aria-busy="true">
            {[0, 1, 2].map((i) => <li key={i} className="h-[70px] animate-pulse rounded-2xl bg-muted" />)}
          </ul>
        ) : !children.length ? (
          <p className="py-16 text-center text-[15px] text-muted-foreground">{t("child.empty")}</p>
        ) : (
          <ul className="space-y-2">
            {children.map((c) => {
              const hasAllergies = c.allergies && c.allergies.length > 0;
              return (
                <li key={c.id}>
                  <button
                    onClick={() => setEditing(c)}
                    className="flex w-full items-center gap-3 rounded-2xl border border-border bg-card p-4 text-left shadow-sm transition-colors hover:bg-accent focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                  >
                    <span className="min-w-0 flex-1">
                      <span className="block truncate text-base font-semibold text-foreground">{c.name}</span>
                      <span className="block text-[13px] text-muted-foreground">{c.dob}</span>
                    </span>
                    {hasAllergies && (
                      <span className="inline-flex items-center gap-1 rounded-full bg-destructive/10 px-2.5 py-1 text-xs font-semibold text-destructive">
                        <TriangleAlert className="size-3.5" aria-hidden /> {c.allergies!.length}
                      </span>
                    )}
                    <ChevronRight className="size-5 shrink-0 text-muted-foreground" aria-hidden />
                  </button>
                </li>
              );
            })}
          </ul>
        )}
      </div>
    </main>
  );
}

function ChildEditor({ initial, rooms, onDone }: { initial: ChildRow | null; rooms: RoomRow[]; onDone: () => void }) {
  const t = useT();
  const api = useCareApi();
  const [name, setName] = useState(initial?.name ?? "");
  const [dob, setDob] = useState(initial?.dob ?? "");
  const [roomId, setRoomId] = useState(initial?.room_id ?? "");
  const [allergies, setAllergies] = useState((initial?.allergies ?? []).join(", "));
  const [medicalNotes, setMedicalNotes] = useState(initial?.medical_notes ?? "");
  const [photoConsent, setPhotoConsent] = useState(initial?.photo_consent ?? false);
  const [busy, setBusy] = useState(false);
  const [err, setErr] = useState<string | null>(null);

  async function save() {
    setBusy(true);
    setErr(null);
    try {
      const id = initial?.id ?? (name.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "") || "child");
      const allergyList = allergies.split(",").map((s) => s.trim()).filter(Boolean);
      const payload = {
        id, name, dob,
        room_id: roomId || undefined,
        allergies: allergyList,
        medical_notes: medicalNotes || undefined,
        photo_consent: photoConsent,
      };
      await api.run(initial ? "child.update" : "child.create", payload);
      onDone();
    } catch (e) {
      setErr((e as Error).message ?? t("common.error_generic"));
    } finally { setBusy(false); }
  }

  return (
    <main className="pb-24">
      <LargeTitle>{initial ? t("child.editor.title.edit") : t("child.editor.title.new")}</LargeTitle>
      <form onSubmit={(e) => { e.preventDefault(); save(); }} className="space-y-6 px-4 pt-1">
        <div className="space-y-4">
          <Field label={t("child.name")} htmlFor="c-name" required>
            <Input id="c-name" value={name} onChange={(e) => setName(e.target.value)} required />
          </Field>
          <Field label={t("child.dob")} htmlFor="c-dob" required>
            <Input id="c-dob" type="date" value={dob} onChange={(e) => setDob(e.target.value)} required />
          </Field>
          <Field label={t("child.room")}>
            <Select value={roomId || "none"} onValueChange={(v) => setRoomId(v === "none" ? "" : v)}>
              <SelectTrigger><SelectValue placeholder="—" /></SelectTrigger>
              <SelectContent>
                <SelectItem value="none">—</SelectItem>
                {rooms.map((r) => <SelectItem key={r.id} value={r.id}>{r.name}</SelectItem>)}
              </SelectContent>
            </Select>
          </Field>
        </div>

        <Section title={t("child.editor.safety")} hint={t("child.editor.safety_help")}>
          <Field label={t("child.allergies")} htmlFor="c-allergies" required hint={t("child.required.allergies_hint")}>
            <Input id="c-allergies" value={allergies} onChange={(e) => setAllergies(e.target.value)} placeholder="peanuts, dairy" />
          </Field>
          <Field label={t("child.medical_notes")} htmlFor="c-medical">
            <Textarea id="c-medical" value={medicalNotes} onChange={(e) => setMedicalNotes(e.target.value)} rows={3} />
          </Field>
        </Section>

        <Section title={t("child.editor.consent")}>
          <label className="flex items-center justify-between gap-3 rounded-2xl border border-border bg-card p-4 shadow-sm">
            <span className="text-sm text-foreground">{t("child.photo_consent")}</span>
            <Switch checked={photoConsent} onCheckedChange={setPhotoConsent} />
          </label>
        </Section>

        {err && <p role="alert" className="text-sm text-destructive">{err}</p>}
        <div className="flex gap-2 pt-2">
          <Button type="button" variant="outline" className="flex-1" onClick={onDone}>{t("common.cancel")}</Button>
          <Button type="submit" className="flex-1" disabled={busy}>{t("common.save")}</Button>
        </div>
      </form>
    </main>
  );
}

function Section({ title, hint, children }: { title: string; hint?: string; children: React.ReactNode }) {
  return (
    <section className="space-y-3">
      <div>
        <h2 className="text-xs font-semibold uppercase tracking-wide text-muted-foreground">{title}</h2>
        {hint && <p className="pt-1 text-xs leading-relaxed text-muted-foreground">{hint}</p>}
      </div>
      {children}
    </section>
  );
}
