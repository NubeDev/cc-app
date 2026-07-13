import { gateway } from "./gateway";

export interface LoginInput { email: string; password: string; }
export interface Session { sub: string; workspaceId: string; role: string; locale: string; }
/** The pre-auth invite preview (email/locale/redeemable) — no session yet. */
export interface InvitePreview { email: string; locale: string; redeemable: boolean; }

export const authApi = {
  login: (input: LoginInput) =>
    gateway<Session>("/api/auth/login", { method: "POST", body: JSON.stringify(input) }),
  logout: () => gateway<void>("/api/auth/logout", { method: "POST" }),
  /** Preview an invite before accepting (renders the accept page in the invitee's language). */
  inviteVerify: (token: string) =>
    gateway<InvitePreview>(`/api/invites/verify?token=${encodeURIComponent(token)}`),
  /** Accept an invite and SET the password: creates the account + credential, mints a session. */
  inviteAccept: (token: string, secret: string) =>
    gateway<Session>("/api/invites/accept", {
      method: "POST",
      body: JSON.stringify({ token, secret }),
    }),
};