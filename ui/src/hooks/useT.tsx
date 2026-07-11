import { createContext, useContext, useMemo, useState } from "react";
import type { ReactNode } from "react";
import { catalogs, detectLocale, type Locale } from "../lib/locale";

type Ctx = { locale: Locale; setLocale: (l: Locale) => void; t: (k: string) => string };

const Ctx = createContext<Ctx | null>(null);

export function LocaleProvider({ children }: { children: ReactNode }) {
  const [locale, setLocaleState] = useState<Locale>(detectLocale);
  const value = useMemo<Ctx>(() => ({
    locale,
    setLocale: (l) => { localStorage.setItem("locale", l); setLocaleState(l); },
    t: (k) => catalogs[locale][k] ?? k,
  }), [locale]);
  return <Ctx.Provider value={value}>{children}</Ctx.Provider>;
}

export function useT(): (k: string) => string {
  const v = useContext(Ctx);
  if (!v) throw new Error("useT outside LocaleProvider");
  return v.t;
}

export function useLocaleSwitch(): { locale: Locale; setLocale: (l: Locale) => void } {
  const v = useContext(Ctx);
  if (!v) throw new Error("useLocaleSwitch outside LocaleProvider");
  return { locale: v.locale, setLocale: v.setLocale };
}