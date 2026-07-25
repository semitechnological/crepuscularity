# moonshine

Lightweight React framework runtime for [crepuscularity](https://github.com/tschk/crepuscularity).

Moonshine sits between a full meta-framework and a signal library:

| Inspiration | What moonshine keeps |
| --- | --- |
| **Next.js** | Shader/canvas helpers (`useFragmentShader`), RSC-friendly page hooks on the server side |
| **Waku** | Tiny server entry (`createMoonshineServer`) and `definePage` modules |
| **Solid** | Fine-grained `createSignal` / `createMemo` / `createStore` with React subscription |
| **Svelte** | Approachable API — few concepts, readable names, Bun-native tooling |

crepuscularity compiles `.crepus` → moonshine (via `@crepuscularity/moonshine`). This package is the **runtime / framework layer** those emits import.

## Install

```bash
bun add moonshine react react-dom
```

## Quick start

```tsx
import {
  createMoonshineApp,
  createSignal,
  useSignal,
  MoonshineRouter,
} from "moonshine";

const count = createSignal(0);

function Counter() {
  const n = useSignal(count);
  return (
    <button type="button" onClick={() => count.set(n + 1)}>
      {n}
    </button>
  );
}

function App() {
  return (
    <MoonshineRouter
      routes={[
        { path: "/", element: <Counter /> },
        { path: "/users/:id", element: <div>user</div> },
      ]}
      fallback={<div>404</div>}
    />
  );
}

createMoonshineApp({ root: App }).mount("#app");
```

## Signals

```ts
import { createSignal, createMemo, createStore, useSignal, useStore, batch } from "moonshine";

const [name, setName] = (() => {
  const s = createSignal("Ada");
  return [s, s.set] as const;
})();

const upper = createMemo(() => name().toUpperCase());

const [state, setState] = createStore({ user: { age: 1 } });
setState((s) => {
  s.user.age++;
});
```

In React components, call `useSignal(signal)` or `useStore(store)` so updates re-render.

## Shaders (Next-compatible pattern)

```tsx
import { useFragmentShader } from "moonshine";

function Plasma() {
  const { canvasRef } = useFragmentShader(`
    vec4 shade(vec2 uv, float t) {
      return vec4(uv, 0.5 + 0.5 * sin(t), 1.0);
    }
  `);
  return <canvas ref={canvasRef} style={{ width: "100%", height: 240 }} />;
}
```

Pass a full GLSL fragment shader, or a `shade(uv, time)` body — moonshine wraps a fullscreen triangle program (WebGL2 with WebGL1 fallback).

## Server helpers

```ts
import { createMoonshineServer, definePage } from "moonshine/server";

const server = createMoonshineServer({
  pages: {
    "/": definePage({
      render: () => "<h1>home</h1>",
    }),
  },
  port: 3000,
});

server.listen(); // Bun.serve
```

## How crepuscularity targets moonshine

1. `.crepus` templates lower to a small JSON IR (stack / text / button / sparkline / …).
2. The CLI emit step imports `@crepuscularity/moonshine`.
3. `renderCrepusIr(ir)` maps IR nodes → React elements built on moonshine primitives (signals for interactivity, router for multi-page shells, shaders for `embed` islands).
4. The resulting module mounts with `createMoonshineApp`.

```
.crepus  →  crepuscularity compiler  →  IR JSON
                                         ↓
                              @crepuscularity/moonshine
                                         ↓
                                    moonshine runtime
                                         ↓
                                      React DOM
```

## Scripts

```bash
bun test
bun run typecheck
```

## License

ISC — same as crepuscularity.
