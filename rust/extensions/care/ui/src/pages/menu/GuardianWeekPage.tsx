import { useEffect, useMemo, useState } from "react";
import { TriangleAlert, ArrowRight } from "lucide-react";
import { useCareApi } from "../../api/care";
import { LargeTitle } from "../../components/LargeTitle";
import { Segmented } from "../../components/ui/segmented";
import { useT } from "../../hooks/useT";
import type { ChildRow } from "../child/ChildrenListPage";
import {
  SLOTS,
  isoDate,
  mondayOf,
  useAllergenLabel,
  type MenuWeek,
  type WeekSlot,
  type WeekSub,
} from "./shared";

// GUARDIAN week view — the primary one-handed phone surface. Never renders any
// other child's data: care.menu.week is authz-scoped to the caller's edges and
// returns only THIS child's derived substitution rows.
export function GuardianWeekPage() {
  const t = useT();
  const api = useCareApi();
  const label = useAllergenLabel();
  const weekStart = useMemo(() => isoDate(mondayOf(new Date())), []);

  const [children, setChildren] = useState<ChildRow[] | null>(null);
  const [childId, setChildId] = useState<string | null>(null);
  const [week, setWeek] = useState<MenuWeek | null>(null);
  const [loadingWeek, setLoadingWeek] = useState(false);

  useEffect(() => {
    api
      .list<ChildRow>("child")
      .then((c) => {
        const active = c.filter((x) => !x.archived);
        setChildren(active);
        setChildId(active[0]?.id ?? null);
      })
      .catch(() => setChildren([]));
  }, []);

  useEffect(() => {
    if (!childId) {
      setWeek(null);
      return;
    }
    let live = true;
    setLoadingWeek(true);
    api
      .run<MenuWeek>("menu.week", { child_id: childId, week_start: weekStart })
      .then((w) => live && setWeek(w))
      .catch(() => live && setWeek({ child_id: childId }))
      .finally(() => live && setLoadingWeek(false));
    return () => {
      live = false;
    };
  }, [childId, weekStart]);

  const activeChild = children?.find((c) => c.id === childId) ?? null;

  return (
    <main className="pb-24">
      <LargeTitle>{t("menu.week.title")}</LargeTitle>
      <p className="px-4 pb-3 text-[13px] text-muted-foreground">
        {t("menu.week_of", { date: weekStart })}
      </p>

      {children && children.length > 1 && (
        <div className="px-4 pb-4">
          <Segmented<string>
            value={childId ?? ""}
            onChange={setChildId}
            columns={Math.min(children.length, 3)}
            segments={children.map((c) => ({ value: c.id, label: c.name }))}
          />
        </div>
      )}

      <div className="space-y-6 px-4">
        {children === null || loadingWeek ? (
          <WeekSkeleton />
        ) : !week?.days?.length ? (
          <p className="py-16 text-center text-[15px] text-muted-foreground">
            {t("menu.empty_week")}
          </p>
        ) : (
          week.days.map((day) => (
            <DaySection key={day.date} date={day.date} slots={day.slots ?? []} label={label} />
          ))
        )}
      </div>

      {activeChild && children && children.length > 1 && (
        <p className="px-4 pt-4 text-center text-xs text-muted-foreground">
          {t("menu.for_child", { name: activeChild.name })}
        </p>
      )}
    </main>
  );
}

function DaySection({
  date,
  slots,
  label,
}: {
  date: string;
  slots: WeekSlot[];
  label: (k: string) => string;
}) {
  const t = useT();
  const byKey = new Map(slots.map((s) => [s.slot, s]));
  const anyItems = slots.some((s) => (s.items ?? []).length);
  return (
    <section>
      <h2 className="px-1 pb-2 text-xs font-semibold uppercase tracking-wide text-muted-foreground">
        {new Date(date + "T00:00:00").toLocaleDateString(undefined, {
          weekday: "long",
          month: "short",
          day: "numeric",
        })}
      </h2>
      <div className="divide-y divide-border overflow-hidden rounded-2xl border border-border bg-card shadow-sm">
        {!anyItems ? (
          <p className="px-4 py-5 text-[15px] text-muted-foreground">{t("menu.no_items")}</p>
        ) : (
          SLOTS.map((slot) => {
            const s = byKey.get(slot);
            const items = s?.items ?? [];
            if (!items.length) return null;
            return (
              <SlotRow
                key={slot}
                slotLabel={t("slot." + slot)}
                items={items.map((i) => i.name)}
                subs={s?.substitutions ?? []}
                label={label}
              />
            );
          })
        )}
      </div>
    </section>
  );
}

function SlotRow({
  slotLabel,
  items,
  subs,
  label,
}: {
  slotLabel: string;
  items: string[];
  subs: WeekSub[];
  label: (k: string) => string;
}) {
  const t = useT();
  return (
    <div className="px-4 py-3.5">
      <div className="text-xs font-semibold uppercase tracking-wide text-muted-foreground">
        {slotLabel}
      </div>
      <div className="pt-1 text-[15px] text-foreground">{items.join(", ")}</div>
      {subs.map((sub, i) => (
        <div key={i} className="pt-2">
          {sub.resolved && sub.substitute ? (
            <div className="flex items-center gap-1.5 text-[13px] text-muted-foreground">
              <span className="line-through">{sub.item}</span>
              <ArrowRight className="size-3.5 shrink-0" aria-hidden />
              <span className="font-medium text-foreground">{sub.substitute}</span>
            </div>
          ) : (
            <div className="flex flex-wrap items-center gap-2">
              <span className="inline-flex items-center gap-1 rounded-full bg-destructive/10 px-2.5 py-1 text-xs font-semibold text-destructive">
                <TriangleAlert className="size-3.5" aria-hidden />
                {t("menu.unresolved")}
              </span>
              <span className="text-[13px] text-muted-foreground">
                {sub.item} · {label(sub.reason)}
              </span>
            </div>
          )}
        </div>
      ))}
    </div>
  );
}

function WeekSkeleton() {
  return (
    <div className="space-y-6" aria-busy="true">
      {[0, 1, 2].map((i) => (
        <div key={i} className="space-y-2">
          <div className="h-3 w-24 animate-pulse rounded bg-muted" />
          <div className="h-28 animate-pulse rounded-2xl bg-muted" />
        </div>
      ))}
    </div>
  );
}
