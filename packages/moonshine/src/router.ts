import {
  createContext,
  createElement,
  useContext,
  useEffect,
  useState,
  useSyncExternalStore,
  type ReactNode,
} from "react";
import { createSignal } from "./signal";

export type RouteParams = Record<string, string>;

export type RouteMatch = {
  path: string;
  pattern: string;
  params: RouteParams;
};

export type RouteDefinition = {
  /** Path pattern, e.g. `/`, `/about`, `/users/:id`. */
  path: string;
  element: ReactNode;
};

/** Compile `/users/:id` → matcher with named params. */
export function matchPath(pattern: string, pathname: string): RouteMatch | null {
  const patternParts = splitPath(pattern);
  const pathParts = splitPath(pathname);

  if (patternParts.length !== pathParts.length) return null;

  const params: RouteParams = {};
  for (let i = 0; i < patternParts.length; i++) {
    const expected = patternParts[i]!;
    const actual = pathParts[i]!;
    if (expected.startsWith(":")) {
      params[expected.slice(1)] = decodeURIComponent(actual);
      continue;
    }
    if (expected !== actual) return null;
  }

  return { path: pathname, pattern, params };
}

function splitPath(path: string): string[] {
  return path
    .replace(/\/+$/, "")
    .replace(/^\/+/, "")
    .split("/")
    .filter(Boolean);
}

/** Resolve first matching route definition. */
export function matchRoutes(
  routes: RouteDefinition[],
  pathname: string,
): (RouteMatch & { element: ReactNode }) | null {
  for (const route of routes) {
    const match = matchPath(route.path, pathname);
    if (match) return { ...match, element: route.element };
  }
  return null;
}

const locationSignal = createSignal(getBrowserPath());

function getBrowserPath(): string {
  if (typeof window === "undefined") return "/";
  return window.location.pathname || "/";
}

function syncFromBrowser(): void {
  locationSignal.set(getBrowserPath());
}

/** Navigate via History API (no full reload). */
export function navigate(to: string, options?: { replace?: boolean }): void {
  if (typeof window === "undefined") {
    locationSignal.set(to);
    return;
  }
  const url = new URL(to, window.location.origin);
  if (options?.replace) {
    window.history.replaceState({}, "", url.pathname + url.search + url.hash);
  } else {
    window.history.pushState({}, "", url.pathname + url.search + url.hash);
  }
  locationSignal.set(url.pathname);
}

/** Current pathname (reactive signal). */
export function getLocation(): string {
  return locationSignal();
}

export function useLocation(): string {
  return useSyncExternalStore(
    locationSignal.subscribe,
    () => locationSignal.peek(),
    () => "/",
  );
}

type RouterContextValue = {
  match: (RouteMatch & { element: ReactNode }) | null;
  params: RouteParams;
  navigate: typeof navigate;
};

const RouterContext = createContext<RouterContextValue | null>(null);

export function useParams(): RouteParams {
  return useContext(RouterContext)?.params ?? {};
}

export function useNavigate(): typeof navigate {
  return useContext(RouterContext)?.navigate ?? navigate;
}

export type MoonshineRouterProps = {
  routes: RouteDefinition[];
  /** Fallback when no route matches. */
  fallback?: ReactNode;
  /** Controlled path (defaults to browser location). */
  path?: string;
};

/**
 * Minimal file-ish router: declarative routes with `:param` segments.
 *
 * ```tsx
 * <MoonshineRouter
 *   routes={[
 *     { path: "/", element: <Home /> },
 *     { path: "/users/:id", element: <User /> },
 *   ]}
 *   fallback={<NotFound />}
 * />
 * ```
 */
export function MoonshineRouter(props: MoonshineRouterProps): ReactNode {
  const browserPath = useLocation();
  const pathname = props.path ?? browserPath;
  const [ready, setReady] = useState(typeof window === "undefined");

  useEffect(() => {
    syncFromBrowser();
    setReady(true);
    const onPop = () => syncFromBrowser();
    window.addEventListener("popstate", onPop);
    return () => window.removeEventListener("popstate", onPop);
  }, []);

  const matched = matchRoutes(props.routes, pathname);
  const value: RouterContextValue = {
    match: matched,
    params: matched?.params ?? {},
    navigate,
  };

  if (!ready && props.path === undefined && typeof window !== "undefined") {
    // Avoid hydration flicker on first client paint before popstate sync
  }

  return createElement(
    RouterContext.Provider,
    { value },
    matched?.element ?? props.fallback ?? null,
  );
}

/** Anchor that uses client-side navigation. */
export function Link(props: {
  href: string;
  children?: ReactNode;
  className?: string;
  replace?: boolean;
}): ReactNode {
  return createElement(
    "a",
    {
      href: props.href,
      className: props.className,
      onClick: (event: MouseEvent) => {
        if (
          event.defaultPrevented ||
          event.button !== 0 ||
          event.metaKey ||
          event.altKey ||
          event.ctrlKey ||
          event.shiftKey
        ) {
          return;
        }
        event.preventDefault();
        navigate(props.href, { replace: props.replace });
      },
    },
    props.children,
  );
}
