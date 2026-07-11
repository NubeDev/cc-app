import { PageTitle } from "../../components/PageTitle";
import { useCareSession } from "../../hooks/useCareSession";
import { useT } from "../../hooks/useT";

export function HomePage() {
  const t = useT();
  const session = useCareSession();
  return (
    <main className="pb-20">
      <PageTitle>{t("app.title")}</PageTitle>
      <p className="px-4 text-sm opacity-70">
        {session ? `${session.role} · ${session.workspaceId}` : t("auth.signIn")}
      </p>
    </main>
  );
}