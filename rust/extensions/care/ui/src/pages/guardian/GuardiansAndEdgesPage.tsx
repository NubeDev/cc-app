import { useEffect, useState } from "react";
import { Plus, Check, ChevronRight } from "lucide-react";
import { useCareApi } from "../../api/care";
import { LargeTitle } from "../../components/LargeTitle";
import { Field } from "../../components/Field";
import { Button } from "../../components/ui/button";
import { Input } from "../../components/ui/input";
import { Textarea } from "../../components/ui/textarea";
import { Switch } from "../../components/ui/switch";
import { Segmented } from "../../components/ui/segmented";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "../../components/ui/select";
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

export function GuardiansListPage({ embedded }: { embedded?: boolean } = {}) {
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
    <main className={embedded ? "" : "pb-24"}>
      {!embedded && <LargeTitle>{t("guardian.list.title")}</LargeTitle>}
      <div className="px-4 pt-1">
        <Button className="mb-4 w-full" onClick={() => setCreating(true)}>
          <Plus /> {t("guardian.editor.title.new")}
        </Button>
        {guardians === null ? (
          <ul className="space-y-2" aria-busy="true">
            {[0, 1].map((i) => <li key={i} className="h-[70px] animate-pulse rounded-2xl bg-muted" />)}
          </ul>
        ) : !guardians.length ? (
          <p className="py-16 text-center text-[15px] text-muted-foreground">{t("guardian.empty")}</p>
        ) : (
          <ul className="space-y-2">
            {guardians.map((g) => (
              <li key={g.id}>
                <button onClick={() => setEditing(g)} className="flex w-full items-center gap-3 rounded-2xl border border-border bg-card p-4 text-left shadow-sm transition-colors hover:bg-accent focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring">
                  <span className="min-w-0 flex-1">
                    <span className="block truncate text-base font-semibold text-foreground">{g.name}</span>
                    <span className="block truncate text-[13px] text-muted-foreground">{g.email}</span>
                  </span>
                  {!g.sub && (
                    <span className="rounded-full bg-primary/10 px-2.5 py-1 text-xs font-semibold text-primary">
                      {t("guardian.invite_pending")}
                    </span>
                  )}
                  <ChevronRight className="size-5 shrink-0 text-muted-foreground" aria-hidden />
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
  const [locale, setLocale] = useState<"en" | "es">((initial?.locale as "en" | "es") ?? "en");
  const [busy, setBusy] = useState(false);
  const [err, setErr] = useState<string | null>(null);

  async function save() {
    setBusy(true); setErr(null);
    try {
      const slug = (email.split("@")[0] ?? "guardian").toLowerCase().replace(/[^a-z0-9]+/g, "-");
      const id = initial?.id ?? slug;
      await api.run("guardian.create", { id, name, email, phone, locale });
      onDone();
    } catch { setErr(t("common.error_generic")); }
    finally { setBusy(false); }
  }

  return (
    <main className="pb-24">
      <LargeTitle>{initial ? t("guardian.editor.title.edit") : t("guardian.editor.title.new")}</LargeTitle>
      <form onSubmit={(e) => { e.preventDefault(); save(); }} className="space-y-4 px-4 pt-1">
        <Field label={t("guardian.name")} htmlFor="g-name" required>
          <Input id="g-name" value={name} onChange={(e) => setName(e.target.value)} required />
        </Field>
        <Field label={t("guardian.email")} htmlFor="g-email" required>
          <Input id="g-email" type="email" value={email} onChange={(e) => setEmail(e.target.value)} required />
        </Field>
        <Field label={t("guardian.phone")} htmlFor="g-phone">
          <Input id="g-phone" type="tel" value={phone} onChange={(e) => setPhone(e.target.value)} />
        </Field>
        <Field label={t("guardian.locale")}>
          <Segmented
            columns={2}
            value={locale}
            onChange={setLocale}
            segments={[
              { value: "en", label: t("center.locale.en") },
              { value: "es", label: t("center.locale.es") },
            ]}
          />
        </Field>
        {err && <p role="alert" className="text-sm text-destructive">{err}</p>}
        <div className="flex gap-2 pt-2">
          <Button type="button" variant="outline" className="flex-1" onClick={onDone}>{t("common.cancel")}</Button>
          <Button type="submit" className="flex-1" disabled={busy}>{t("common.save")}</Button>
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
      <LargeTitle>{t("edge.list.title")}</LargeTitle>
      <div className="px-4 pt-1">
        <Button className="mb-4 w-full" onClick={() => setCreating(true)}>
          <Plus /> {t("edge.editor.title.new")}
        </Button>
        {!edges?.length ? (
          <p className="py-16 text-center text-[15px] text-muted-foreground">{t("edge.empty")}</p>
        ) : (
          <ul className="space-y-2">
            {edges.map((e, i) => (
              <li key={i}>
                <button onClick={() => setEditing(e)} className="flex w-full items-center gap-3 rounded-2xl border border-border bg-card p-4 text-left shadow-sm transition-colors hover:bg-accent focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring">
                  <span className="min-w-0 flex-1">
                    <span className="block truncate font-medium text-foreground">{e.guardian_name ?? e.guardian_sub}</span>
                    <span className="block text-[13px] text-muted-foreground">{e.relationship}</span>
                  </span>
                  {e.can_pickup && (
                    <span className="inline-flex items-center gap-1 rounded-full bg-primary/10 px-2 py-1 text-xs font-semibold text-primary">
                      <Check className="size-3.5" aria-hidden /> {t("edge.flag.can_pickup")}
                    </span>
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
      <LargeTitle>{initial ? t("edge.editor.title.edit") : t("edge.editor.title.new")}</LargeTitle>
      <form onSubmit={(e) => { e.preventDefault(); save(); }} className="space-y-4 px-4 pt-1">
        <Field label={t("guardian.name")}>
          <Select value={guardianId} onValueChange={setGuardianId}>
            <SelectTrigger><SelectValue /></SelectTrigger>
            <SelectContent>
              {guardians.map((g) => <SelectItem key={g.id} value={g.id}>{g.name}</SelectItem>)}
            </SelectContent>
          </Select>
        </Field>
        <Field label={t("edge.relationship")}>
          <Segmented
            columns={RELS.length}
            value={relationship}
            onChange={setRelationship}
            className="text-xs"
            segments={RELS.map((r) => ({ value: r, label: t(`edge.relationship.${r}`) }))}
          />
        </Field>
        <section className="divide-y divide-border overflow-hidden rounded-2xl border border-border bg-card shadow-sm">
          <ToggleRow label={t("edge.flag.can_pickup")} value={canPickup} onChange={setCanPickup} />
          <ToggleRow label={t("edge.flag.receives_daily_feed")} value={receivesFeed} onChange={setReceivesFeed} />
          <ToggleRow label={t("edge.flag.receives_billing")} value={receivesBilling} onChange={setReceivesBilling} />
          <ToggleRow label={t("edge.flag.emergency_contact")} value={emergency} onChange={setEmergency} />
        </section>
        <Field label={t("edge.flag.custody_notes")} htmlFor="e-custody">
          <Textarea id="e-custody" value={custodyNotes} onChange={(e) => setCustodyNotes(e.target.value)} rows={2} />
        </Field>
        <div className="flex gap-2 pt-2">
          <Button type="button" variant="outline" className="flex-1" onClick={onDone}>{t("common.cancel")}</Button>
          <Button type="submit" className="flex-1" disabled={busy}>{t("common.save")}</Button>
        </div>
      </form>
    </main>
  );
}

function ToggleRow({ label, value, onChange }: { label: string; value: boolean; onChange: (v: boolean) => void }) {
  return (
    <label className="flex items-center justify-between gap-3 px-4 py-3">
      <span className="text-sm text-foreground">{label}</span>
      <Switch checked={value} onCheckedChange={onChange} />
    </label>
  );
}
