import { test, expect, describe, afterEach } from "bun:test";
import React from "react";
import { createRoot, Root } from "react-dom/client";
import { createSignal, useSignal, useStore, createStore, batch } from "../src/signal";

import { JSDOM } from "jsdom";
const dom = new JSDOM("<!DOCTYPE html><html><body><div id='root'></div></body></html>");
globalThis.window = dom.window as any;
globalThis.document = dom.window.document as any;
globalThis.navigator = dom.window.navigator;
globalThis.IS_REACT_ACT_ENVIRONMENT = true;

describe("React hooks", () => {
  let container: HTMLDivElement;
  let root: Root;

  afterEach(() => {
    if (root) {
      React.act(() => {
        root.unmount();
      });
    }
  });

  describe("useSignal", () => {
    test("renders initial value and updates on mutation", async () => {
      const count = createSignal(10);

      function App() {
        const val = useSignal(count);
        return <div id="count">{val}</div>;
      }

      container = document.createElement("div");
      root = createRoot(container);

      await React.act(async () => {
        root.render(<App />);
      });

      expect(container.querySelector("#count")?.textContent).toBe("10");

      await React.act(async () => {
        count.set(20);
      });

      expect(container.querySelector("#count")?.textContent).toBe("20");

      await React.act(async () => {
        count.set(n => n + 1);
      });

      expect(container.querySelector("#count")?.textContent).toBe("21");
    });

    test("updates properly with batched mutations", async () => {
      const count = createSignal(0);

      let renderCount = 0;
      function App() {
        renderCount++;
        const val = useSignal(count);
        return <div id="count">{val}</div>;
      }

      container = document.createElement("div");
      root = createRoot(container);

      await React.act(async () => {
        root.render(<App />);
      });

      renderCount = 0; // reset after mount

      await React.act(async () => {
        batch(() => {
          count.set(1);
          count.set(2);
          count.set(3);
        });
      });

      expect(container.querySelector("#count")?.textContent).toBe("3");
      expect(renderCount).toBe(1); // should only render once due to batching
    });

    test("unsubscribes when unmounted", async () => {
      const count = createSignal(0);
      let renderCount = 0;

      function App({ show }: { show: boolean }) {
        if (!show) return <div id="hidden">hidden</div>;
        return <Child />;
      }

      function Child() {
        renderCount++;
        const val = useSignal(count);
        return <div id="count">{val}</div>;
      }

      container = document.createElement("div");
      root = createRoot(container);

      await React.act(async () => {
        root.render(<App show={true} />);
      });

      expect(container.querySelector("#count")?.textContent).toBe("0");

      const rendersBeforeHide = renderCount;

      // Unmount Child
      await React.act(async () => {
        root.render(<App show={false} />);
      });

      expect(container.querySelector("#hidden")?.textContent).toBe("hidden");

      // Update signal - should not trigger render in Child as it's unmounted
      await React.act(async () => {
        count.set(100);
      });

      expect(renderCount).toBe(rendersBeforeHide);
    });
  });

  describe("useStore", () => {
    test("throws if passed non-store proxy", () => {
      let error: any = null;
      try {
        useStore({ plain: "object" } as any);
      } catch (e) {
        error = e;
      }
      expect(error).not.toBeNull();
      expect(error?.message).toContain("expected a createStore() proxy");
    });
  });
});
