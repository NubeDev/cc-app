import { useT } from "../hooks/useT";

export function OfflinePage() {
  const t = useT();
  return (
    <main className="mx-auto flex min-h-screen max-w-sm flex-col justify-center px-6 text-center">
      <p>{t("error.offline")}</p>
    </main>
  );
}