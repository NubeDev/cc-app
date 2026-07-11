// remoteEntry.tsx — the ONE file that defines what this extension exports
// to the host. A single `defineRemote(...)` from @nube/ext-ui-sdk.
// CLAUDE.md rule 6: never hand-write mount/mountWidget/createRoot or inject
// styles into document.head. SDK owns scoped mount + React root + widget
// dispatch + CSS isolation.
//
// Pair: extTailwindPreset() in tailwind.config.ts, tokens.css with no
// @tailwind base/:root{}/.dark{}.

import { defineRemote } from "@nube/ext-ui-sdk";

export default defineRemote({
  id: "care",
  styles: () => import("./styles/tokens.css"),
  page: () => import("./pages/Home"),
  widgets: () => [
    import("./widgets/NextMealWidget"),
    import("./widgets/AttendanceBadgeWidget"),
  ],
});