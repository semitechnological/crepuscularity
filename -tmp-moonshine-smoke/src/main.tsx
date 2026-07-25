import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { renderCrepusIr } from "@tschk/crepus-moonshine";
import type { CrepusIr } from "@tschk/crepus-moonshine";
// When @tschk/crepus-moonshine exports `ViewIr`, prefer: import type { ViewIr } from "@tschk/crepus-moonshine";
import { Sparkline } from "@tschk/moonshine-components";
import "./app.css";

const sampleIr = {
  version: 1,
  root: [
    {
      kind: "stack",
      axis: "column",
      gap: 16,
      children: [
        {
          kind: "text",
          content: "/tmp/moonshine-smoke",
          style: { fontSize: 28, fontWeight: 700 },
        },
        {
          kind: "text",
          content: "Moonshine + Crepuscularity View IR",
          style: { opacity: 0.7 },
        },
        {
          kind: "badge",
          label: "renderCrepusIr",
          tone: "accent",
        },
        {
          kind: "button",
          label: "Ping",
          onClick: "ping",
        },
      ],
    },
  ],
} as const satisfies CrepusIr;

function App() {
  return (
    <main className="shell">
      {renderCrepusIr(sampleIr, {
        onAction: (handler) => console.log("action:", handler),
      })}
      <section className="spark">
        <h2>Sparkline</h2>
        <Sparkline values={[2, 4, 3, 7, 5, 9, 6, 8, 10, 7]} color="blue" height={56} />
      </section>
    </main>
  );
}

const el = document.getElementById("app");
if (!el) throw new Error("#app missing");
createRoot(el).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
