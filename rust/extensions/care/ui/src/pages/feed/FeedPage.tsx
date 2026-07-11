import { useCareApi } from "../../api/care";
import { PageTitle } from "../../components/PageTitle";
import { useT } from "../../hooks/useT";

export function FeedPage() {
  const api = useCareApi();
  const t = useT();
  // Real implementation: subscribe via care.feed.watch (milestone 08).
  void api;
  return (
    <main className="pb-20">
      <PageTitle>{t("nav.feed")}</PageTitle>
    </main>
  );
}