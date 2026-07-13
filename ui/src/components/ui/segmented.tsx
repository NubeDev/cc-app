import * as React from "react";
import { cn } from "@/lib/utils";

// iOS segmented control: a pill track with a selected segment. Built on real
// buttons (keyboard + ARIA intact via aria-pressed). Used for EN/ES and other
// small binary/ternary switches in bar chrome.
export interface Segment<T extends string> {
  value: T;
  label: React.ReactNode;
  ariaLabel?: string;
}

export function Segmented<T extends string>({
  segments,
  value,
  onChange,
  className,
}: {
  segments: ReadonlyArray<Segment<T>>;
  value: T;
  onChange: (v: T) => void;
  className?: string;
}) {
  return (
    <div role="group" className={cn("inline-flex items-center rounded-full bg-muted p-0.5", className)}>
      {segments.map((s) => {
        const active = s.value === value;
        return (
          <button
            key={s.value}
            type="button"
            aria-pressed={active}
            aria-label={s.ariaLabel}
            onClick={() => onChange(s.value)}
            className={cn(
              "rounded-full px-3 py-1 text-xs font-semibold transition-all duration-200 ease-out",
              "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
              active
                ? "bg-card text-foreground shadow-sm"
                : "text-muted-foreground hover:text-foreground",
            )}
          >
            {s.label}
          </button>
        );
      })}
    </div>
  );
}
