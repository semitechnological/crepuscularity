// The composition point. `crepus moon build` creates this file once and then
// leaves it alone, so this is where the generated page, the generated
// components and hand-written TSX are wired together.
//
//   App     — generated from templates/page.crepus
//   Badge   — generated from templates/badge.csx
//   Footer  — generated from the inline template in src/main.rs
//   Counter — hand-written React, from ts/Counter.tsx
import { createApp } from "@tschk/moonshine/react";
import { App } from "./App";
import { Badge } from "./components/Badge";
import { Footer } from "./components/Footer";
import { Counter } from "./ts/Counter";

function Root() {
  return (
    <div className="bg-zinc-950">
      <App />
      <div className="flex flex-col items-start gap-4 px-6 pb-16">
        <Badge />
        <Counter start={0} />
        <Footer />
      </div>
    </div>
  );
}

createApp({ root: Root }).mount("#app");
