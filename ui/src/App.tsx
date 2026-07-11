import { LocaleProvider } from "./hooks/useT";
import { Shell } from "./components/Shell";
import { Router } from "./router";
import "./styles/index.css";

export function App() {
  return (
    <LocaleProvider>
      <Shell>
        <Router />
      </Shell>
    </LocaleProvider>
  );
}