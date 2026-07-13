// ext Tailwind preset (no @tailwind base / :root{} / .dark{} here — see styles/tokens.css).
import { extTailwindPreset } from "@nube/ext-ui-sdk/tailwind";
import tailwindcssAnimate from "tailwindcss-animate";

export default {
  presets: [extTailwindPreset()],
  content: ["./src/**/*.{ts,tsx}"],
  plugins: [tailwindcssAnimate],
};