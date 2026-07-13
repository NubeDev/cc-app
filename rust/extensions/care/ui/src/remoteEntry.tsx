// remoteEntry.tsx — the ONE file that defines what this extension exports
// to the host. A single `defineRemote(...)` from @nube/ext-ui-sdk.
// CLAUDE.md rule 6: never hand-write mount/mountWidget/createRoot or inject
// styles into document.head. SDK owns scoped mount + React root + widget
// dispatch + CSS isolation.
//
// The SDK contract (see `RemoteDef` in the ext-ui-sdk): `styles` is a CSS
// STRING (imported `?inline`), `page`/`widgets` are RENDER FUNCTIONS
// `(ctx, bridge) => ReactNode` — NOT lazy `() => import(...)` thunks (those
// resolve to Promises, which React can't render — "[object Promise]"). The
// SDK already wraps the tree in `RuntimeProvider` (so `useSession` resolves);
// the ext wraps its own page in `LocaleProvider` (the i18n/theme context
// `useT`/`useTheme` read).
//
// Pair: extTailwindPreset() in tailwind.config.ts, tokens.css with no
// @tailwind base/:root{}/.dark{}.

import { defineRemote } from "@nube/ext-ui-sdk";
import styles from "./styles/tokens.css?inline";
import { LocaleProvider } from "./hooks/useT";
import { HomePage } from "./pages/Home";
import { NextMealWidget } from "./widgets/NextMealWidget";
import { AttendanceBadgeWidget } from "./widgets/AttendanceBadgeWidget";

export default defineRemote({
  id: "care",
  styles,
  page: () => (
    <LocaleProvider>
      <HomePage />
    </LocaleProvider>
  ),
  widgets: {
    // Keyed by the manifest `[[widget]]` slug the shell passes; each tile owns
    // its own LocaleProvider so a widget mounted standalone still translates.
    "next-meal": (ctx) => (
      <LocaleProvider>
        <NextMealWidget childId={(ctx.binding?.childId as string) ?? ""} />
      </LocaleProvider>
    ),
    "attendance-badge": () => (
      <LocaleProvider>
        <AttendanceBadgeWidget />
      </LocaleProvider>
    ),
  },
});
