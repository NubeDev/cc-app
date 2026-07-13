import { Home, Users, Settings } from "lucide-react";
import type { LucideIcon } from "lucide-react";
import { useT } from "../hooks/useT";
import { useCareSession } from "../hooks/useCareSession";
import { cn } from "../lib/cn";

type TabKey = "today" | "children" | "admin";

interface Props {
  active: TabKey;
  onChange: (tab: TabKey) => void;
  showAdmin?: boolean;
}

// The iOS bottom tab bar (thumb zone), translucent backdrop-blur chrome. Icons
// carry recognition, labels carry clarity; the active tab tints to the accent.
export function TabBar({ active, onChange, showAdmin }: Props) {
  const t = useT();
  const session = useCareSession();
  const isAdmin = showAdmin ?? session?.role === "admin";

  const items: Array<{ key: TabKey; label: string; icon: LucideIcon }> = [
    { key: "today", label: t("nav.feed"), icon: Home },
    { key: "children", label: t("nav.children"), icon: Users },
  ];
  if (isAdmin) items.push({ key: "admin", label: t("nav.admin"), icon: Settings });

  return (
    <nav className="fixed inset-x-0 bottom-0 z-20 border-t border-border/70 bg-background/70 pb-[env(safe-area-inset-bottom)] backdrop-blur-xl">
      <div className="mx-auto flex max-w-2xl justify-around">
      {items.map((i) => {
        const isActive = active === i.key;
        const Icon = i.icon;
        return (
          <button
            key={i.key}
            onClick={() => onChange(i.key)}
            aria-current={isActive ? "page" : undefined}
            className={cn(
              "flex min-h-[52px] flex-1 flex-col items-center justify-center gap-0.5 px-2 pt-1.5 text-[11px] font-medium transition-colors",
              "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-ring",
              isActive ? "text-primary" : "text-muted-foreground hover:text-foreground",
            )}
          >
            <Icon className="size-6" strokeWidth={isActive ? 2.4 : 2} aria-hidden />
            {i.label}
          </button>
        );
      })}
      </div>
    </nav>
  );
}
