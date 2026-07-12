import { useT } from "../hooks/useT";
import { useCareSession } from "../hooks/useCareSession";

interface Props {
  active: "today" | "children" | "admin";
  onChange: (tab: "today" | "children" | "admin") => void;
  showAdmin?: boolean;
}

export function TabBar({ active, onChange, showAdmin }: Props) {
  const t = useT();
  const session = useCareSession();
  const isAdmin = showAdmin ?? session?.role === "admin";

  const items: Array<{ key: "today" | "children" | "admin"; label: string }> = [
    { key: "today", label: t("nav.feed") },
    { key: "children", label: t("nav.children") },
  ];
  if (isAdmin) items.push({ key: "admin", label: t("nav.admin") });

  return (
    <nav className="fixed inset-x-0 bottom-0 z-20 flex justify-around border-t border-border bg-background/80 py-1.5 backdrop-blur supports-[backdrop-filter]:bg-background/60">
      {items.map((i) => (
        <button
          key={i.key}
          onClick={() => onChange(i.key)}
          className={`flex min-h-[44px] flex-1 flex-col items-center justify-center gap-0.5 px-2 text-xs font-medium transition ${active === i.key ? "text-primary" : "text-muted-foreground"}`}
        >
          <span className={`h-1 w-1 rounded-full ${active === i.key ? "bg-primary" : "bg-transparent"}`} />
          {i.label}
        </button>
      ))}
    </nav>
  );
}