import { useParams } from "react-router-dom";
import { authApi } from "../api/auth";
import { useT } from "../hooks/useT";

export function InviteAcceptPage() {
  const t = useT();
  const { token = "" } = useParams();
  return (
    <main className="mx-auto flex min-h-screen max-w-sm flex-col justify-center px-6">
      <h1 className="mb-4 text-2xl font-semibold">{t("auth.welcome")}</h1>
      <button
        onClick={() => authApi.inviteAccept(token)}
        className="rounded bg-black py-3 text-white"
      >
        {t("auth.signIn")}
      </button>
    </main>
  );
}