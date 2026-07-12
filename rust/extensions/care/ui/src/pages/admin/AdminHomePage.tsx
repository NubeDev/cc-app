import { useState } from "react";
import { PageTitle } from "../../components/PageTitle";
import { useT } from "../../hooks/useT";
import { CentersRoomsPage } from "./CentersRoomsPage";
import { ChildrenListPage } from "../child/ChildrenListPage";
import { EnrollmentListPage } from "../enrollment/EnrollmentPage";
import { GuardiansListPage } from "../guardian/GuardiansAndEdgesPage";

type Tab = "schools" | "children" | "enrollment" | "guardians";

export function AdminHomePage() {
  const t = useT();
  const [tab, setTab] = useState<Tab>("schools");
  const tabs: Array<{ key: Tab; label: string }> = [
    { key: "schools", label: t("admin.schools") },
    { key: "children", label: t("nav.children") },
    { key: "enrollment", label: t("admin.enrollment") },
    { key: "guardians", label: t("admin.guardians") },
  ];
  return (
    <div>
      <PageTitle>{t("admin.title")}</PageTitle>
      <nav className="sticky top-0 z-10 -mx-4 border-b border-border bg-background/95 px-4 backdrop-blur">
        <div className="-mb-px flex gap-1 overflow-x-auto">
          {tabs.map((tt) => (
            <button
              key={tt.key}
              onClick={() => setTab(tt.key)}
              className={`shrink-0 border-b-2 px-3 py-2.5 text-sm font-medium transition ${tab === tt.key ? "border-primary text-foreground" : "border-transparent text-muted-foreground"}`}
            >
              {tt.label}
            </button>
          ))}
        </div>
      </nav>
      {tab === "schools" && <CentersRoomsPage />}
      {tab === "children" && <ChildrenListPage />}
      {tab === "enrollment" && <EnrollmentListPage />}
      {tab === "guardians" && <GuardiansListPage />}
    </div>
  );
}