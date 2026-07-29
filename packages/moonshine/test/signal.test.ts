import { describe, expect, test } from "bun:test";
import {
  batch,
  createMemo,
  createSignal,
  createStore,
} from "../src/signal";
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
