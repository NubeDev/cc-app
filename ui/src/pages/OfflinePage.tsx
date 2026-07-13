import { CloudOff } from "lucide-react";
import { useT } from "../hooks/useT";
import { Button } from "../components/ui/button";

export function OfflinePage() {
  const t = useT();
  return (
    <main className="mx-auto flex min-h-[100dvh] max-w-sm flex-col items-center justify-center gap-4 px-6 text-center">
      <span className="flex size-16 items-center justify-center rounded-full bg-muted text-muted-foreground">
        <CloudOff className="size-7" aria-hidden />
      </span>
      <h1 className="text-xl font-semibold tracking-tight text-foreground">{t("error.offline.title")}</h1>
      <p className="text-[15px] leading-relaxed text-muted-foreground">{t("error.offline")}</p>
      <Button variant="outline" onClick={() => window.location.reload()} className="mt-2">
        {t("common.retry")}
      </Button>
    </main>
  );
}
