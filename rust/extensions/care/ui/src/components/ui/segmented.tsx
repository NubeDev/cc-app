import * as React from "react";
import { cn } from "../../lib/cn";

// iOS segmented control: a pill track with a selected segment on real buttons
// (keyboard + aria-pressed intact). Used for locale, relationship, status, and
// enrollment-status pickers where a small closed set beats a dropdown.
export interface Segment<T extends string> {
  value: T;
  label: React.ReactNode;
}

export function Segmented<T extends string>({
  segments,
  value,
  onChange,
  className,
  columns,
}: {
  segments: ReadonlyArray<Segment<T>>;
  value: T;
  onChange: (v: T) => void;
  className?: string;
  columns?: number;
}) {
  return (
    <div
      role="group"
      className={cn("gap-0.5 rounded-xl bg-muted p-0.5", columns ? "grid" : "inline-flex", className)}
      style={columns ? { gridTemplateColumns: `repeat(${columns}, minmax(0, 1fr))` } : undefined}
    >
      {segments.map((s) => {
        const active = s.value === value;
        return (
          <button
            key={s.value}
            type="button"
            aria-pressed={active}
            onClick={() => onChange(s.value)}
            className={cn(
              "rounded-[0.625rem] px-3 py-1.5 text-sm font-medium transition-all duration-200 ease-out",
              "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
              active ? "bg-card text-foreground shadow-sm" : "text-muted-foreground hover:text-foreground",
            )}
          >
            {s.label}
          </button>
        );
      })}
    </div>
  );
}
