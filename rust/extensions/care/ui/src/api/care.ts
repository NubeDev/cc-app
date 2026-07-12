import { useMcpClient } from "@nube/ext-ui-sdk/runtime";

export function useCareApi() {
  const call = useMcpClient();
  return {
    list: <T>(verb: string, args?: unknown) =>
      call<T[]>(`care.${verb}.list`, args ?? {}),
    get: <T>(verb: string, id: string) => call<T>(`care.${verb}.get`, { id }),
    /**
     * Call a `care.*` verb that doesn't follow the `<noun>.<action>`
     * shape (`create` / `update` / `archive` / `link` / `unlink` etc).
     * Pass the FULL verb including the action (e.g. `"center.create"`).
     */
    run: <T>(verb: string, args: unknown) => call<T>(`care.${verb}`, args),
  };
}