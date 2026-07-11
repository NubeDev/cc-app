import en from "../locales/en.json";
import es from "../locales/es.json";

export type Locale = "en" | "es";
export type Catalog = Record<string, string>;

export const catalogs: Record<Locale, Catalog> = { en, es };

export function isLocale(x: unknown): x is Locale {
  return x === "en" || x === "es";
}