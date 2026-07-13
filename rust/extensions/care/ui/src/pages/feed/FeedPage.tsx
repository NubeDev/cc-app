import { useCareApi } from "../../api/care";
import { LargeTitle } from "../../components/LargeTitle";
import { useT } from "../../hooks/useT";

export function FeedPage() {
  const api = useCareApi();
  const t = useT();
  // Real implementation: subscribe via care.feed.watch (milestone 08).
  void api;
  return (
    <main className="pb-24">
      <LargeTitle>{t("nav.feed")}</LargeTitle>
      <p className="px-4 py-16 text-center text-[15px] text-muted-foreground">{t("feed.empty")}</p>
    </main>
  );
}
