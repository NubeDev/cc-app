// ext Tailwind preset (no @tailwind base / :root{} / .dark{} here — see styles/tokens.css).
import { extTailwindPreset } from "@nube/ext-ui-sdk/tailwind";

export default {
  presets: [extTailwindPreset()],
  content: ["./src/**/*.{ts,tsx}"],
};