export async function gateway<T>(path: string, init?: RequestInit): Promise<T> {
  const r = await fetch(path, {
    credentials: "include",
    headers: { "Content-Type": "application/json" },
    ...init,
  });
  if (!r.ok) throw new Error(`gateway ${path} -> ${r.status}`);
  return r.json() as Promise<T>;
}