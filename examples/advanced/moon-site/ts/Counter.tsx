// Hand-written TSX, not generated. `crepus moon build` copies ts/ into the app,
// so this sits alongside the components emitted from Rust and can be imported
// from them. Interactive state belongs here today: the Rust path emits
// structural markup and does not wire event handlers.
import { useState } from "react";

export function Counter({ start = 0 }: { start?: number }) {
  const [n, setN] = useState(start);
  return (
    <button
      type="button"
      className="rounded-md border border-zinc-700 px-3 py-1 text-sm text-zinc-100"
      onClick={() => setN((v) => v + 1)}
    >
      clicked {n} times
    </button>
  );
}

export default Counter;
