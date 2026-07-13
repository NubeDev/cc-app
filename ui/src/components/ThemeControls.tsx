import { Moon, Sun } from "lucide-react";
import { useLocaleSwitch, useTheme, useT } from "../hooks/useT";
import { Segmented } from "./ui/segmented";
import { Button } from "./ui/button";

// The shared bar-chrome controls (EN/ES segmented + light/dark toggle) that
// every pre-auth host screen carries. One component, so the login,
// invite-accept, and workspace-picker headers stay byte-identical.
export function ThemeControls() {
  const t = useT();
  const { locale, setLocale } = useLocaleSwitch();
  const { resolved, setTheme } = useTheme();
  return (
    <div className="flex items-center gap-2">
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
    </div>
  );
}
