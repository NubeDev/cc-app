import type { ReactNode } from "react";
import { useT } from "../hooks/useT";

interface Props { children: ReactNode; }
export function PageTitle({ children }: Props) {
  const t = useT();
  return (
    <header className="px-4 py-3 text-lg font-semibold" aria-label={t("app.title")}>
      {children}
    </header>
  );
}