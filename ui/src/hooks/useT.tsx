import { createContext, useContext, useMemo, useState, useEffect } from "react";
import type { ReactNode } from "react";
import { catalogs, detectLocale, type Locale, type Theme } from "../lib/locale";

type Ctx = {
  locale: Locale;
  setLocale: (l: Locale) => void;
  t: (k: string) => string;
  theme: Theme;
  setTheme: (th: Theme) => void;
  resolved: "light" | "dark";
};

const Ctx = createContext<Ctx | null>(null);

export function LocaleProvider({ children }: { children: ReactNode }) {
  const [locale, setLocaleState] = useState<Locale>(detectLocale);
  const [theme, setThemeState] = useState<Theme>("system");
  const [systemDark, setSystemDark] = useState(false);

  useEffect(() => {
    const tStored = typeof localStorage !== "undefined" ? localStorage.getItem("theme") as Theme | null : null;
    if (tStored === "light" || tStored === "dark" || tStored === "system") setThemeState(tStored);
    if (typeof window !== "undefined") {
      const mq = window.matchMedia("(prefers-color-scheme: dark)");
      setSystemDark(mq.matches);
      const onChange = (e: MediaQueryListEvent) => setSystemDark(e.matches);
      mq.addEventListener("change", onChange);
      return () => mq.removeEventListener("change", onChange);
    }
  }, []);

  const resolved: "light" | "dark" = theme === "system" ? (systemDark ? "dark" : "light") : theme;

  useEffect(() => {
    document.documentElement.classList.toggle("dark", resolved === "dark");
  }, [resolved]);

  const value = useMemo<Ctx>(() => ({
    locale,
    setLocale: (l) => { localStorage.setItem("locale", l); setLocaleState(l); },
    t: (k) => catalogs[locale][k] ?? k,
    theme,
    setTheme: (th) => { localStorage.setItem("theme", th); setThemeState(th); },
    resolved,
  }), [locale, theme, resolved]);
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

export function useTheme(): { theme: Theme; setTheme: (th: Theme) => void; resolved: "light" | "dark" } {
  const v = useContext(Ctx);
  if (!v) throw new Error("useTheme outside LocaleProvider");
  return { theme: v.theme, setTheme: v.setTheme, resolved: v.resolved };
}