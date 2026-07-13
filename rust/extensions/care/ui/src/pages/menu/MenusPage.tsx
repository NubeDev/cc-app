import { LargeTitle } from "../../components/LargeTitle";
import { useT } from "../../hooks/useT";

export function MenusPage() {
  const t = useT();
  return (
    <main className="pb-24">
      <LargeTitle>{t("menu.today")}</LargeTitle>
      <p className="px-4 py-16 text-center text-[15px] text-muted-foreground">{t("feed.empty")}</p>
    </main>
  );
}
