import { createContext, useContext, useMemo, useState, useEffect } from "react";
import type { ReactNode } from "react";
import en from "../locales/en.json";
import es from "../locales/es.json";

export type Locale = "en" | "es";
type Catalog = Record<string, string>;
const catalogs: Record<Locale, Catalog> = { en, es };

export function detectLocale(): Locale {
  const stored = typeof localStorage !== "undefined" ? localStorage.getItem("locale") : null;
  if (stored === "en" || stored === "es") return stored;
  if (typeof navigator !== "undefined") {
    const lang = navigator.language?.slice(0, 2);
    if (lang === "es") return "es";
  }
  return "en";
}

export type Theme = "light" | "dark" | "system";
type Ctx = {
  locale: Locale;
  setLocale: (l: Locale) => void;
  t: (k: string) => string;
  theme: Theme;
  setTheme: (th: Theme) => void;
  resolvedTheme: "light" | "dark";
};

const Ctx = createContext<Ctx | null>(null);

export function LocaleProvider({ children }: { children: ReactNode }) {
  const [locale, setLocaleState] = useState<Locale>(detectLocale);
  const [theme, setThemeState] = useState<Theme>("system");
  const [systemDark, setSystemDark] = useState(false);

  useEffect(() => {
    const tStored = localStorage.getItem("theme") as Theme | null;
    if (tStored === "light" || tStored === "dark" || tStored === "system") setThemeState(tStored);
    const mq = window.matchMedia("(prefers-color-scheme: dark)");
    setSystemDark(mq.matches);
    const onChange = (e: MediaQueryListEvent) => setSystemDark(e.matches);
    mq.addEventListener("change", onChange);
    return () => mq.removeEventListener("change", onChange);
  }, []);

  const resolvedTheme: "light" | "dark" = theme === "system" ? (systemDark ? "dark" : "light") : theme;

  useEffect(() => {
    document.documentElement.classList.toggle("dark", resolvedTheme === "dark");
  }, [resolvedTheme]);

  const value = useMemo<Ctx>(() => ({
    locale,
    setLocale: (l) => { localStorage.setItem("locale", l); setLocaleState(l); },
    t: (k) => catalogs[locale][k] ?? catalogs.en[k] ?? k,
    theme,
    setTheme: (th) => { localStorage.setItem("theme", th); setThemeState(th); },
    resolvedTheme,
  }), [locale, theme, resolvedTheme]);

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

export function useTheme(): { theme: Theme; setTheme: (t: Theme) => void; resolved: "light" | "dark" } {
  const v = useContext(Ctx);
  if (!v) throw new Error("useTheme outside LocaleProvider");
  return { theme: v.theme, setTheme: v.setTheme, resolved: v.resolvedTheme };
}