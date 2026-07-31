import { describe, expect, test } from "bun:test";
import {
  batch,
  createMemo,
  createSignal,
  createStore,
  useStore,
} from "../src/signal";
import { matchPath, matchRoutes } from "../src/router";
import {
  createFullscreenFragment,
  wrapFragmentSource,
} from "../src/shaders";
import {
  createMoonshineServer,
  definePage,
  handleMoonshineRequest,
  resolvePage,
} from "../src/server";
import { renderHook, act } from "@testing-library/react";

describe("createSignal", () => {
  test("reads and writes", () => {
    const count = createSignal(0);
    expect(count()).toBe(0);
    count.set(1);
    expect(count()).toBe(1);
    count.set((n) => n + 1);
    expect(count()).toBe(2);
  });

  test("notifies subscribers", () => {
    const count = createSignal(0);
    let hits = 0;
    const unsub = count.subscribe(() => {
      hits++;
    });
    count.set(1);
    count.set(1); // same value — no notify
    count.set(2);
    unsub();
    count.set(3);
    expect(hits).toBe(2);
  });

  test("batch coalesces notifications", () => {
    const a = createSignal(0);
    let hits = 0;
    a.subscribe(() => {
      hits++;
    });
    batch(() => {
      a.set(1);
      a.set(2);
      a.set(3);
    });
    expect(hits).toBe(1);
    expect(a()).toBe(3);
  });
});

describe("createMemo", () => {
  test("derives and updates", () => {
    const n = createSignal(2);
    const doubled = createMemo(() => n() * 2);
    expect(doubled()).toBe(4);
    n.set(5);
    expect(doubled()).toBe(10);
  });

  test("notifies when dependency changes", () => {
    const n = createSignal(1);
    const m = createMemo(() => n() + 10);
    let hits = 0;
    m.subscribe(() => {
      hits++;
    });
    n.set(2);
    expect(m()).toBe(12);
    expect(hits).toBeGreaterThanOrEqual(1);
  });
});

describe("createStore", () => {
  test("nested mutations notify", () => {
    const [state, setState] = createStore({ user: { name: "Ada", age: 1 } });
    let hits = 0;
    // Access through proxy then subscribe via a signal bridge:
    // store notifies on mutation — listen by wrapping a read in a memo.
    const age = createMemo(() => state.user.age);
    age.subscribe(() => {
      hits++;
    });

    setState((s) => {
      s.user.age = 2;
    });
    expect(state.user.age).toBe(2);
    expect(age()).toBe(2);
    expect(hits).toBeGreaterThanOrEqual(1);

    state.user.name = "Grace";
    expect(state.user.name).toBe("Grace");
  });
});

describe("useStore", () => {
  test("throws on invalid store", () => {
    expect(() => {
      // @ts-ignore
      useStore({ invalid: true });
    }).toThrow("useStore: expected a createStore() proxy");
  });

  test("wires up proxy to sync external store", async () => {
    const [store, setStore] = createStore({ value: 0 });

    let renderCount = 0;
    let hookValue = 0;

    // 1) Setup a happy-dom environment to mock window/document for React tests
    const { Window } = require('happy-dom');
    const window = new Window();
    const document = window.document;

    // Polyfill for React to run in node-like environment
    const origWindow = global.window;
    const origDocument = global.document;
    const origNavigator = global.navigator;

    try {
      // @ts-ignore
      global.window = window;
      // @ts-ignore
      global.document = document;
      // @ts-ignore
      global.navigator = window.navigator;

      // We wrap the hook execution in renderHook which simulates a real React component environment
      // where useSyncExternalStore is allowed
      const { result } = renderHook(() => {
        renderCount++;
        const s = useStore(store);
        hookValue = s.value;
        return s;
      });

      expect(result.current.value).toBe(0);
      const initialRenderCount = renderCount;

      await act(async () => {
        setStore(s => {
          s.value = 42;
        });
      });

      // The true test of useStore: Did updating the store cause a re-render?
      expect(renderCount).toBeGreaterThan(initialRenderCount);

      // The value should be updated
      expect(store.value).toBe(42);
      expect(hookValue).toBe(42);
    } finally {
      // Restore globals
      global.window = origWindow;
      global.document = origDocument;
      global.navigator = origNavigator;
    }
  });
});

describe("router", () => {
  test("matchPath extracts params", () => {
    const m = matchPath("/users/:id", "/users/42");
    expect(m).not.toBeNull();
    expect(m!.params.id).toBe("42");
  });

  test("matchPath rejects length mismatch", () => {
    expect(matchPath("/a/b", "/a")).toBeNull();
  });

  test("matchRoutes picks first hit", () => {
    const hit = matchRoutes(
      [
        { path: "/", element: "home" },
        { path: "/about", element: "about" },
        { path: "/users/:id", element: "user" },
      ],
      "/users/7",
    );
    expect(hit?.element).toBe("user");
    expect(hit?.params.id).toBe("7");
  });
});

describe("shaders", () => {
  test("wrapFragmentSource wraps shade() bodies", () => {
    const src = wrapFragmentSource(
      `vec4 shade(vec2 uv, float t) { return vec4(uv, 0.5, 1.0); }`,
      true,
    );
    expect(src).toContain("#version 300 es");
    expect(src).toContain("void main");
    expect(src).toContain("shade(uv, u_time)");
  });

  test("createFullscreenFragment returns vertex + fragment", () => {
    const prog = createFullscreenFragment(
      `vec4 shade(vec2 uv, float t) { return vec4(1.0); }`,
    );
    expect(prog.vertex).toContain("gl_Position");
    expect(prog.fragment).toContain("shade");
  });

  test("full shaders pass through", () => {
    const full = `#version 300 es
precision highp float;
out vec4 c;
void main() { c = vec4(1.0); }
`;
    expect(wrapFragmentSource(full, true)).toBe(full);
  });
});

describe("server", () => {
  test("resolvePage exact and splat", () => {
    const pages = {
      "/": definePage({ render: () => "home" }),
      "/blog/*": definePage({ render: () => "blog" }),
    };
    expect(resolvePage(pages, "/")?.render({} as never)).toBe("home");
    expect(resolvePage(pages, "/blog/a")?.render({} as never)).toBe("blog");
    expect(resolvePage(pages, "/missing")).toBeNull();
  });

  test("handleMoonshineRequest returns html", async () => {
    const server = createMoonshineServer({
      pages: {
        "/": definePage({ render: () => "<h1>ok</h1>" }),
      },
    });
    const res = await server.fetch(new Request("http://localhost/"));
    expect(res.status).toBe(200);
    expect(await res.text()).toBe("<h1>ok</h1>");
  });

  test("handleMoonshineRequest 404", async () => {
    const res = await handleMoonshineRequest(new Request("http://localhost/x"), {
      pages: {},
    });
    expect(res.status).toBe(404);
  });
});
