import { useMemo, useState } from "react";
import { Plus, TriangleAlert, X } from "lucide-react";
import { useCareApi } from "../../api/care";
import { Button } from "../../components/ui/button";
import { Input } from "../../components/ui/input";
import { useT } from "../../hooks/useT";
import {
  ALLERGENS,
  AllergenBadge,
  AllergenToggle,
  unresolvedRestrictions,
  useAllergenLabel,
  type Menu,
  type MenuItem,
  type MenuSubstitution,
  type Slot,
} from "./shared";

// The week-grid CELL (a room×slot tile) + its bottom-sheet CELL EDITOR, split
// out of MenuPlannerPage to keep each file within the FILE-LAYOUT 400-line cap.
export function Cell({
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

export function CellEditor({
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
