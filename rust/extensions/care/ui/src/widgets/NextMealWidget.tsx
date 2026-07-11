import { useT } from "../hooks/useT";

interface Props { childId: string; }

export function NextMealWidget({ childId }: Props) {
  const t = useT();
  return (
    <section className="rounded-[var(--care-radius)] border p-3" data-child={childId}>
      <h3 className="text-sm font-medium">{t("menu.today")}</h3>
    </section>
  );
}