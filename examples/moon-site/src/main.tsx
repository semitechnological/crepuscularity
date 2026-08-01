// The composition point. `crepus moon build` creates this file once and then
// leaves it alone, so this is where the generated page, the generated
// components and hand-written TSX are wired together.
//
//   App     — generated from templates/page.crepus
//   Badge   — generated from templates/badge.csx
//   Footer  — generated from the inline template in src/main.rs
//   Counter — hand-written React, from ts/Counter.tsx
//
// The generated App takes `scope` (values its expressions read) and `handlers`
// (functions its events call). State stays here in ordinary React; the template
// decides what to render from it.
import { useState } from "react";
import { createApp } from "@tschk/moonshine/react";
import { App } from "./App";
import { Badge } from "./components/Badge";
import { Footer } from "./components/Footer";
import { Counter } from "./ts/Counter";

const TAGS = ["crepus", "crepusx", "tsx"];

function Root() {
  const [count, setCount] = useState(0);

  return (
    <div className="bg-zinc-950">
      <App
        scope={{ count, tags: TAGS }}
        handlers={{ increment: () => setCount((n) => n + 1) }}
      />
      <div className="flex flex-col items-start gap-4 px-6 pb-16">
        <Badge />
        <Counter start={0} />
        <Footer />
      </div>
    </div>
  );
}

createApp({ root: Root }).mount("#app");
