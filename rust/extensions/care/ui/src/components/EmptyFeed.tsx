import { useT } from "../hooks/useT";

export function EmptyFeed() {
  const t = useT();
  return (
    <p className="px-4 py-8 text-center text-sm opacity-70">{t("feed.empty")}</p>
  );
}