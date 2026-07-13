import type { ReactNode } from "react";
import { LargeTitle } from "./LargeTitle";

// PageTitle is the care screens' header. It now renders the iOS large title
// (DESIGN.md) via LargeTitle so every existing caller upgrades at once; new
// screens that need a trailing action use LargeTitle directly.
export function PageTitle({ children }: { children: ReactNode }) {
  return <LargeTitle>{children}</LargeTitle>;
}
