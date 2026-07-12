import { LocaleProvider } from "./hooks/useT";
import { TopBar } from "./components/TopBar";
import { HomePage } from "./pages/Home";

export function App() {
  return (
    <LocaleProvider>
      <TopBar />
      <HomePage />
    </LocaleProvider>
  );
}