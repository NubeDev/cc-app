import { PageTitle } from "../../../components/PageTitle";
import { BigButton } from "../../../components/BigButton";
import { useT } from "../../../hooks/useT";

export function AdminHomePage() {
  const t = useT();
  return (
    <main className="pb-20">
      <PageTitle>{t("nav.admin")}</PageTitle>
      <div className="space-y-2 px-4">
        <BigButton label={t("invite.create")} onClick={() => {}} />
      </div>
    </main>
  );
}