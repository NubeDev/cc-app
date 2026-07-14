import { createContext, useContext, useMemo, useState, useEffect } from "react";
import type { ReactNode } from "react";
import { useSession } from "@nube/ext-ui-sdk/runtime";
import type { CareSession } from "./useCareSession";
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
  // The host session is the SOURCE of the locale: the shell resolves it (invite
  // locale → guardian preference → center default) and forwards it through the
  // mount ctx, which the SDK surfaces as `useSession().locale`. The ext MUST
  // follow it, else a Spanish guardian sees an English product (CLAUDE.md rule
  // 8). `LocaleProvider` sits inside the SDK's `RuntimeProvider`, so
  // `useSession()` resolves here. `care.locale` in localStorage is only ever an
  // EXPLICIT in-ext override (the EN/ES toggle) — absent that, session wins, and
  // a later host-locale change re-syncs.
  const session = useSession<CareSession>();
  const sessionLocale: Locale = session?.locale === "es" ? "es" : "en";

  const [locale, setLocaleState] = useState<Locale>(() => {
    const stored = typeof localStorage !== "undefined" ? localStorage.getItem("care.locale") : null;
    if (stored === "en" || stored === "es") return stored;
    return sessionLocale;
  });
  const [theme, setThemeState] = useState<Theme>("system");
  const [systemDark, setSystemDark] = useState(false);

  // The host EN/ES toggle is authoritative: when it changes, it remounts this
  // provider with a fresh `session.locale`, and we adopt it — clearing any prior
  // in-ext override so the host switch always wins. (The in-ext toggle still
  // works within a single mount; a host flip resets to the host's choice.)
  useEffect(() => {
    localStorage.setItem("care.locale", sessionLocale);
    setLocaleState(sessionLocale);
  }, [sessionLocale]);

  useEffect(() => {
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