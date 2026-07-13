import type { ReactNode } from "react";

// The iOS large-title screen header (DESIGN.md §Typography: ~28–34px bold,
// tight tracking). Replaces the old compact PageTitle across care screens.
// `trailing` hosts an optional action (e.g. an Add button) on the baseline.
export function LargeTitle({ children, trailing }: { children: ReactNode; trailing?: ReactNode }) {
  return (
    <header className="flex items-end justify-between gap-3 px-4 pb-2 pt-3">
      <h1 className="text-[1.75rem] font-bold leading-tight tracking-tight text-foreground">
        {children}
      </h1>
      {trailing}
    </header>
  );
}
