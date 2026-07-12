import { useEffect, useState } from "react";
import { useCareApi } from "../../api/care";
import { PageTitle } from "../../components/PageTitle";
import { useT } from "../../hooks/useT";

export interface CenterRow { id: string; name: string; default_locale?: string; archived?: boolean; address?: string; phone?: string; email?: string; }
export interface RoomRow { id: string; name: string; center_id: string; archived?: boolean; }

export function CentersRoomsPage() {
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
    <main className="pb-24">
      <PageTitle>{t("admin.schools")}</PageTitle>
      {error && <p className="px-4 py-2 text-sm text-destructive">{t("common.error_generic")}</p>}

      <section className="px-4">
        <header className="flex items-baseline justify-between pb-2">
          <h2 className="text-base font-semibold tracking-tight">{t("center.list.title")}</h2>
          <button
            onClick={() => setShowCenter(true)}
            className="rounded-full bg-primary px-3 py-1.5 text-sm font-medium text-primary-foreground"
          >
            {t("common.add")}
          </button>
        </header>
        {!centers?.length ? (
          <p className="py-4 text-sm opacity-60">{t("center.empty")}</p>
        ) : (
          <ul className="divide-y divide-border rounded-2xl border bg-card">
            {centers.map((c) => (
              <li key={c.id}>
                <div
                  className="flex w-full items-center justify-between px-4 py-3 text-left"
                >
                  <span>
                    <span className="block font-medium">{c.name}</span>
                    {c.address && <span className="block text-xs opacity-60">{c.address}</span>}
                  </span>
                  <span className="text-xs uppercase tracking-wide opacity-60">{c.default_locale ?? ""}</span>
                </div>
              </li>
            ))}
          </ul>
        )}
      </section>

      <section className="mt-8 px-4">
        <header className="flex items-baseline justify-between pb-2">
          <h2 className="text-base font-semibold tracking-tight">{t("room.list.title")}</h2>
        </header>
        {!rooms?.length ? (
          <p className="py-4 text-sm opacity-60">{t("room.empty")}</p>
        ) : (
          <ul className="divide-y divide-border rounded-2xl border bg-card">
            {rooms.map((r) => {
              const center = centers?.find((c) => c.id === r.center_id);
              return (
                <li key={r.id}>
                  <div
                    className="flex w-full items-center justify-between px-4 py-3 text-left"
                  >
                    <span>
                      <span className="block font-medium">{r.name}</span>
                      <span className="block text-xs opacity-60">{center?.name ?? r.center_id}</span>
                    </span>
                  </div>
                </li>
              );
            })}
          </ul>
        )}
        {centers?.map((c) => (
          <button
            key={c.id}
            onClick={() => setShowRoom(c.id)}
            className="mt-2 block w-full rounded-2xl border border-dashed px-4 py-3 text-left text-sm text-primary"
          >
            + {t("common.add")} {t("room.name").toLowerCase()} → {c.name}
          </button>
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
  const [locale, setLocale] = useState(initial?.default_locale ?? "en");
  const [busy, setBusy] = useState(false);
  const [err, setErr] = useState<string | null>(null);

  async function save() {
    setBusy(true);
    setErr(null);
    try {
      const id = initial?.id ?? name.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "") || "center";
      await api.run("center.create", { id, name, address, phone, email, default_locale: locale });
      onDone();
    } catch (e) {
      setErr(t("common.error_generic"));
    } finally {
      setBusy(false);
    }
  }

  return (
    <main className="pb-24">
      <PageTitle>{initial ? t("center.editor.title.edit") : t("center.editor.title.new")}</PageTitle>
      <form onSubmit={(e) => { e.preventDefault(); save(); }} className="space-y-4 px-4">
        <Field label={t("center.name")} required>
          <input value={name} onChange={(e) => setName(e.target.value)} required className="w-full rounded-xl border border-border bg-card px-3 py-2.5" />
        </Field>
        <Field label={t("center.address")}>
          <input value={address} onChange={(e) => setAddress(e.target.value)} className="w-full rounded-xl border border-border bg-card px-3 py-2.5" />
        </Field>
        <Field label={t("center.phone")}>
          <input type="tel" value={phone} onChange={(e) => setPhone(e.target.value)} className="w-full rounded-xl border border-border bg-card px-3 py-2.5" />
        </Field>
        <Field label={t("center.email")}>
          <input type="email" value={email} onChange={(e) => setEmail(e.target.value)} className="w-full rounded-xl border border-border bg-card px-3 py-2.5" />
        </Field>
        <Field label={t("center.default_locale")}>
          <div className="grid grid-cols-2 gap-2">
            {(["en", "es"] as const).map((l) => (
              <button
                key={l}
                type="button"
                onClick={() => setLocale(l)}
                className={`rounded-xl border px-3 py-2.5 text-sm ${locale === l ? "border-primary bg-primary text-primary-foreground" : "border-border bg-card"}`}
              >
                {t(`center.locale.${l}`)}
              </button>
            ))}
          </div>
        </Field>
        {err && <p className="text-sm text-red-500">{err}</p>}
        <div className="flex gap-2 pt-4">
          <button type="button" onClick={onDone} className="flex-1 rounded-xl border border-border px-4 py-3">{t("common.cancel")}</button>
          <button type="submit" disabled={busy} className="flex-1 rounded-xl bg-primary px-4 py-3 font-medium text-primary-foreground disabled:opacity-50">
            {t("common.save")}
          </button>
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
      const id = initial?.id ?? name.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "") || "room";
      await api.run("room.create", { id, name, center_id: cid });
      onDone();
    } finally { setBusy(false); }
  }

  return (
    <main className="pb-24">
      <PageTitle>{initial ? t("room.editor.title.edit") : t("room.editor.title.new")}</PageTitle>
      <form onSubmit={(e) => { e.preventDefault(); save(); }} className="space-y-4 px-4">
        <Field label={t("room.name")} required>
          <input value={name} onChange={(e) => setName(e.target.value)} required className="w-full rounded-xl border border-border bg-card px-3 py-2.5" />
        </Field>
        <Field label={t("room.center")}>
          <select value={cid} onChange={(e) => setCid(e.target.value)} className="w-full rounded-xl border border-border bg-card px-3 py-2.5">
            {centers.map((c) => <option key={c.id} value={c.id}>{c.name}</option>)}
          </select>
        </Field>
        <div className="flex gap-2 pt-4">
          <button type="button" onClick={onDone} className="flex-1 rounded-xl border border-border px-4 py-3">{t("common.cancel")}</button>
          <button type="submit" disabled={busy} className="flex-1 rounded-xl bg-primary px-4 py-3 font-medium text-primary-foreground disabled:opacity-50">
            {t("common.save")}
          </button>
        </div>
      </form>
    </main>
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