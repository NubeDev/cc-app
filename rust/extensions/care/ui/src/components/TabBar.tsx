import { useT } from "../hooks/useT";
import { useCareSession } from "../hooks/useCareSession";

export function TabBar() {
  const t = useT();
  const session = useCareSession();
  const items: Array<{ key: string; label: string }> = [
    { key: "feed", label: t("nav.feed") },
    { key: "children", label: t("nav.children") },
    { key: "menus", label: t("nav.menus") },
    { key: "messages", label: t("nav.messages") },
  ];
  if (session?.role === "admin") items.push({ key: "admin", label: t("nav.admin") });

  return (
    <nav className="fixed inset-x-0 bottom-0 flex justify-around border-t bg-background py-2">
      {items.map((i) => (
        <a key={i.key} href={`#/${i.key}`} className="min-h-[var(--care-touch-target)] px-3 text-sm">
          {i.label}
        </a>
      ))}
    </nav>
  );
}