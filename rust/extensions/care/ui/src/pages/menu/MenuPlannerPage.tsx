import { useEffect, useMemo, useState } from "react";
import { ChevronLeft, ChevronRight, Plus, TriangleAlert, Copy, X } from "lucide-react";
import { useCareApi } from "../../api/care";
import { LargeTitle } from "../../components/LargeTitle";
import { Field } from "../../components/Field";
import { Button } from "../../components/ui/button";
import { Input } from "../../components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "../../components/ui/select";
import { useT } from "../../hooks/useT";
import type { RoomRow } from "../child/ChildrenListPage";
import {
  ALLERGENS,
  SLOTS,
  AllergenBadge,
  AllergenToggle,
  isoDate,
  mondayOf,
  addDays,
  parseIso,
  weekDates,
  unresolvedRestrictions,
  useAllergenLabel,
  type Menu,
  type MenuItem,
  type MenuSubstitution,
  type Slot,
} from "./shared";

type CellKey = string; // `${date}|${slot}`
const cellKey = (date: string, slot: string): CellKey => `${date}|${slot}`;

// ADMIN planner — a real week × slot grid at laptop width, stacked on phone.
// The editor is a bottom sheet on phone / an inline overlay on desktop.
export function MenuPlannerPage() {
  const t = useT();
  const api = useCareApi();
  const label = useAllergenLabel();

  const [rooms, setRooms] = useState<RoomRow[] | null>(null);
  const [roomId, setRoomId] = useState("");
  const [weekStart, setWeekStart] = useState(() => isoDate(mondayOf(new Date())));
  const [menus, setMenus] = useState<Record<CellKey, Menu>>({});
  const [loading, setLoading] = useState(false);
  const [editing, setEditing] = useState<{ date: string; slot: Slot } | null>(null);
  const [copied, setCopied] = useState(false);

  const dates = useMemo(() => weekDates(weekStart), [weekStart]);

  useEffect(() => {
    api
      .list<RoomRow>("room")
      .then((r) => {
        setRooms(r);
        setRoomId((prev) => prev || r[0]?.id || "");
      })
      .catch(() => setRooms([]));
  }, []);

  async function loadWeek() {
    if (!roomId) return;
    setLoading(true);
    try {
      const pairs: Array<{ date: string; slot: Slot }> = [];
      for (const date of dates) for (const slot of SLOTS) pairs.push({ date, slot });
      const results = await Promise.all(
        pairs.map((p) =>
          api
            .run<Menu>("menu.get", { date: p.date, room_id: roomId, slot: p.slot })
            .catch(() => null),
        ),
      );
      const next: Record<CellKey, Menu> = {};
      pairs.forEach((p, i) => {
        const m = results[i];
        if (m && (m.items?.length || m.substitutions?.length))
          next[cellKey(p.date, p.slot)] = m;
      });
      setMenus(next);
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    loadWeek();
  }, [roomId, weekStart]);

  function shiftWeek(delta: number) {
    setWeekStart(isoDate(addDays(parseIso(weekStart), delta * 7)));
  }

  async function copyLastWeek() {
    if (!roomId) return;
    const from = isoDate(addDays(parseIso(weekStart), -7));
    await api.run("menu.copy_week", {
      room_id: roomId,
      from_week_start: from,
      to_week_start: weekStart,
    });
    setCopied(true);
    setTimeout(() => setCopied(false), 2500);
    await loadWeek();
  }

  function onSaved(m: Menu) {
    setMenus((prev) => ({ ...prev, [cellKey(m.date, m.slot)]: m }));
    setEditing(null);
  }

  return (
    <main className="pb-24">
      <LargeTitle>{t("menu.planner.title")}</LargeTitle>

      <div className="flex flex-wrap items-end gap-3 px-4 pb-4">
        <div className="min-w-[10rem] flex-1">
          <Field label={t("menu.pick_room")}>
            <Select value={roomId} onValueChange={setRoomId}>
              <SelectTrigger>
                <SelectValue placeholder="—" />
              </SelectTrigger>
              <SelectContent>
                {(rooms ?? []).map((r) => (
                  <SelectItem key={r.id} value={r.id}>
                    {r.name}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </Field>
        </div>
        <div className="flex items-center gap-1">
          <Button variant="outline" size="icon" onClick={() => shiftWeek(-1)} aria-label={t("menu.prev_week")}>
            <ChevronLeft />
          </Button>
          <span className="min-w-[7rem] text-center text-[13px] font-medium text-foreground">
            {t("menu.week_of", { date: weekStart })}
          </span>
          <Button variant="outline" size="icon" onClick={() => shiftWeek(1)} aria-label={t("menu.next_week")}>
            <ChevronRight />
          </Button>
        </div>
        <Button variant="secondary" onClick={copyLastWeek} disabled={!roomId}>
          <Copy /> {copied ? t("menu.copied") : t("menu.copy_week")}
        </Button>
      </div>

      {!roomId ? (
        <p className="px-4 py-16 text-center text-[15px] text-muted-foreground">{t("room.empty")}</p>
      ) : (
        <div className="px-4">
          {/* Desktop grid ≥1024px */}
          <div className="hidden overflow-x-auto lg:block">
            <table className="w-full border-separate border-spacing-1">
              <thead>
                <tr>
                  <th className="w-24" />
                  {SLOTS.map((slot) => (
                    <th
                      key={slot}
                      className="pb-1 text-left text-xs font-semibold uppercase tracking-wide text-muted-foreground"
                    >
                      {t("slot." + slot)}
                    </th>
                  ))}
                </tr>
              </thead>
              <tbody>
                {dates.map((date) => (
                  <tr key={date}>
                    <th className="pr-2 align-top text-left text-xs font-medium text-muted-foreground">
                      {new Date(date + "T00:00:00").toLocaleDateString(undefined, {
                        weekday: "short",
                        day: "numeric",
                      })}
                    </th>
                    {SLOTS.map((slot) => (
                      <td key={slot} className="align-top">
                        <Cell
                          menu={menus[cellKey(date, slot)]}
                          loading={loading}
                          label={label}
                          onClick={() => setEditing({ date, slot })}
                        />
                      </td>
                    ))}
                  </tr>
                ))}
              </tbody>
            </table>
          </div>

          {/* Phone: stacked by day */}
          <div className="space-y-6 lg:hidden">
            {dates.map((date) => (
              <section key={date}>
                <h2 className="px-1 pb-2 text-xs font-semibold uppercase tracking-wide text-muted-foreground">
                  {new Date(date + "T00:00:00").toLocaleDateString(undefined, {
                    weekday: "long",
                    month: "short",
                    day: "numeric",
                  })}
                </h2>
                <div className="space-y-2">
                  {SLOTS.map((slot) => (
                    <div key={slot}>
                      <div className="px-1 pb-1 text-[11px] font-semibold uppercase tracking-wide text-muted-foreground">
                        {t("slot." + slot)}
                      </div>
                      <Cell
                        menu={menus[cellKey(date, slot)]}
                        loading={loading}
                        label={label}
                        onClick={() => setEditing({ date, slot })}
                      />
                    </div>
                  ))}
                </div>
              </section>
            ))}
          </div>
        </div>
      )}

      {editing && (
        <CellEditor
          date={editing.date}
          slot={editing.slot}
          roomId={roomId}
          initial={menus[cellKey(editing.date, editing.slot)]}
          onClose={() => setEditing(null)}
          onSaved={onSaved}
        />
      )}
    </main>
  );
}

function Cell({
  menu,
  loading,
  label,
  onClick,
}: {
  menu?: Menu;
  loading: boolean;
  label: (k: string) => string;
  onClick: () => void;
}) {
  const t = useT();
  const items = menu?.items ?? [];
  const unresolved = menu ? unresolvedRestrictions(menu) : [];
  return (
    <button
      onClick={onClick}
      className="flex min-h-[4.5rem] w-full flex-col gap-1 rounded-xl border border-border bg-card p-2.5 text-left shadow-sm transition-colors hover:bg-accent focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
    >
      {loading && !menu ? (
        <span className="h-4 w-full animate-pulse rounded bg-muted" />
      ) : !items.length ? (
        <span className="inline-flex items-center gap-1 text-xs text-muted-foreground">
          <Plus className="size-3.5" aria-hidden /> {t("menu.add_item")}
        </span>
      ) : (
        <>
          {unresolved.length > 0 && (
            <span className="inline-flex w-fit items-center gap-1 rounded-full bg-destructive/10 px-2 py-0.5 text-[11px] font-semibold text-destructive">
              <TriangleAlert className="size-3" aria-hidden />
              {t("menu.unresolved_count", { count: unresolved.length })}
            </span>
          )}
          {items.map((it, i) => (
            <span key={i} className="text-[13px] leading-tight text-foreground">
              {it.name}
              {(it.allergens ?? []).length > 0 && (
                <span className="pl-1 text-[11px] font-medium text-destructive">
                  {(it.allergens ?? []).map(label).join(", ")}
                </span>
              )}
            </span>
          ))}
        </>
      )}
    </button>
  );
}

function CellEditor({
  date,
  slot,
  roomId,
  initial,
  onClose,
  onSaved,
}: {
  date: string;
  slot: Slot;
  roomId: string;
  initial?: Menu;
  onClose: () => void;
  onSaved: (m: Menu) => void;
}) {
  const t = useT();
  const api = useCareApi();
  const label = useAllergenLabel();
  const [items, setItems] = useState<MenuItem[]>(
    initial?.items?.map((i) => ({ name: i.name, allergens: [...(i.allergens ?? [])] })) ?? [],
  );
  const [subs, setSubs] = useState<MenuSubstitution[]>(
    initial?.substitutions?.map((s) => ({ ...s })) ?? [],
  );
  const [busy, setBusy] = useState(false);
  const [err, setErr] = useState<string | null>(null);

  // The set of restrictions currently implied by tagged items.
  const restrictions = useMemo(() => {
    const set = new Set<string>();
    for (const it of items) for (const a of it.allergens ?? []) set.add(a);
    return [...set];
  }, [items]);

  function addItem() {
    setItems((p) => [...p, { name: "", allergens: [] }]);
  }
  function setItemName(idx: number, name: string) {
    setItems((p) => p.map((it, i) => (i === idx ? { ...it, name } : it)));
  }
  function toggleAllergen(idx: number, key: string) {
    setItems((p) =>
      p.map((it, i) => {
        if (i !== idx) return it;
        const has = (it.allergens ?? []).includes(key);
        return {
          ...it,
          allergens: has
            ? (it.allergens ?? []).filter((a) => a !== key)
            : [...(it.allergens ?? []), key],
        };
      }),
    );
  }
  function removeItem(idx: number) {
    setItems((p) => p.filter((_, i) => i !== idx));
  }
  function setSubstitute(restriction: string, substitute: string) {
    setSubs((p) => {
      const exists = p.find((s) => s.restriction === restriction);
      if (exists) return p.map((s) => (s.restriction === restriction ? { ...s, substitute } : s));
      return [...p, { restriction, substitute }];
    });
  }

  async function save() {
    setBusy(true);
    setErr(null);
    try {
      const cleanItems = items
        .filter((it) => it.name.trim())
        .map((it) => ({ name: it.name.trim(), allergens: it.allergens ?? [] }));
      const cleanSubs = subs
        .filter((s) => restrictions.includes(s.restriction) && (s.substitute ?? "").trim())
        .map((s) => ({ restriction: s.restriction, substitute: (s.substitute ?? "").trim() }));
      await api.run("menu.set", {
        date,
        room_id: roomId,
        slot,
        items: cleanItems,
        substitutions: cleanSubs,
      });
      onSaved({ date, room_id: roomId, slot, items: cleanItems, substitutions: cleanSubs });
    } catch (e) {
      setErr((e as Error).message ?? t("common.error_generic"));
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="fixed inset-0 z-50 flex items-end justify-center bg-foreground/40 backdrop-blur-sm sm:items-center">
      <div className="max-h-[90vh] w-full max-w-lg overflow-y-auto rounded-t-2xl border border-border bg-background p-4 shadow-sm sm:rounded-2xl">
        <div className="flex items-center justify-between pb-3">
          <h2 className="text-lg font-semibold text-foreground">
            {new Date(date + "T00:00:00").toLocaleDateString(undefined, {
              weekday: "short",
              month: "short",
              day: "numeric",
            })}{" "}
            · {t("slot." + slot)}
          </h2>
          <Button variant="ghost" size="icon" onClick={onClose} aria-label={t("common.cancel")}>
            <X />
          </Button>
        </div>

        <div className="space-y-3">
          {items.map((it, idx) => (
            <div key={idx} className="rounded-2xl border border-border bg-card p-3 shadow-sm">
              <div className="flex items-center gap-2">
                <Input
                  value={it.name}
                  placeholder={t("menu.item_name")}
                  onChange={(e) => setItemName(idx, e.target.value)}
                />
                <Button variant="ghost" size="icon" onClick={() => removeItem(idx)} aria-label={t("common.delete")}>
                  <X />
                </Button>
              </div>
              <div className="pt-2">
                <div className="pb-1.5 text-[11px] font-semibold uppercase tracking-wide text-muted-foreground">
                  {t("menu.item_allergens")}
                </div>
                <div className="flex flex-wrap gap-1.5">
                  {ALLERGENS.map((a) => (
                    <AllergenToggle
                      key={a}
                      label={label(a)}
                      active={(it.allergens ?? []).includes(a)}
                      onToggle={() => toggleAllergen(idx, a)}
                    />
                  ))}
                </div>
              </div>
            </div>
          ))}
          <Button variant="outline" className="w-full" onClick={addItem}>
            <Plus /> {t("menu.add_item")}
          </Button>
        </div>

        {restrictions.length > 0 && (
          <div className="pt-5">
            <div className="pb-2 text-xs font-semibold uppercase tracking-wide text-muted-foreground">
              {t("menu.substitutions")}
            </div>
            <div className="space-y-2">
              {restrictions.map((r) => {
                const val = subs.find((s) => s.restriction === r)?.substitute ?? "";
                const unresolved = !val.trim();
                return (
                  <div key={r} className="rounded-2xl border border-border bg-card p-3 shadow-sm">
                    <div className="flex items-center justify-between gap-2 pb-1.5">
                      <span className="text-[13px] font-medium text-foreground">
                        {t("menu.substitute_for", { allergen: label(r) })}
                      </span>
                      {unresolved && <AllergenBadge tone="flag" label={t("menu.unresolved")} />}
                    </div>
                    <Input
                      value={val}
                      placeholder={t("menu.enter_substitute")}
                      onChange={(e) => setSubstitute(r, e.target.value)}
                    />
                  </div>
                );
              })}
            </div>
          </div>
        )}

        {err && (
          <p role="alert" className="pt-3 text-sm text-destructive">
            {err}
          </p>
        )}
        <div className="flex gap-2 pt-5">
          <Button variant="outline" className="flex-1" onClick={onClose}>
            {t("common.cancel")}
          </Button>
          <Button className="flex-1" onClick={save} disabled={busy}>
            {t("common.save")}
          </Button>
        </div>
      </div>
    </div>
  );
}
