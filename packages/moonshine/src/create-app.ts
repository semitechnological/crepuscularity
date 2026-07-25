import { createElement, type ComponentType, type ReactNode } from "react";
import { createRoot, type Root } from "react-dom/client";

export type MoonshineAppOptions = {
  /** React root component. */
  root: ComponentType;
  /** Optional CSS class applied to the mount host. */
  className?: string;
};

export type MoonshineApp = {
  /** Mount into a DOM container (selector or element). */
  mount: (container: string | Element) => void;
  /** Unmount and dispose the React root. */
  unmount: () => void;
  /** True after a successful mount and before unmount. */
  readonly mounted: boolean;
};

/**
 * Create a lightweight Moonshine app shell.
 *
 * ```ts
 * const app = createMoonshineApp({ root: App });
 * app.mount("#app");
 * ```
 */
export function createMoonshineApp(options: MoonshineAppOptions): MoonshineApp {
  let reactRoot: Root | null = null;
  let host: Element | null = null;
  let mounted = false;

  return {
    get mounted() {
      return mounted;
    },
    mount(container) {
      if (mounted) {
        throw new Error("createMoonshineApp: already mounted");
      }

      const el =
        typeof container === "string" ? document.querySelector(container) : container;
      if (!el) {
        throw new Error(
          `createMoonshineApp: mount target not found: ${String(container)}`,
        );
      }

      host = el;
      if (options.className) {
        el.classList.add(...options.className.split(/\s+/).filter(Boolean));
      }

      reactRoot = createRoot(el);
      reactRoot.render(createElement(options.root));
      mounted = true;
    },
    unmount() {
      if (!mounted || !reactRoot) return;
      reactRoot.unmount();
      reactRoot = null;
      if (host && options.className) {
        for (const cls of options.className.split(/\s+/).filter(Boolean)) {
          host.classList.remove(cls);
        }
      }
      host = null;
      mounted = false;
    },
  };
}

/** @internal helper for tests / SSR stubs that only need an element tree. */
export function renderToNode(root: ComponentType): ReactNode {
  return createElement(root);
}
