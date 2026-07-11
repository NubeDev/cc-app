import { useMcpClient } from "@nube/ext-ui-sdk/runtime";

export function useCareApi() {
  const call = useMcpClient();
  return {
    list: <T>(verb: string, args?: unknown) =>
      call<T[]>(`care.${verb}.list`, args ?? {}),
    get: <T>(verb: string, id: string) => call<T>(`care.${verb}.get`, { id }),
    run: <T>(verb: string, args: unknown) => call<T>(`care.${verb}`, args),
  };
}