/**
 * Light server-entry helpers (Waku-inspired).
 *
 * These are intentionally thin: crepuscularity / Bun / Next can own the
 * actual HTTP server; Moonshine only standardizes page modules and adapters.
 */

export type MoonshineRequest = {
  url: string;
  method: string;
  headers: Headers;
  pathname: string;
  searchParams: URLSearchParams;
};

export type MoonshinePageModule = {
  /** Optional RSC/SSR-friendly data loader. */
  getConfig?: () => Promise<Record<string, unknown>> | Record<string, unknown>;
  /** Render body for the path. May return a string (HTML) or opaque UI handle. */
  render: (req: MoonshineRequest) => Promise<unknown> | unknown;
};

export type MoonshineServerOptions = {
  /** Map of pathname → page module (file-ish routing table). */
  pages: Record<string, MoonshinePageModule>;
  /** Called when no page matches. */
  notFound?: (req: MoonshineRequest) => Promise<Response> | Response;
  /** Port for `Bun.serve` helper (default 3000). */
  port?: number;
};

/** Normalize a Fetch API Request into a Moonshine request bag. */
export function toMoonshineRequest(request: Request): MoonshineRequest {
  const url = new URL(request.url);
  return {
    url: request.url,
    method: request.method,
    headers: request.headers,
    pathname: url.pathname.replace(/\/+$/, "") || "/",
    searchParams: url.searchParams,
  };
}

/**
 * Resolve a page module for a pathname.
 * Supports exact keys and a trailing `/*` splat prefix.
 */
export function resolvePage(
  pages: Record<string, MoonshinePageModule>,
  pathname: string,
): MoonshinePageModule | null {
  if (pages[pathname]) return pages[pathname]!;
  if (pages["/"] && pathname === "") return pages["/"]!;

  // Longest-prefix `/*` match
  let best: { key: string; mod: MoonshinePageModule } | null = null;
  for (const [key, mod] of Object.entries(pages)) {
    if (!key.endsWith("/*")) continue;
    const prefix = key.slice(0, -1); // keep trailing /
    const base = prefix.endsWith("/") ? prefix.slice(0, -1) || "/" : prefix;
    if (pathname === base || pathname.startsWith(base === "/" ? "/" : `${base}/`)) {
      if (!best || key.length > best.key.length) best = { key, mod };
    }
  }
  return best?.mod ?? null;
}

/** Handle one request against a pages table. */
export async function handleMoonshineRequest(
  request: Request,
  options: Pick<MoonshineServerOptions, "pages" | "notFound">,
): Promise<Response> {
  const req = toMoonshineRequest(request);
  const page = resolvePage(options.pages, req.pathname);

  if (!page) {
    if (options.notFound) return options.notFound(req);
    return new Response("Not Found", { status: 404 });
  }

  const body = await page.render(req);
  if (body instanceof Response) return body;
  if (typeof body === "string") {
    return new Response(body, {
      headers: { "content-type": "text/html; charset=utf-8" },
    });
  }
  return new Response(JSON.stringify(body), {
    headers: { "content-type": "application/json; charset=utf-8" },
  });
}

export type MoonshineServer = {
  fetch: (request: Request) => Promise<Response>;
  /** Start Bun.serve when running under Bun; returns the server handle. */
  listen: () => unknown;
  port: number;
};

/**
 * Create a Waku-like server entry from a pages map.
 *
 * ```ts
 * const server = createMoonshineServer({
 *   pages: {
 *     "/": { render: () => "<h1>Home</h1>" },
 *     "/about": { render: () => "<h1>About</h1>" },
 *   },
 * });
 * server.listen();
 * ```
 */
export function createMoonshineServer(options: MoonshineServerOptions): MoonshineServer {
  const port = options.port ?? 3000;

  const fetch = (request: Request) => handleMoonshineRequest(request, options);

  return {
    fetch,
    port,
    listen() {
      if (typeof Bun === "undefined") {
        throw new Error("createMoonshineServer.listen() requires Bun");
      }
      return Bun.serve({
        port,
        fetch,
      });
    },
  };
}

/** Define a page module with inference-friendly typing. */
export function definePage(page: MoonshinePageModule): MoonshinePageModule {
  return page;
}
