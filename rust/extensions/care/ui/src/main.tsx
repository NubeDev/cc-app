import type { ReactNode } from "react";
import { createRoot } from "react-dom/client";
import "./styles/index.css";
import { App } from "./App";

function Root({ children }: { children: ReactNode }) {
  return <div className="mx-auto min-h-screen max-w-screen-xl bg-background text-foreground antialiased">{children}</div>;
}

const root = createRoot(document.getElementById("ext-root")!);
root.render(<Root><App /></Root>);