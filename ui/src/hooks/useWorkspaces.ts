import { useEffect, useState } from "react";
import { gateway } from "../api/gateway";

export interface Workspace { id: string; name: string; role: string; }

export function useWorkspaces() {
  const [data, setData] = useState<Workspace[] | null>(null);
  const [error, setError] = useState<Error | null>(null);
  useEffect(() => {
    gateway<Workspace[]>("/api/me/workspaces")
      .then(setData)
      .catch(setError);
  }, []);
  return { data, error };
}