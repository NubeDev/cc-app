import { createContext, useContext, useMemo, useState, useEffect } from "react";
import type { ReactNode } from "react";
import en from "../locales/en.json";
import es from "../locales/es.json";

export type Locale = "en" | "es";
type Catalog = Record<string, string>;
const catalogs: Record<Locale, Catalog> = { en, es };

export function interpolate(template: string, vars?: Record<string, unknown>): string {
  if (!vars) return template;
  return template.replace(/\{\{(\w+)\}\}/g, (_, name) => {
    return name in vars ? String(vars[name]) : `{{${name}}}`;
  });
}

export type Theme = "light" | "dark" | "system";
type Ctx = {
  locale: Locale;
  setLocale: (l: Locale) => void;
  t: (key: string, vars?: Record<string, unknown>) => string;
  theme: Theme;
  setTheme: (t: Theme) => void;
  resolvedTheme: "light" | "dark";
};

const Ctx = createContext<Ctx | null>(null);

export function LocaleProvider({ children }: { children: ReactNode }) {
  const [locale, setLocaleState] = useState<Locale>("en");
  const [theme, setThemeState] = useState<Theme>("system");
  const [systemDark, setSystemDark] = useState(false);

  useEffect(() => {
    const stored = localStorage.getItem("care.locale") as Locale | null;
    if (stored === "en" || stored === "es") setLocaleState(stored);
    const tStored = localStorage.getItem("care.theme") as Theme | null;
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
    setLocale: (l) => { localStorage.setItem("care.locale", l); setLocaleState(l); },
    t: (k, vars) => interpolate(catalogs[locale][k] ?? catalogs.en[k] ?? k, vars),
    theme,
    setTheme: (th) => { localStorage.setItem("care.theme", th); setThemeState(th); },
    resolvedTheme,
  }), [locale, theme, resolvedTheme]);

  return <Ctx.Provider value={value}>{children}</Ctx.Provider>;
}

export function useT(): (k: string, vars?: Record<string, unknown>) => string {
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