import { gateway } from "./gateway";

export interface LoginInput { email: string; password: string; }
export interface Session { sub: string; workspaceId: string; role: string; locale: string; }

export const authApi = {
  login: (input: LoginInput) =>
    gateway<Session>("/api/auth/login", { method: "POST", body: JSON.stringify(input) }),
  logout: () => gateway<void>("/api/auth/logout", { method: "POST" }),
  inviteAccept: (token: string) =>
    gateway<Session>("/api/invites/accept", { method: "POST", body: JSON.stringify({ token }) }),
};