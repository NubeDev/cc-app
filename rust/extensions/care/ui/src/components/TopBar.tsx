import { useLocaleSwitch, useTheme } from "../hooks/useT";

export function TopBar() {
  const { locale, setLocale } = useLocaleSwitch();
  const { resolved, setTheme } = useTheme();
  return (
    <header className="sticky top-0 z-30 flex items-center justify-end gap-2 border-b border-border bg-background/80 px-4 py-2 backdrop-blur">
      <div className="flex overflow-hidden rounded-full border border-border bg-card text-xs">
        <button
          aria-pressed={locale === "en"}
          onClick={() => setLocale("en")}
          className={`px-3 py-1.5 font-medium transition ${locale === "en" ? "bg-primary text-primary-foreground" : "text-muted-foreground"}`}
        >EN</button>
        <button
          aria-pressed={locale === "es"}
          onClick={() => setLocale("es")}
          className={`px-3 py-1.5 font-medium transition ${locale === "es" ? "bg-primary text-primary-foreground" : "text-muted-foreground"}`}
        >ES</button>
      </div>
      <button
        onClick={() => setTheme(resolved === "dark" ? "light" : "dark")}
        className="flex h-8 w-8 items-center justify-center rounded-full border border-border bg-card text-sm transition"
        aria-label="Toggle theme"
      >
        {resolved === "dark" ? "☼" : "☾"}
      </button>
    </header>
  );
}