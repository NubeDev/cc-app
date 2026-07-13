import { LocaleProvider } from "./hooks/useT";
import { TopBar } from "./components/TopBar";
import { HomePage } from "./pages/Home";

export function App() {
  return (
    <LocaleProvider>
      <TopBar />
      {/* Mobile-first single column, capped and centered on laptop so the
          product reads as a designed app — not a stretched phone (DESIGN.md
          §Layout). Admin multi-column surfaces can widen this later. */}
      <div className="mx-auto w-full max-w-2xl">
        <HomePage />
      </div>
    </LocaleProvider>
  );
}
