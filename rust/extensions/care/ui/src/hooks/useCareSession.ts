import { useSession } from "@nube/ext-ui-sdk/runtime";

export type Role = "admin" | "staff" | "guardian" | "kiosk";

export interface CareSession {
  sub: string;
  workspaceId: string;
  role: Role;
  locale: "en" | "es";
}

export function useCareSession(): CareSession | null {
  const s = useSession<CareSession>();
  return s ?? null;
}