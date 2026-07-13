import { useEffect, useState } from "react";
import { Plus } from "lucide-react";
import { useCareApi } from "../../api/care";
import { LargeTitle } from "../../components/LargeTitle";
import { Field } from "../../components/Field";
import { Button } from "../../components/ui/button";
import { Input } from "../../components/ui/input";
import { Segmented } from "../../components/ui/segmented";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "../../components/ui/select";
import { useT } from "../../hooks/useT";

export interface CenterRow { id: string; name: string; default_locale?: string; archived?: boolean; address?: string; phone?: string; email?: string; }
export interface RoomRow { id: string; name: string; center_id: string; archived?: boolean; }

export function CentersRoomsPage({ embedded }: { embedded?: boolean } = {}) {
  const t = useT();
  const api = useCareApi();
  const [centers, setCenters] = useState<CenterRow[] | null>(null);
  const [rooms, setRooms] = useState<RoomRow[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [showCenter, setShowCenter] = useState(false);
  const [editingCenter, setEditingCenter] = useState<CenterRow | null>(null);
  const [showRoom, setShowRoom] = useState<string | null>(null);
  const [editingRoom, setEditingRoom] = useState<RoomRow | null>(null);

  async function refresh() {
    try {
      const [c, r] = await Promise.all([
        api.list<CenterRow>("center"),
        api.list<RoomRow>("room"),
      ]);
      setCenters(c);
      setRooms(r);
    } catch (e) {
      setError((e as Error).message);
    }
  }
  useEffect(() => { refresh(); }, []);

  if (showCenter || editingCenter) {
    return <CenterEditor initial={editingCenter} onDone={() => { setShowCenter(false); setEditingCenter(null); refresh(); }} />;
  }
  if (showRoom !== null || editingRoom) {
    return <RoomEditor centerId={showRoom ?? editingRoom?.center_id ?? ""} initial={editingRoom} centers={centers ?? []} onDone={() => { setShowRoom(null); setEditingRoom(null); refresh(); }} />;
  }

  return (
    <main className={embedded ? "" : "pb-24"}>
      {!embedded && <LargeTitle>{t("admin.schools")}</LargeTitle>}
      {error && <p role="alert" className="px-4 py-2 text-sm text-destructive">{t("common.error_generic")}</p>}

      <section className="px-4 pt-1">
        <header className="flex items-center justify-between pb-2">
          <h2 className="text-base font-semibold tracking-tight text-foreground">{t("center.list.title")}</h2>
          <Button size="pill" onClick={() => setShowCenter(true)}>
            <Plus /> {t("common.add")}
          </Button>
        </header>
        {!centers?.length ? (
          <p className="py-4 text-[15px] text-muted-foreground">{t("center.empty")}</p>
        ) : (
          <ul className="divide-y divide-border overflow-hidden rounded-2xl border border-border bg-card shadow-sm">
            {centers.map((c) => (
              <li key={c.id} className="flex items-center justify-between gap-3 px-4 py-3">
                <span className="min-w-0">
                  <span className="block truncate font-medium text-foreground">{c.name}</span>
                  {c.address && <span className="block truncate text-[13px] text-muted-foreground">{c.address}</span>}
                </span>
                <span className="text-xs uppercase tracking-wide text-muted-foreground">{c.default_locale ?? ""}</span>
              </li>
            ))}
          </ul>
        )}
      </section>

      <section className="mt-8 px-4">
        <h2 className="pb-2 text-base font-semibold tracking-tight text-foreground">{t("room.list.title")}</h2>
        {!rooms?.length ? (
          <p className="py-4 text-[15px] text-muted-foreground">{t("room.empty")}</p>
        ) : (
          <ul className="divide-y divide-border overflow-hidden rounded-2xl border border-border bg-card shadow-sm">
            {rooms.map((r) => {
              const center = centers?.find((c) => c.id === r.center_id);
              return (
                <li key={r.id} className="flex items-center justify-between gap-3 px-4 py-3">
                  <span className="min-w-0">
                    <span className="block truncate font-medium text-foreground">{r.name}</span>
                    <span className="block truncate text-[13px] text-muted-foreground">{center?.name ?? r.center_id}</span>
                  </span>
                </li>
              );
            })}
          </ul>
        )}
        {centers?.map((c) => (
          <Button
            key={c.id}
            variant="outline"
            onClick={() => setShowRoom(c.id)}
            className="mt-2 w-full justify-start border-dashed text-primary"
          >
            <Plus /> {t("common.add")} {t("room.name").toLowerCase()} → {c.name}
          </Button>
        ))}
      </section>
    </main>
  );
}

function CenterEditor({ initial, onDone }: { initial: CenterRow | null; onDone: () => void }) {
  const t = useT();
  const api = useCareApi();
  const [name, setName] = useState(initial?.name ?? "");
  const [address, setAddress] = useState(initial?.address ?? "");
  const [phone, setPhone] = useState(initial?.phone ?? "");
  const [email, setEmail] = useState(initial?.email ?? "");
  const [locale, setLocale] = useState<"en" | "es">((initial?.default_locale as "en" | "es") ?? "en");
  const [busy, setBusy] = useState(false);
  const [err, setErr] = useState<string | null>(null);

  async function save() {
    setBusy(true);
    setErr(null);
    try {
      const id = initial?.id ?? (name.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "") || "center");
      await api.run("center.create", { id, name, address, phone, email, default_locale: locale });
      onDone();
    } catch {
      setErr(t("common.error_generic"));
    } finally {
      setBusy(false);
    }
  }

  return (
    <main className="pb-24">
      <LargeTitle>{initial ? t("center.editor.title.edit") : t("center.editor.title.new")}</LargeTitle>
      <form onSubmit={(e) => { e.preventDefault(); save(); }} className="space-y-4 px-4 pt-1">
        <Field label={t("center.name")} htmlFor="ce-name" required>
          <Input id="ce-name" value={name} onChange={(e) => setName(e.target.value)} required />
        </Field>
        <Field label={t("center.address")} htmlFor="ce-address">
          <Input id="ce-address" value={address} onChange={(e) => setAddress(e.target.value)} />
        </Field>
        <Field label={t("center.phone")} htmlFor="ce-phone">
          <Input id="ce-phone" type="tel" value={phone} onChange={(e) => setPhone(e.target.value)} />
        </Field>
        <Field label={t("center.email")} htmlFor="ce-email">
          <Input id="ce-email" type="email" value={email} onChange={(e) => setEmail(e.target.value)} />
        </Field>
        <Field label={t("center.default_locale")}>
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

function RoomEditor({ centerId, initial, centers, onDone }: { centerId: string; initial: RoomRow | null; centers: CenterRow[]; onDone: () => void }) {
  const t = useT();
  const api = useCareApi();
  const [name, setName] = useState(initial?.name ?? "");
  const [cid, setCid] = useState(initial?.center_id ?? centerId ?? centers[0]?.id ?? "");
  const [busy, setBusy] = useState(false);

  async function save() {
    setBusy(true);
    try {
      const id = initial?.id ?? (name.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "") || "room");
      await api.run("room.create", { id, name, center_id: cid });
      onDone();
    } finally { setBusy(false); }
  }

  return (
    <main className="pb-24">
      <LargeTitle>{initial ? t("room.editor.title.edit") : t("room.editor.title.new")}</LargeTitle>
      <form onSubmit={(e) => { e.preventDefault(); save(); }} className="space-y-4 px-4 pt-1">
        <Field label={t("room.name")} htmlFor="ro-name" required>
          <Input id="ro-name" value={name} onChange={(e) => setName(e.target.value)} required />
        </Field>
        <Field label={t("room.center")}>
          <Select value={cid} onValueChange={setCid}>
            <SelectTrigger><SelectValue /></SelectTrigger>
            <SelectContent>
              {centers.map((c) => <SelectItem key={c.id} value={c.id}>{c.name}</SelectItem>)}
            </SelectContent>
          </Select>
        </Field>
        <div className="flex gap-2 pt-2">
          <Button type="button" variant="outline" className="flex-1" onClick={onDone}>{t("common.cancel")}</Button>
          <Button type="submit" className="flex-1" disabled={busy}>{t("common.save")}</Button>
        </div>
      </form>
    </main>
  );
}
