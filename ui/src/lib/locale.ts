import en from "../locales/en.json";
import es from "../locales/es.json";

export type Locale = "en" | "es";
export type Theme = "light" | "dark" | "system";
type Catalog = Record<string, string>;

export const catalogs: Record<Locale, Catalog> = { en, es };

export function detectLocale(): Locale {
  const stored = typeof localStorage !== "undefined" ? localStorage.getItem("locale") : null;
  if (stored === "en" || stored === "es") return stored;
  if (typeof navigator !== "undefined") {
    const lang = navigator.language?.slice(0, 2);
    if (lang === "es") return "es";
  }
  return "en";
}
