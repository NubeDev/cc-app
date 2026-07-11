import { createContext, useContext } from "react";
import { catalogs, type Locale } from "../lib/locale";

type Ctx = { locale: Locale; t: (key: string) => string };

export const LocaleContext = createContext<Ctx>({
  locale: "en",
  t: (k) => catalogs.en[k] ?? k,
});

export function useT(): (key: string) => string {
  return useContext(LocaleContext).t;
}

export function useLocale(): Locale {
  return useContext(LocaleContext).locale;
}