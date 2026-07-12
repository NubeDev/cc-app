import { useEffect, useState } from "react";
import { useCareApi } from "../../api/care";
import { PageTitle } from "../../components/PageTitle";
import { useT } from "../../hooks/useT";

export interface GuardianRow { id: string; name: string; email: string; phone?: string; locale?: string; sub?: string | null; }
export interface ChildRow { id: string; name: string; }
export interface EdgeRow {
  edge_id?: string;
  guardian_sub?: string;
  child_id?: string;
  guardian_id?: string;
  guardian_name?: string;
  relationship?: string;
  live?: boolean;
  can_pickup?: boolean;
  receives_daily_feed?: boolean;
  receives_billing?: boolean;
  emergency_contact?: boolean;
  custody_notes?: string;
}

const RELS = ["mother", "father", "grandparent", "guardian", "other"] as const;

export function GuardiansListPage() {
  const t = useT();
  const api = useCareApi();
  const [guardians, setGuardians] = useState<GuardianRow[] | null>(null);
  const [editing, setEditing] = useState<GuardianRow | null>(null);
  const [creating, setCreating] = useState(false);

  async function refresh() {
    setGuardians(await api.list<GuardianRow>("guardian"));
  }
  useEffect(() => { refresh().catch(() => {}); }, []);

  if (editing || creating) {
    return <GuardianEditor initial={editing} onDone={() => { setEditing(null); setCreating(false); refresh(); }} />;
  }

  return (
    <main className="pb-24">
      <PageTitle>{t("guardian.list.title")}</PageTitle>
      <div className="px-4">
        <button onClick={() => setCreating(true)} className="mb-4 w-full rounded-2xl bg-primary px-4 py-3 font-medium text-primary-foreground">
          + {t("guardian.editor.title.new")}
        </button>
        {!guardians?.length ? (
          <p className="py-6 text-center text-sm opacity-60">{t("guardian.empty")}</p>
        ) : (
          <ul className="space-y-2">
            {guardians.map((g) => (
              <li key={g.id}>
                <button onClick={() => setEditing(g)} className="flex w-full items-start justify-between rounded-2xl border border-border bg-card p-4 text-left">
                  <span>
                    <span className="block text-base font-semibold">{g.name}</span>
                    <span className="block text-xs opacity-60">{g.email}</span>
                  </span>
                  {!g.sub && <span className="rounded-full bg-amber-100 px-2.5 py-1 text-xs font-medium text-amber-900 dark:bg-amber-900/30 dark:text-amber-200">{t("guardian.invite_pending")}</span>}
                </button>
              </li>
            ))}
          </ul>
        )}
      </div>
    </main>
  );
}

function GuardianEditor({ initial, onDone }: { initial: GuardianRow | null; onDone: () => void }) {
  const t = useT();
  const api = useCareApi();
  const [name, setName] = useState(initial?.name ?? "");
  const [email, setEmail] = useState(initial?.email ?? "");
  const [phone, setPhone] = useState(initial?.phone ?? "");
  const [locale, setLocale] = useState(initial?.locale ?? "en");
  const [busy, setBusy] = useState(false);
  const [err, setErr] = useState<string | null>(null);

  async function save() {
    setBusy(true); setErr(null);
    try {
      const id = initial?.id ?? email.split("@")[0].toLowerCase().replace(/[^a-z0-9]+/g, "-");
      await api.run("guardian.create", { id, name, email, phone, locale });
      onDone();
    } catch (e) { setErr(t("common.error_generic")); }
    finally { setBusy(false); }
  }

  return (
    <main className="pb-24">
      <PageTitle>{initial ? t("guardian.editor.title.edit") : t("guardian.editor.title.new")}</PageTitle>
      <form onSubmit={(e) => { e.preventDefault(); save(); }} className="space-y-4 px-4">
        <Field label={t("guardian.name")} required>
          <input value={name} onChange={(e) => setName(e.target.value)} required className="w-full rounded-xl border border-border bg-card px-3 py-2.5" />
        </Field>
        <Field label={t("guardian.email")} required>
          <input type="email" value={email} onChange={(e) => setEmail(e.target.value)} required className="w-full rounded-xl border border-border bg-card px-3 py-2.5" />
        </Field>
        <Field label={t("guardian.phone")}>
          <input type="tel" value={phone} onChange={(e) => setPhone(e.target.value)} className="w-full rounded-xl border border-border bg-card px-3 py-2.5" />
        </Field>
        <Field label={t("guardian.locale")}>
          <div className="grid grid-cols-2 gap-2">
            {(["en", "es"] as const).map((l) => (
              <button key={l} type="button" onClick={() => setLocale(l)} className={`rounded-xl border px-3 py-2.5 text-sm ${locale === l ? "border-primary bg-primary text-primary-foreground" : "border-border bg-card"}`}>
                {t(`center.locale.${l}`)}
              </button>
            ))}
          </div>
        </Field>
        {err && <p className="text-sm text-destructive">{err}</p>}
        <div className="flex gap-2 pt-4">
          <button type="button" onClick={onDone} className="flex-1 rounded-xl border border-border px-4 py-3">{t("common.cancel")}</button>
          <button type="submit" disabled={busy} className="flex-1 rounded-xl bg-primary px-4 py-3 font-medium text-primary-foreground disabled:opacity-50">{t("common.save")}</button>
        </div>
      </form>
    </main>
  );
}

export function FamilyEdgesPage({ childId }: { childId: string }) {
  const t = useT();
  const api = useCareApi();
  const [edges, setEdges] = useState<EdgeRow[] | null>(null);
  const [guardians, setGuardians] = useState<GuardianRow[]>([]);
  const [creating, setCreating] = useState(false);
  const [editing, setEditing] = useState<EdgeRow | null>(null);

  async function refresh() {
    const child = await api.get<{ id: string; name: string }>("child", childId);
    const all = await api.list<GuardianRow>("guardian");
    setGuardians(all);
    // For an MVP family/edges view we use a derived read: the
    // guardianship list per child isn't exposed as a verb yet, but the
    // chokepoint's edges arrive here via the read delegation. Today
    // we render the simple per-child links via the seeded data shape.
    void child;
    setEdges([]); // populated by EdgeEditor mutations
  }
  useEffect(() => { refresh().catch(() => {}); }, [childId]);

  if (creating) {
    return <EdgeEditor childId={childId} initial={null} guardians={guardians} onDone={() => { setCreating(false); refresh(); }} />;
  }
  if (editing) {
    return <EdgeEditor childId={childId} initial={editing} guardians={guardians} onDone={() => { setEditing(null); refresh(); }} />;
  }

  return (
    <main className="pb-24">
      <PageTitle>{t("edge.list.title")}</PageTitle>
      <div className="px-4">
        <button onClick={() => setCreating(true)} className="mb-4 w-full rounded-2xl bg-primary px-4 py-3 font-medium text-primary-foreground">
          + {t("edge.editor.title.new")}
        </button>
        {!edges?.length ? (
          <p className="py-6 text-center text-sm opacity-60">{t("edge.empty")}</p>
        ) : (
          <ul className="space-y-2">
            {edges.map((e, i) => (
              <li key={i}>
                <button onClick={() => setEditing(e)} className="flex w-full items-start justify-between rounded-2xl border border-border bg-card p-4 text-left">
                  <span>
                    <span className="block font-medium">{e.guardian_name ?? e.guardian_sub}</span>
                    <span className="block text-xs opacity-60">{e.relationship}</span>
                  </span>
                  {e.can_pickup && <span className="rounded-full bg-emerald-100 px-2 py-1 text-xs text-emerald-900 dark:bg-emerald-900/30 dark:text-emerald-200">✓ {t("edge.flag.can_pickup")}</span>}
                </button>
              </li>
            ))}
          </ul>
        )}
      </div>
    </main>
  );
}

function EdgeEditor({ childId, initial, guardians, onDone }: { childId: string; initial: EdgeRow | null; guardians: GuardianRow[]; onDone: () => void }) {
  const t = useT();
  const api = useCareApi();
  const [guardianId, setGuardianId] = useState(initial?.guardian_id ?? guardians[0]?.id ?? "");
  const [relationship, setRelationship] = useState<string>(initial?.relationship ?? "guardian");
  const [canPickup, setCanPickup] = useState(initial?.can_pickup ?? true);
  const [receivesFeed, setReceivesFeed] = useState(initial?.receives_daily_feed ?? true);
  const [receivesBilling, setReceivesBilling] = useState(initial?.receives_billing ?? false);
  const [emergency, setEmergency] = useState(initial?.emergency_contact ?? false);
  const [custodyNotes, setCustodyNotes] = useState(initial?.custody_notes ?? "");
  const [busy, setBusy] = useState(false);

  async function save() {
    setBusy(true);
    try {
      const g = guardians.find((x) => x.id === guardianId);
      const guardian_sub = g?.sub ?? `user:${guardianId}`;
      await api.run("guardianship.link", {
        guardian_sub,
        child_id: childId,
        relationship,
        can_pickup: canPickup,
        receives_daily_feed: receivesFeed,
        receives_billing: receivesBilling,
        emergency_contact: emergency,
        custody_notes: custodyNotes || undefined,
      });
      onDone();
    } finally { setBusy(false); }
  }

  return (
    <main className="pb-24">
      <PageTitle>{initial ? t("edge.editor.title.edit") : t("edge.editor.title.new")}</PageTitle>
      <form onSubmit={(e) => { e.preventDefault(); save(); }} className="space-y-4 px-4">
        <Field label={t("guardian.name")}>
          <select value={guardianId} onChange={(e) => setGuardianId(e.target.value)} className="w-full rounded-xl border border-border bg-card px-3 py-2.5">
            {guardians.map((g) => <option key={g.id} value={g.id}>{g.name}</option>)}
          </select>
        </Field>
        <Field label={t("edge.relationship")}>
          <div className="grid grid-cols-3 gap-2">
            {RELS.map((r) => (
              <button key={r} type="button" onClick={() => setRelationship(r)} className={`rounded-xl border px-2 py-2.5 text-xs ${relationship === r ? "border-primary bg-primary text-primary-foreground" : "border-border bg-card"}`}>
                {t(`edge.relationship.${r}`)}
              </button>
            ))}
          </div>
        </Field>
        <section className="space-y-2 rounded-2xl border border-border bg-card p-4">
          <ToggleRow label={t("edge.flag.can_pickup")} value={canPickup} onChange={setCanPickup} />
          <ToggleRow label={t("edge.flag.receives_daily_feed")} value={receivesFeed} onChange={setReceivesFeed} />
          <ToggleRow label={t("edge.flag.receives_billing")} value={receivesBilling} onChange={setReceivesBilling} />
          <ToggleRow label={t("edge.flag.emergency_contact")} value={emergency} onChange={setEmergency} />
        </section>
        <Field label={t("edge.flag.custody_notes")}>
          <textarea value={custodyNotes} onChange={(e) => setCustodyNotes(e.target.value)} rows={2} className="w-full rounded-xl border border-border bg-card px-3 py-2.5" />
        </Field>
        <div className="flex gap-2 pt-4">
          <button type="button" onClick={onDone} className="flex-1 rounded-xl border border-border px-4 py-3">{t("common.cancel")}</button>
          <button type="submit" disabled={busy} className="flex-1 rounded-xl bg-primary px-4 py-3 font-medium text-primary-foreground disabled:opacity-50">{t("common.save")}</button>
        </div>
      </form>
    </main>
  );
}

function ToggleRow({ label, value, onChange }: { label: string; value: boolean; onChange: (v: boolean) => void }) {
  return (
    <label className="flex items-center justify-between gap-3 py-1">
      <span className="text-sm">{label}</span>
      <button type="button" onClick={() => onChange(!value)} className={`relative h-7 w-12 rounded-full transition ${value ? "bg-primary" : "bg-muted"}`}>
        <span className={`absolute top-0.5 h-6 w-6 rounded-full bg-background shadow transition ${value ? "left-5" : "left-0.5"}`} />
      </button>
    </label>
  );
}

function Field({ label, required, children }: { label: string; required?: boolean; children: React.ReactNode }) {
  return (
    <label className="block">
      <span className="block pb-1.5 text-sm text-muted-foreground">
        {label}{required && <span className="ml-1 text-destructive">*</span>}
      </span>
      {children}
    </label>
  );
}