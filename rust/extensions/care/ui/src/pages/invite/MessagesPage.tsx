import { PageTitle } from "../../../components/PageTitle";
import { useT } from "../../../hooks/useT";

export function MessagesPage() {
  const t = useT();
  return (
    <main className="pb-20">
      <PageTitle>{t("nav.messages")}</PageTitle>
    </main>
  );
}