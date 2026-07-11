import { PageTitle } from "../../../components/PageTitle";
import { useT } from "../../../hooks/useT";

export function MenusPage() {
  const t = useT();
  return (
    <main className="pb-20">
      <PageTitle>{t("menu.today")}</PageTitle>
    </main>
  );
}