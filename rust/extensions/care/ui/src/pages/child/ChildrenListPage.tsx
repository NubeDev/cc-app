import { useEffect, useState } from "react";
import { useCareApi } from "../../api/care";
import { PageTitle } from "../../components/PageTitle";
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

export function ChildrenListPage() {
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
    <main className="pb-24">
      <PageTitle>{t("nav.children")}</PageTitle>
      <div className="px-4">
        <button onClick={() => setCreating(true)} className="mb-4 w-full rounded-2xl bg-primary px-4 py-3 font-medium text-primary-foreground">
          + {t("child.editor.title.new")}
        </button>
        {!children?.length ? (
          <p className="py-6 text-center text-sm opacity-60">{t("child.empty")}</p>
        ) : (
          <ul className="space-y-2">
            {children.map((c) => (
              <li key={c.id}>
                <button
                  onClick={() => setEditing(c)}
                  className="flex w-full items-start justify-between rounded-2xl border border-border bg-card p-4 text-left"
                >
                  <span>
                    <span className="block text-base font-semibold">{c.name}</span>
                    <span className="block text-xs opacity-60">{c.dob}</span>
                  </span>
                  {c.allergies && c.allergies.length > 0 ? (
                    <span className="rounded-full bg-destructive/10 px-2.5 py-1 text-xs font-medium text-destructive">
                      ⚠ {c.allergies.length}
                    </span>
                  ) : (
                    <span className="rounded-full bg-muted px-2.5 py-1 text-xs opacity-60">{t("common.edit")}</span>
                  )}
                </button>
              </li>
            ))}
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
      const id = initial?.id ?? name.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "") || "child";
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
      <PageTitle>{initial ? t("child.editor.title.edit") : t("child.editor.title.new")}</PageTitle>
      <form onSubmit={(e) => { e.preventDefault(); save(); }} className="space-y-5 px-4">
        <Section title={t("child.name")}>
          <Field label={t("child.name")} required>
            <input value={name} onChange={(e) => setName(e.target.value)} required className="w-full rounded-xl border border-border bg-card px-3 py-2.5" />
          </Field>
          <Field label={t("child.dob")} required>
            <input type="date" value={dob} onChange={(e) => setDob(e.target.value)} required className="w-full rounded-xl border border-border bg-card px-3 py-2.5" />
          </Field>
          <Field label={t("child.room")}>
            <select value={roomId} onChange={(e) => setRoomId(e.target.value)} className="w-full rounded-xl border border-border bg-card px-3 py-2.5">
              <option value="">—</option>
              {rooms.map((r) => <option key={r.id} value={r.id}>{r.name}</option>)}
            </select>
          </Field>
        </Section>

        <Section title={t("child.editor.safety")} hint={t("child.editor.safety_help")}>
          <Field label={t("child.allergies")} required hint={t("child.required.allergies_hint")}>
            <input value={allergies} onChange={(e) => setAllergies(e.target.value)} placeholder="peanuts, dairy" className="w-full rounded-xl border border-border bg-card px-3 py-2.5" />
          </Field>
          <Field label={t("child.medical_notes")}>
            <textarea value={medicalNotes} onChange={(e) => setMedicalNotes(e.target.value)} rows={3} className="w-full rounded-xl border border-border bg-card px-3 py-2.5" />
          </Field>
        </Section>

        <Section title={t("child.editor.consent")}>
          <label className="flex items-start gap-3">
            <input type="checkbox" checked={photoConsent} onChange={(e) => setPhotoConsent(e.target.checked)} className="mt-1 h-5 w-5 rounded" />
            <span className="text-sm">{t("child.photo_consent")}</span>
          </label>
        </Section>

        {err && <p className="text-sm text-destructive">{err}</p>}
        <div className="flex gap-2 pt-4">
          <button type="button" onClick={onDone} className="flex-1 rounded-xl border border-border px-4 py-3">{t("common.cancel")}</button>
          <button type="submit" disabled={busy} className="flex-1 rounded-xl bg-primary px-4 py-3 font-medium text-primary-foreground disabled:opacity-50">{t("common.save")}</button>
        </div>
      </form>
    </main>
  );
}

function Section({ title, hint, children }: { title: string; hint?: string; children: React.ReactNode }) {
  return (
    <section>
      <h2 className="pb-1 text-sm font-semibold uppercase tracking-wide text-muted-foreground">{title}</h2>
      {hint && <p className="pb-3 text-xs text-muted-foreground">{hint}</p>}
      <div className="space-y-3">{children}</div>
    </section>
  );
}

function Field({ label, required, hint, children }: { label: string; required?: boolean; hint?: string; children: React.ReactNode }) {
  return (
    <label className="block">
      <span className="block pb-1 text-sm text-foreground">
        {label}{required && <span className="ml-1 text-destructive">*</span>}
        {hint && <span className="ml-2 text-xs text-muted-foreground">{hint}</span>}
      </span>
      {children}
    </label>
  );
}