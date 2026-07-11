import { useEffect, useRef } from "react";
import { useParams } from "react-router-dom";
import { mountExtension } from "@nube/ext-ui-sdk";
import { useT } from "../hooks/useT";

export function ExtMountPage() {
  const t = useT();
  const { workspaceId = "" } = useParams();
  const hostRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!hostRef.current) return;
    const unmount = mountExtension({
      container: hostRef.current,
      id: "care",
      workspaceId,
    });
    return unmount;
  }, [workspaceId]);

  return (
    <main className="min-h-screen">
      <div ref={hostRef} aria-busy="true" aria-label={t("shell.loading")} />
    </main>
  );
}