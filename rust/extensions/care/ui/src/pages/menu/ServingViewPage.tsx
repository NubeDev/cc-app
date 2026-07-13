import { useEffect, useMemo, useState } from "react";
import { TriangleAlert, ArrowRight } from "lucide-react";
import { useCareApi } from "../../api/care";
import { LargeTitle } from "../../components/LargeTitle";
import { Field } from "../../components/Field";
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
  SLOTS,
  AllergenBadge,
  isoDate,
  useAllergenLabel,
  type Menu,
  type MenuItem,
  type MenuSubstitution,
  type Slot,
} from "./shared";

// STAFF serving view — big, legible, "who can't eat what right now". Today's
// slots for the staff's room; every allergen-flagged item shows a loud red flag
// and its entered substitute prominently.
export function ServingViewPage() {
  const t = useT();
  const api = useCareApi();
  const today = useMemo(() => isoDate(new Date()), []);

  const [rooms, setRooms] = useState<RoomRow[] | null>(null);
  const [roomId, setRoomId] = useState("");
  const [menus, setMenus] = useState<Record<string, Menu>>({});
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    api
      .list<RoomRow>("room")
      .then((r) => {
        setRooms(r);
        setRoomId((prev) => prev || r[0]?.id || "");
      })
      .catch(() => setRooms([]));
  }, []);

  useEffect(() => {
    if (!roomId) return;
    let live = true;
    setLoading(true);
    Promise.all(
      SLOTS.map((slot) =>
        api
          .run<Menu>("menu.get", { date: today, room_id: roomId, slot })
          .catch(() => null),
      ),
    )
      .then((results) => {
        if (!live) return;
        const next: Record<string, Menu> = {};
        SLOTS.forEach((slot, i) => {
          const m = results[i];
          if (m && m.items?.length) next[slot] = m;
        });
        setMenus(next);
      })
      .finally(() => live && setLoading(false));
    return () => {
      live = false;
    };
  }, [roomId, today]);

  const anySlots = SLOTS.some((s) => menus[s]);

  return (
    <main className="pb-24">
      <LargeTitle>{t("menu.serving.title")}</LargeTitle>
      <p className="px-4 pb-3 text-[15px] text-muted-foreground">
        {new Date(today + "T00:00:00").toLocaleDateString(undefined, {
          weekday: "long",
          month: "long",
          day: "numeric",
        })}
      </p>

      <div className="px-4 pb-4">
        <Field label={t("menu.pick_room")}>
          <Select value={roomId} onValueChange={setRoomId}>
            <SelectTrigger className="h-12 text-base">
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

      <div className="space-y-4 px-4">
        {loading && !anySlots ? (
          [0, 1].map((i) => <div key={i} className="h-40 animate-pulse rounded-2xl bg-muted" />)
        ) : !anySlots ? (
          <p className="py-16 text-center text-[15px] text-muted-foreground">{t("menu.no_items")}</p>
        ) : (
          SLOTS.map((slot) => (menus[slot] ? <SlotCard key={slot} slot={slot} menu={menus[slot]} /> : null))
        )}
      </div>
    </main>
  );
}

function SlotCard({ slot, menu }: { slot: Slot; menu: Menu }) {
  const t = useT();
  const label = useAllergenLabel();
  const items = menu.items ?? [];
  const subs = menu.substitutions ?? [];

  return (
    <section className="overflow-hidden rounded-2xl border border-border bg-card shadow-sm">
      <h2 className="border-b border-border px-4 py-3 text-lg font-bold text-foreground">
        {t("slot." + slot)}
      </h2>
      <ul className="divide-y divide-border">
        {items.map((it, i) => (
          <ItemRow key={i} item={it} subs={subs} label={label} />
        ))}
      </ul>
    </section>
  );
}

function ItemRow({
  item,
  subs,
  label,
}: {
  item: MenuItem;
  subs: MenuSubstitution[];
  label: (k: string) => string;
}) {
  const t = useT();
  const allergens = item.allergens ?? [];
  const flagged = allergens.length > 0;
  // Substitutes entered for this item's allergens.
  const relevant = subs.filter(
    (s) => allergens.includes(s.restriction) && (s.substitute ?? "").trim(),
  );
  const unresolved = allergens.filter(
    (a) => !subs.some((s) => s.restriction === a && (s.substitute ?? "").trim()),
  );

  return (
    <li className="px-4 py-4">
      <div className="flex flex-wrap items-center gap-2">
        {flagged && <TriangleAlert className="size-5 shrink-0 text-destructive" aria-hidden />}
        <span className="text-lg font-semibold text-foreground">{item.name}</span>
        {allergens.map((a) => (
          <AllergenBadge key={a} tone="flag" label={label(a)} />
        ))}
      </div>

      {relevant.map((s, i) => (
        <div
          key={i}
          className="mt-2 flex items-center gap-2 rounded-xl bg-muted px-3 py-2 text-[15px]"
        >
          <span className="font-medium text-muted-foreground">{label(s.restriction)}</span>
          <ArrowRight className="size-4 shrink-0 text-muted-foreground" aria-hidden />
          <span className="font-semibold text-foreground">{s.substitute}</span>
        </div>
      ))}

      {unresolved.length > 0 && (
        <div className="mt-2 flex items-center gap-1.5 rounded-xl bg-destructive/10 px-3 py-2 text-sm font-semibold text-destructive">
          <TriangleAlert className="size-4 shrink-0" aria-hidden />
          {t("menu.unresolved")} · {unresolved.map(label).join(", ")}
        </div>
      )}
    </li>
  );
}
