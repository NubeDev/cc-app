import { Moon, Sun } from "lucide-react";
import { useLocaleSwitch, useTheme, useT } from "../hooks/useT";
import { Segmented } from "./ui/segmented";
import { Button } from "./ui/button";

// The translucent top bar chrome (DESIGN.md §Shape & Depth: backdrop-blur is
// the one sanctioned blur). Carries the EN/ES + light/dark controls that
// propagate through the host theme seam.
export function TopBar() {
  const t = useT();
  const { locale, setLocale } = useLocaleSwitch();
  const { resolved, setTheme } = useTheme();
  return (
    <header className="sticky top-0 z-30 flex items-center justify-end gap-2 border-b border-border/70 bg-background/70 px-4 py-2 backdrop-blur-xl">
      <Segmented
        value={locale}
        onChange={setLocale}
        segments={[
          { value: "en", label: "EN" },
          { value: "es", label: "ES" },
        ]}
      />
      <Button
        variant="outline"
        size="icon"
        onClick={() => setTheme(resolved === "dark" ? "light" : "dark")}
        aria-label={t("shell.theme.toggle")}
      >
        {resolved === "dark" ? <Sun /> : <Moon />}
      </Button>
    </header>
  );
}
