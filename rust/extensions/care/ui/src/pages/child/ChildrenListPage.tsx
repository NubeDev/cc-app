import { useCareApi } from "../../../api/care";
import { PageTitle } from "../../../components/PageTitle";
import { useT } from "../../../hooks/useT";

export function ChildrenListPage() {
  const api = useCareApi();
  const t = useT();
  void api.list("child");
  return (
    <main className="pb-20">
      <PageTitle>{t("nav.children")}</PageTitle>
    </main>
  );
}