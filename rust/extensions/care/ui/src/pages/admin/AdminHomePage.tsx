import { useState } from "react";
import { LargeTitle } from "../../components/LargeTitle";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "../../components/ui/tabs";
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
    <div className="pb-24">
      <LargeTitle>{t("admin.title")}</LargeTitle>
      <Tabs value={tab} onValueChange={(v) => setTab(v as Tab)}>
        <div className="sticky top-12 z-10 bg-background/70 px-4 pb-2 pt-1 backdrop-blur-xl">
          <TabsList className="w-full">
            {tabs.map((tt) => (
              <TabsTrigger key={tt.key} value={tt.key} className="flex-1">
                {tt.label}
              </TabsTrigger>
            ))}
          </TabsList>
        </div>
        <TabsContent value="schools"><CentersRoomsPage embedded /></TabsContent>
        <TabsContent value="children"><ChildrenListPage embedded /></TabsContent>
        <TabsContent value="enrollment"><EnrollmentListPage embedded /></TabsContent>
        <TabsContent value="guardians"><GuardiansListPage embedded /></TabsContent>
      </Tabs>
    </div>
  );
}
