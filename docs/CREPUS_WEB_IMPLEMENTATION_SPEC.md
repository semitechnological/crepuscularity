# Crepuscularity — runtime & web framework: master build plan

**Audience:** Engineers or AI agents shipping the **`.crepus` runtime**, **web + webext** targets, **compiler driver**, and **dev/prod toolchains**.  
**Product intent:** **Maximum practical speed**—fast dev feedback, fast SSR/SSG, fast client updates, fast CI—using the same techniques as the best stacks (fine-grained reactivity, incremental compilation, aggressive caching, parallel task graphs).

**How to read this doc**

| Section | Purpose |
|---------|---------|
| **§0** | **Master plan** — pillars, Turbo/Next lessons, **§0.7 full index** of frameworks & techniques, optimization checklist |
| **§1–§3** | Repo state, goals, parallelism layers |
| **§4–§7** | Deep dives: Leptos, Dioxus, Sanic, Bun/Solid |
| **§8** | Web frameworks **by language** + Next/Turbo (**§8.10**) + infra (**§8.11**) + extras (**§8.12**) |
| **§9** | Browser extensions (MV3) |
| **§10–§13** | Target architecture, driver cache, tests, open choices |
| **§14** | **Phased implementation** (execute in order) |
| **§15–§16** | References, other docs in `docs/` |

Borrow **mechanisms**; do not fork Next.js or Turbopack.

---

## 0. Master build plan — what we are building and how it stays fast

### 0.1 Outcomes (define “done”)

| Surface | Dev | Prod |
|---------|-----|------|
| **Author** | Edit `.crepus` → visible update in **≪1s** without `rustc` when markup-only; Rust edits → **incremental** `cargo` only | — |
| **SSR / SSG** | Hot path **traced** (parse/eval/emit) | **Streamed** or batched HTML; **Rayon** over independent pages/components on host |
| **Client (WASM)** | Optional fast refresh for islands | **Fine-grained** DOM updates (no default full-template stringify); **`-O3`**, optional LTO/PGO |
| **CI** | — | **Cacheable** steps: fingerprinted codegen, optional **remote** cache (Turborepo-style); matrices **parallel** |
| **Webext** | Same parse cache as web where **CSP** allows | Popup/options/sidepanel **same reactive story** as web; SW stays **thin** |

### 0.2 Architectural pillars (must all land)

1. **Single AST core** (`crepuscularity-core`) — parsers + eval; no forked template semantics per target.  
2. **Incremental **driver**** — treat compile pipeline like a **query system**: parse → bind analysis → per-target emit; **invalidate minimal** subgraph when a file or section changes (Turbopack **lesson**, §0.4).  
3. **Static vs dynamic split** — every template → **frozen shape** + **binding sites** (Dioxus `Template` + Svelte **lesson**).  
4. **Reactive client graph** — signals / memos / effects (Leptos / Solid **lesson**); effects batch DOM writes.  
5. **Hydration contract** — HTML markers + **serialized server context** (`hydration_context` **lesson**).  
6. **Task graph for the repo** — `core` before `web` before apps; **optional `turbo.json`** (or `cargo` + custom driver) with explicit `dependsOn`, `inputs`, `outputs` (Turborepo **lesson**, §0.4).

### 0.3 Lessons from **Turborepo** (monorepo task orchestration)

- **`turbo.json` pipeline:** tasks declare **`dependsOn`** (`^build` = upstream package build first), **`inputs`** (globs that bust cache), **`outputs`** (artifacts to restore), **`env`** (cache key).  
- **Local + remote cache:** hashes of inputs → skip work; **remote** shares hits across CI and laptops.  
- **Parallel execution:** independent tasks run concurrently; DAG respects deps.

**Apply to Crepuscularity:** Define tasks: `crepus:parse`, `crepus:codegen`, `cargo:check-wasm`, `cargo:check-host`, `crepus:webext-manifest`, **tests**. Document **inputs**: `**/*.crepus`, `Cargo.toml`, `webext.toml`. **Outputs**: `target/crepus-out/**`, generated `*.rs`. Wire **CI** with `TURBO_TOKEN` / team or equivalent **sccache**/GitHub Actions cache.

### 0.4 Lessons from **Turbopack** (incremental computation)

Source: [Inside Turbopack: incremental computation](https://nextjs.org/blog/turbopack-incremental-computation) — **value cells** (`Vc`), automatic dependency edges, **dirty propagation**, **demand-driven** re-exec (only **active** entrypoints, e.g. open route in dev), **aggregation graph** for cheap global queries, **filesystem persistence** of the graph for `next dev` restarts.

**Apply to Crepuscularity:**

| Turbopack idea | Crepus analogue |
|----------------|-----------------|
| Value cell | **Fingerprinted** artifact: `(path, section?, stage)` → AST, binding map, codegen chunk |
| Dirty on file change | Notify **only** consumers in **include graph** and codegen dependents |
| Demand-driven | **Dev:** only recompute **open route / watched entries**; **build:** full closure |
| Skip if equal | Compare **semantic hash** of emitted Rust/HTML before writing (avoid rustc invalidation) |
| FS cache | Persist **`.crepus-cache/`** across **`crepus dev`** restarts (warm incremental) |

### 0.5 Lessons from **Next.js** (App Router, caching, server-first UI)

- **Server by default:** RSC / server components → **minimal JS** to client; interactivity opt-in (`"use client"`). **Analog:** `.crepus` defaults **static HTML + islands**; reactive WASM only where declared.  
- **Layered caches (mental model):** (1) **request memo** — dedupe work inside one render; (2) **data cache** — persisted fetch (our: memoize **external props** / API); (3) **full route cache** — SSG HTML + payload (our: **pre-render** + disk); (4) **client router cache** — instant back/forward (our: optional **in-memory** fragment cache).  
- **Segments & layouts:** shared shell, nested invalidation. **Analog:** **multi-component files** + **include** graph → invalidate **subtree** not whole site.  
- **Bundling:** Turbopack as default dev/build engine—**Rust** incremental bundler **lesson** already in §0.4.

**Apply:** Document **cache tags** per template/route for **on-demand** rebuild (like `revalidateTag` concept)—even if v1 is coarse “file changed.”

### 0.6 Delivery principles (optimization “stuff” checklist)

- **Profile before exotic:** `cargo build --timings`, `tracing` spans, browser Performance panel.  
- **Tokio:** never block the SSR executor on disk; **spawn_blocking** or async fs where needed.  
- **`mold`/`lld`**, **`CARGO_BUILD_JOBS`**, **workspace split** (stable core crate vs churny generated).  
- **CI cache:** **`sccache`** (or Turborepo remote cache, §0.3) so cold agents reuse codegen.  
- **Tests:** **`cargo nextest`** for faster parallel scheduling in CI.  
- **PGO / BOLT** optional on **release** WASM/host after baseline stable.  
- **WASM load:** **`instantiateStreaming`** + correct **`Content-Type`**; **`wasm-opt`** for **speed** preset when tuning guest code.  
- **HTML hot path:** if profiling shows escape dominates, consider **vectorized** escape libs; **arena / bump allocator** per request render (Zig-lesson: bound alloc lifetime).  
- **Binary DOM mutation protocol** under load (Dioxus **Sledgehammer** lesson, §5.3).  
- **`resolver = "2"`** workspace for WASM members.  
- **Correctness = speed:** golden HTML / hydration tests catch regressions before slow human QA.

### 0.7 Index — frameworks & techniques (coverage audit)

Everything below was used as **input** to this plan—either a **named product** or a **technique**. Use this table to see **what we studied** and **where it landed** in the spec.

| Name | Kind | What to learn from it | Where |
|------|------|------------------------|-------|
| **Leptos** | Rust UI + SSR | `reactive_graph`, `hydration_context`, `tachys`, prop vs attr | **§4**, **§8.2** |
| **Dioxus** | Rust UI cross-platform | `VirtualDom`, `Template`+dynamic pools, `WriteMutations`, Sledgehammer WS/batch, RSX hot reload, Subsecond patches | **§5**, **§8.2** |
| **Reactively** | Algorithm | Fine-grained graph semantics (Leptos lineage) | **§4.1**, **§15** |
| **Solid** | Compile-to-JS UI | Signals, run once, no VDOM | **§7.2**, **§8.3** |
| **Svelte** | Compiler UI | AST→analysis→codegen, `changed` guards, runes | **§8.3** |
| **React** (ecosystem) | VDOM baseline | **Contrast:** whole-tree reconcile is what we **avoid** on the hot path; prefer Solid/Svelte/Leptos patterns | *implicit* |
| **Vue 3** | Compiler + reactivity | **Compiler** strips static; **Proxy**/`ref` tracking—**analogy:** our binding analysis + runtime graph | **§8.12** |
| **Bun** | JS runtime + toolchain | JSC, Zig core, integrated bundler—**few process hops** | **§7.1**, **§8.1** |
| **Hono** | Edge/router | Ultra-thin stack | **§8.1** |
| **Fastify** | Node HTTP | Schema at edge, low overhead | **§8.1** |
| **Next.js** | Meta-framework | Server-first, layered caches, layouts, Turbopack default | **§0.5**, **§8.10** |
| **Turborepo** | Monorepo CI | `turbo.json`, inputs/outputs, remote cache | **§0.3**, **§8.10**, **§14 Phase 9** |
| **Turbopack** | Bundler | Value cells, dirty propagation, demand-driven, FS cache | **§0.4**, **§8.10** |
| **Astro** | Static + islands | Zero-JS default, opt-in interactivity | **§8.7** |
| **Qwik** | Resumability | Serialize boundaries, resume with less client replay | **§8.7**, **§10.4** |
| **htmx / Hotwire** | Hypermedia | HTML round-trips, **minimal** client—fits **static `.crepus`** + SSR | **§8.12** |
| **Phoenix / LiveView** | Elixir | Channels, server-owned state, DOM diffs over WS | **§8.6**, **§0** synthesis |
| **Sanic** | Python ASGI | uvloop-class loop, compiled tree router | **§6**, **§8.5** |
| **FastAPI / Starlette** | Python ASGI | Pydantic at boundary | **§8.5** |
| **Blacksheep** | Python ASGI | *Optional:* ultra-light async contender—same lessons as Sanic | **§8.12** |
| **Veb** | V web | Compile-time templates, App/Context, livereload inject, single binary | **§8.4** |
| **Axum / Tower / Hyper** | Rust HTTP | Non-blocking SSR, streaming | **§8.2** |
| **Actix** | Rust HTTP | Worker / runtime layout for **saturation** throughput—tune if SSR is RPS-bound | **§8.12** |
| **Remix / SvelteKit / Nuxt** | Full-stack meta | File routes, nested layouts, server loaders—**analogy:** our route→template registry + invalidation | **§8.12** |
| **Deno Fresh** | Islands | Server Preact, island hydration | **§8.12** |
| **Drogon / µWebSockets** | C++ HTTP | Filters, batch WS | **§8.8** |
| **nginx / Caddy / OpenResty / fasthttp / Netty** | Infra | Process model, pools, JIT config at edge | **§8.11** |
| **Cloudflare Workers** | Edge isolates | Small units, fast cold—**analogy:** islands + small WASM | **§8.11** |
| **GPUI + `view!`** (repo) | Native | Compile-time `.crepus` path—**mirror** semantics with web runtime | **§1**, macros crate |
| **Manifest V3 + webext crate** | Extensions | SW lifecycle, CSP, sandboxed iframe widgets | **§9** |
| **Parallelism L1–L3** | Technique | Rayon driver, overlapped `cargo`, rustc `-j` | **§3**, **§11**, **§14** |
| **Rust-first + runtime interpreter** | Technique | Fast dev without macro explosion on every template | **§10.2**, **§14** |
| **Hot reload vs recompile** | Technique | Template=data vs Rust=code (Dioxus RSX lesson) | **§5.4**, **§10.2**, **§14 Phase 1–2** |

If a row points only to **§8.12**, see the subsection below for the one-paragraph takeaway.

---

## 1. Repository baseline (current code)

| Crate / area | Role today |
|--------------|------------|
| `crates/crepuscularity-core` | AST, `TemplateContext`, `eval_expr`, parsers |
| `crates/crepuscularity-runtime` | Runtime parse, renderer, hot-reload engine (GPUI-oriented) |
| `crates/crepuscularity-web` | `parse_template` → **HTML string** (`render_*_to_html`), virtual `TemplateContext::virtual_files`, optional **`parallel`** + **Rayon** (host only) |
| `crates/crepuscularity_macros` | `view!` compile-time path (GPUI); not full web story |
| `crates/crepuscularity-cli` | **`crepus web`** — `.crepus` + `runtime/` → `dist/` (HTML shell, `crepus-bundle.json`, wasm-bindgen `pkg/`); **`crepus webext`** scaffolding, builds, manifest |
| `crates/crepuscularity-web` | **`render_bundle`** parses `crepus-bundle.json` and calls **`render_from_files`** (shared with **`crepus web serve`** and wasm site runtimes) |
| `crates/crepuscularity-dev` | `crepus-dev` — dev server / hot reload direction |
| `crates/crepuscularity-webext` | MV3 manifest types, **capability scan** of `.crepus`, **`widgets`**: `json_to_template`, `build_frame_doc` for sandboxed iframe `srcdoc`; see [webext.md](./webext.md) |

**Gap:** Web path is **string SSR/SSG** without a first-class **reactive client graph**, **hydration contract**, or unified **build driver** that overlaps WASM + host SSR with **per-component** invalidation. **Webext** reuses render helpers but does not yet share one **pipeline** with web for codegen, hot reload, and reactive islands inside extension UIs.

---

## 2. Goals and non-goals

### 2.1 Goals

- **Dev speed:** `.crepus` changes → **hot reload** (sub-second where possible) **without** full `rustc` when the template is data-only; Rust changes → **incremental `cargo`**.
- **Prod speed:** SSR/SSG and client updates minimize **wasted work** (fine-grained where dynamic, parallel where independent).
- **One driver mental model:** Staged pipeline (parse → analyze → emit per target) with **L1 Rayon**, **L2 parallel `cargo`** for client + server.
- **DSL-first:** `.crepus` remains the primary view; Rust owns types, side effects, integration.
- **Webext parity:** Popup, options, side panel, content-script UIs, and sandboxed widget **iframes** should use the **same** parse → analyze → emit story as web where MV3 security allows (see §9).

### 2.2 Non-goals (initially)

- Pixel parity with Dioxus/Leptos **ecosystem** (routers, devtools, asset macros).
- Replacing **rustc** incremental compilation.
- **Kernel-bypass NIC** (DPDK-style) or custom allocators **unless** profiling proves the bottleneck—document patterns only.

---

## 3. Parallelism model (three layers)

```
L3  cargo / rustc     — crate graph, -j, incremental deps
L2  Driver            — wasm32 artifact ∥ host SSR binary (two builds overlapped)
L1  Crepus driver     — Rayon: batch parse / codegen / analysis over files & sections
```

**WASM guest:** disable Rayon in the browser unless **explicit** wasm threads; keep Rayon on **host** dev server, SSR, CLI.

---

## 4. How **Leptos** does it (mechanics to learn)

Primary sources: `leptos-rs/leptos` crates **`reactive_graph`**, **`hydration_context`**, **`leptos_dom`**, **`tachys`**.

### 4.1 Fine-grained reactivity (`reactive_graph`)

From the crate’s own description:

- Primitives: **Signals** (mutable state), **Computations** (derived / memos, sync and async), **Effects** (sync to non-reactive world, e.g. DOM).
- **Source vs subscriber** nodes in a **reactive graph**; updates propagate only along subscribed edges—**no default full-tree diff**.
- **Automatic dependency tracking** at runtime: subscribers **re-subscribe** when a computation re-runs so branches that weren’t taken stop listening (dynamic dependencies).
- **Effects are treated as expensive:** the algorithm avoids re-running effects unnecessarily; effects are **async-scheduled** to the host executor (browser: `wasm-bindgen-futures`, server: `tokio`, etc.).
- Algorithm lineage: based on **[Reactively](https://github.com/modderme123/reactively)** (see article linked from `reactive_graph/README.md`).

**Lesson for Crepuscularity:** For interactive web, **do not** make “rebuild entire template string” the default hot path. Bind **effects** to **binding sites** (text node, attribute, list reconciliation).

### 4.2 Hydration data (`hydration_context`)

`hydration_context` addresses **two** problems:

1. Ship HTML from server so the client can attach listeners / interactivity (**hydration**).
2. Ship **the same data** the server used so the client doesn’t **re-fetch** or diverge (`SharedContext` pattern).

**Lesson:** Any SSR + client design needs an explicit **serialized context** contract (versioned), not only HTML strings.

### 4.3 DOM / view layer

Leptos pairs reactivity with a **typed view** layer (`view!` expands to structures consumed by **`tachys`** and DOM integration). Attributes like `prop:value` vs `value` matter because DOM **properties** vs **attributes** behave differently (see Leptos “COMMON_BUGS” on `<input>`).

**Lesson:** Generated or hand code must distinguish **property** vs **attribute** updates for correct reactive form controls.

---

## 5. How **Dioxus** does it (mechanics to learn)

Primary sources: `DioxusLabs/dioxus` **`notes/architecture/*.md`** (especially `01-CORE`, `04-SIGNALS`, `05-FULLSTACK`, `06-RENDERERS`, `07-HOTRELOAD`, `00-OVERVIEW`).

### 5.1 Core runtime (`01-CORE`)

- **`VirtualDom`** holds **scopes** (slab), **`dirty_scopes`** (`BTreeSet` ordered by **height**), runtime, scheduler channel.
- **Initial render:** `rebuild()` → `run_scope` → recursive **scope** + node creation → mutations written to a **`WriteMutations`** sink.
- **Updates:** scheduler marks scopes dirty → **`render_immediate()`** processes dirty scopes **top-to-bottom** (height order) → **`run_and_diff_scope`** runs component then **`diff_scope`**.
- **VNode** carries a static **`Template`** (`TemplateNode` tree, **node_paths** / **attr_paths** slices) plus **`dynamic_nodes`** / **`dynamic_attrs`**—**static skeleton is compiled; only dynamic slots participate heavily in diffing.**
- **Mutations** are an explicit enum API: append children, create text, load template, set attribute, replace, etc.—**renderer-agnostic**.

**Lesson:** Separating **(a) static template shape** from **(b) dynamic pools** is how you shrink steady-state diff cost. Crepuscularity `.crepus` can compile or analyze into the same **shape + dynamic binding list**.

### 5.2 Signals (`04-SIGNALS`, summarized earlier in this repo)

- **Generational-box** gives **Copy**-like handles; **Signal** read **subscribes** current reactive scope; write **marks dirty** on subscribers.
- **Memo** lazy + `PartialEq` skip; **Store** for nested granular field paths.

**Lesson:** If Crepuscularity ships a client runtime, **one** graph implementation; optional later: field-path stores for large props.

### 5.3 Web renderer (`06-RENDERERS`)

- **`WriteMutations`** implemented for web often via **Sledgehammer**: **binary-encoded** DOM ops, **template cache** in JS, batch application.
- **Events:** delegation from root, walk to `data-dioxus-id`.
- **Hydration:** SSR embeds attributes; client gets encoded **hydration context**; **`skip_mutations`** path walks existing DOM and assigns ids; streaming hydration for suspense boundaries.

**Lesson:** For max throughput, **batch** DOM writes; **binary** protocols beat naive per-call JS interop at scale.

### 5.4 Hot reload (`07-HOTRELOAD`) — two systems

**A. Subsecond (Rust logic)**  
- **Jump table indirection:** calls go through **`APP_JUMP_TABLE`**; patch loads dylib, updates pointers; **ASLR** handled via anchor symbol offset.
- **Thin builds** recompile only changed functions into a patch library.
- Limits: struct layout changes, TLS quirks, workspace scope—**WASM module reload** is limited.

**B. RSX template hot reload**  
- **Conservative gate:** strip/compare RSX bodies—if **Rust** changed, **full rebuild**; if only **literals/template** changed, **template diff**.
- **Dynamic pools** for text segments, dynamic nodes, dynamic attrs; **greedy reuse** scoring.

**Devtools:** WebSocket **`DevserverMsg`** (`HotReload`, `FullReloadCommand`, etc.), PID/build id filtering.

**Lesson for Crepuscularity:** Mirror the **split**:  
- **`.crepus` hot** = parse + AST swap + re-render/patch (like RSX path).  
- **Rust hot** = `cargo incremental` (full Subsecond-style patching is **optional advanced**).

### 5.5 Crate graph (`00-OVERVIEW`)

Dioxus explicitly separates: **`dioxus-core`**, **`dioxus-rsx`**, **`dioxus-signals`**, renderers, fullstack, CLI, **`subsecond`**, wasm split tooling.

**Lesson:** Keep **parser**, **runtime graph**, **emitters**, **devtools transport** in separable crates.

---

## 6. How **Sanic** does it (Python — architectural lessons)

Official positioning: **async-first** ASGI stack, built for I/O-bound concurrency.

Documented / widely described mechanics:

- **Asyncio event loop**; **`uvloop`** replaces default loop with **libuv**-backed implementation (same family as Node’s I/O model), improving **poll** / syscall efficiency on epoll/kqueue-class OS APIs.
- **Compiled router** (since v21.x ecosystem): **tree-based** router (`sanic-router`) instead of **regex** routing for lower overhead at match time.
- **Unified request/response cycle** (v21.3 notes) to reduce branching between “streaming” vs “normal” paths.

**Lesson for Crepuscularity dev/SSR server:** Use **Tokio** well (or concurrent equivalent), **pre-compile** route → template registry maps, **avoid per-request** allocation storms, **stream** body where it helps TTFB.

---

## 7. How **Bun** / **Solid** inform the shape (short)

### 7.1 Bun (JS runtime)

Public architecture story (summarized across Bun’s materials and third-party writeups):

- **JavaScriptCore** emphasis on **startup** latency vs V8’s peak JIT focus in some workloads.
- Runtime and tooling implemented in **Zig** with tight integration between bundler, resolver, and core—**few process hops** for `bun run`.

**Lesson:** **`crepus`** CLI should avoid **N subprocess spawns per tick**; prefer **long-lived `crepus dev`** with in-process watch + **one** child `cargo` when needed.

### 7.2 Solid

Documented model: **JSX compiles once** to real DOM operations; **components execute once**; **signals** drive targeted updates—**no VDOM** on the hot path.

**Lesson:** Crepuscularity should treat **component render** as **establish bindings**, not **re-execute full template** on each state tick unless analysis says it’s static-closed.

---

## 8. Web frameworks across languages — how each gets fast

This is the **primary** cross-language survey: **app and UI frameworks** (Bun, Leptos, Svelte, **Veb**, Phoenix, …). **§8.10** is a short **infrastructure** appendix (nginx, Netty, fasthttp). *Deeper Rust traces: §4–§5.*

### 8.1 JavaScript / TypeScript — **Bun**, **Hono**, Node stacks

| Piece | How it works | Lesson for Crepuscularity |
|-------|----------------|---------------------------|
| **Bun** | **JavaScriptCore** + **Zig** core; **integrated** runtime, bundler, transpiler; **`Bun.serve`**; optimized for **low startup** and **few hops** vs separate npm/webpack processes | **`crepus dev`**: one long-lived process; don’t spawn a full toolchain shell on every save |
| **Hono** | **Tiny** router; Workers/Bun/Node; **minimal** middleware default | SSR: **thin handler** → **O(1) template lookup** |
| **Fastify** (Node) | Schema-driven; **low overhead** plugins | Validate at **HTTP boundary**; never re-parse routing per template byte |

### 8.2 Rust — **Leptos**, **Dioxus**, **Axum** (host)

| Framework | How it works | Lesson |
|-----------|----------------|--------|
| **Leptos** | **`reactive_graph`** (signals / memos / effects, Reactively-style tracking); **`tachys` + dom**; **`hydration_context`** ships **state + HTML** | **Fine-grained client** + **hydration payload**; see **§4** |
| **Dioxus** | **`VirtualDom`**, **Template + dynamic pools**, **`WriteMutations`**, **binary** DOM batches; **RSX hot reload** + optional **Subsecond** patches | **Static/dynamic split**; see **§5** |
| **Axum** | **Tower** + **Hyper** async HTTP | **Non-blocking** SSR, **streaming** bodies |

### 8.3 Compile-to-JS UI — **Svelte**, **Solid**

| Framework | How it works | Lesson |
|-----------|----------------|--------|
| **Svelte** | Compiler: **AST → analysis → codegen** imperative JS (`create_fragment`, `if (changed.x) set_data(...)`). **No VDOM.** **Runes** (`$state`, …) in v5 widen reactivity | **`.crepus` driver** should emit **guarded updates** per binding—**compile-time** knowledge of what can change |
| **Solid** | JSX → **DOM refs + signals**; component runs **once** | **Subscribe once**, update **only bound nodes** |

### 8.4 V — **Veb** (`vlib/veb`)

| Piece | How it works | Lesson |
|-------|----------------|--------|
| **Codegen** | V **→ C → native**; **`-prod`** release | Rust **release + LTO** / PGO analogy |
| **Templates** | **Precompiled** at **compile time** (failures are **compile errors**) | **Driver:** surface template errors at **`crepus build`**, not silent runtime |
| **`App` vs `Context`** | Shared **App** vs **per-request Context** (embeds `veb.Context`) | Global registry vs **`TemplateContext` per render** |
| **Routing** | Methods + attributes (`@['/p']`, `@[get]`) — **registry**, not regex-heavy | Pre-map **route → template entry** |
| **Live reload** | `v -d veb_livereload watch run .` — **inject** small client before `</html>` to trigger refresh when sources or compiled templates change | Optional **HTML injection** or **WS** for `crepus` dev |
| **Deploy** | **Single binary** embeds templates | **`include_bytes!` / virtual map** for SSR binaries |

Official module docs: https://docs.vlang.io/veb.html — source: `vlib/veb/README.md` in **vlang/v**.

### 8.5 Python — **Sanic**, **FastAPI** / **Starlette**

| Framework | How it works | Lesson |
|-----------|----------------|--------|
| **Sanic** | **async** ASGI; **uvloop**; **compiled tree router** | **Tokio** + **compiled routes** — **§6** |
| **FastAPI** | **Starlette** ASGI + **Pydantic** I/O | Typed **props** at API edge |

### 8.6 Elixir — **Phoenix**, **LiveView**

| Piece | How it works | Lesson |
|-------|----------------|--------|
| **Phoenix** | **Channels** (WS), **Presence** | **HMR / devtools** transport |
| **LiveView** | Server **state** + **diffs** → browser **DOM patches** | Optional **crepus** “**patch stream**” mode, not only static HTML |

### 8.7 Meta / islands — **Astro**, **Qwik** (concepts)

| Idea | Mechanism | Lesson |
|------|-----------|--------|
| **Astro** | Default **no JS**; **islands** for interactivity | **Static `.crepus`** default; opt-in **hydration** |
| **Qwik** | **Resumable** serialized state / boundaries | Aligns with **serialized hydration** (**§10.4**) |

### 8.8 C++ app servers — **Drogon**, **µWebSockets**

| Framework | How it works | Lesson |
|-----------|----------------|--------|
| **Drogon** | Async HTTP + **filters** | Middleware **short-circuit** before expensive render |
| **µWebSockets** | Minimal async HTTP/WS | **Batch** WS messages if doing LiveView-style |

### 8.9 Synthesis — frameworks → Crepuscularity

| Pattern | Source | Apply |
|---------|--------|--------|
| Compile-time template **analysis + codegen** | **Svelte**, **Veb** | Driver emits **per-template update fns** or IR |
| **Fine-grained** graph | **Leptos**, **Solid** | WASM **signals + effects** |
| **Static shell + dynamic pools** | **Dioxus** | AST → **frozen shape** + dynamic bindings |
| **Integrated toolchain** | **Bun** | **`crepus`** owns watch + orchestration |
| **Live reload** | **Veb**, **Dioxus** | Template **fast path** vs Rust **rebuild** |
| **Server-driven patches** | **LiveView** | Optional **second output** besides HTML string |
| **Monorepo task cache + DAG** | **Turborepo** | **`turbo.json`**-style `inputs`/`outputs` for `crepus` + `cargo` (§0.3) |
| **Fine-grained incremental build** | **Turbopack** | **Value-cell** style driver cache + dirty propagation (§0.4) |
| **Server-first + layered cache** | **Next.js** App Router | Static default, islands, route/segment invalidation (§0.5) |

### 8.10 JavaScript meta-stack — **Next.js**, **Turborepo**, **Turbopack**

These are not “another language UI” like Svelte—they are how the **largest JS apps** optimize **builds, dev loop, and shipping**. Steal the **architecture**, not the runtime.

| Piece | What it does | Copy into Crepuscularity |
|-------|----------------|---------------------------|
| **Next.js** (App Router) | **Server-first** rendering; **client** only where marked; **nested layouts**; **several cache layers** (request memo, data cache, full-route static cache, client router cache — see §0.5); **revalidatePath / revalidateTag** for targeted invalidation | **Default:** static `.crepus` / SSR string; **opt-in** reactive WASM islands; **include/layout graph** for partial invalidation; future **tagged** template deps |
| **Turborepo** | Declares **`turbo.json`** tasks: **`dependsOn`**, **`inputs`**, **`outputs`**, **`env`**; **local `.turbo` cache** + **remote** (`turbo login`, `TURBO_TOKEN` / `TURBO_TEAM` in CI) | Optional **`crepus-workspace.json`** or document **standard `turbo`** wrapping `cargo` + `crepus`; **share CI cache** for codegen artifacts |
| **Turbopack** | **Rust** bundler; **incremental computation** via **value cells**, auto deps, **dirty propagation**, **demand-driven** recompute for active routes, **aggregation graph**, **on-disk** dev cache (post–Next 16.1 story) | **`crepus dev`:** persistent **`.crepus-cache/`**, **active entry** invalidation, **semantic equality** skip writes; driver internals as **query graph** not one-shot script |

Official deep-dive: [Inside Turbopack: Building Faster by Building Less](https://nextjs.org/blog/turbopack-incremental-computation). Turborepo: https://turbo.build/repo/docs — caching / CI.

### 8.11 Appendix — infra (not UI frameworks)

| Layer | Examples | Note |
|-------|----------|------|
| Proxy | **nginx**, **Caddy** | TLS, compression, static |
| Go low-level | **fasthttp** | Pooled `[]byte` if micro-optimizing |
| JVM IO | **Netty** | Pooled `ByteBuf` |
| Edge | **Workers isolates** | Small cold units |
| nginx + **LuaJIT** | **OpenResty** | JIT config — analogy: **cached codegen** |

### 8.12 Extra frameworks & patterns (one-paragraph each)

Listed in **§0.7** for traceability; expand here only as needed for implementation spikes.

- **Vue 3:** **compile-time** strips static hoisting; **runtime** `ref`/`reactive` tracks deps—**same split** as our “analyze `.crepus` → codegen static shell + signal hooks for dynamic nodes.”  
- **React (contrast):** default **reconcile Virtual DOM**—**expensive** vs fine-grained; we **do not** adopt VDOM as the default client model.  
- **htmx / Hotwire Turbo:** server returns **HTML fragments**; browser swaps partials—aligns with **SSR-first** `.crepus` and **optional** small clients (no big WASM).  
- **Actix:** often **multi-worker** / thread model for HTTP at peak RPS—if **SSR** is compute-heavy, benchmark vs Axum; **lesson** is **thread/worker** layout, not “rewrite in Actix.”  
- **Blacksheep** (Python): another **ASGI** perf-oriented stack—**same** lessons as Sanic (async loop, thin handlers, compiled routing).  
- **Remix / SvelteKit / Nuxt:** **nested routes**, **loaders**, **shared layouts**—map to **include graph** + **segment invalidation** (§0.5 Next analogy).  
- **Deno Fresh:** **islands** with server-driven composition—reinforces **static + opt-in client** (§8.7 Astro overlap).

---

## 9. Browser extensions — `crepuscularity-webext` (Manifest V3)

Extensions are **first-class** for this spec: same **`.crepus`**, same **core AST**, and the same **speed goals**, constrained by **browser security** and **MV3 lifecycle**.

### 9.1 MV3 mechanics (why extension code is “different”)

| Concept | Behavior | Implication for Crepuscularity |
|---------|----------|-------------------------------|
| **Service worker (background)** | **Ephemeral**; wakes on events; **no long-lived state** unless persisted | **Heavy reactive graph** in SW should **serialize checkpoints** or live in **offscreen** / **popup** where allowed; prefer **message** + **small WASM** boot |
| **Content scripts** | **Isolated world**; DOM access; **no** direct page JS variable access | **Templates** that need page data go through **DOM APIs** or **injected bridges**; design **props** as explicit messages |
| **Popup / options / side panel** | Short-lived **extension pages**; full DOM | **Ideal** for reactive WASM + `.crepus` **same as web** |
| **`chrome.runtime` messaging** | Async, structured clones | **Actor-style** (BEAM lesson): define **typed** envelopes; **batch** updates to reduce chatter |
| **CSP** | Restricts **`eval`**, inline scripts in some surfaces | **Hot reload** may require **bundled** dev script or **allowed** `unsafe-eval` only in **unpacked dev**—document clearly |
| **Sandboxed `iframe` + `srcdoc`** | Used for untrusted widget HTML | Already: **`widgets::build_frame_doc`** assembles doc; **`json_to_template`** bridges JSON props → `TemplateValue` |

### 9.2 Current crate capabilities (implementers’ baseline)

Source: `crates/crepuscularity-webext/`

- **`manifest`**: `ManifestV3`, JSON emit, capabilities section.  
- **`scanner`**: **`scan_crepus_for_capabilities`** — static scan of `.crepus` for needed API usage.  
- **`watcher`**: **`CapabilityWatcher`** — dev-time **file watch** / suggestion of missing capabilities.  
- **`widgets`**: **`build_frame_doc`**, **`json_to_template`** — glue between JS host and `crepuscularity-core` values.  
- **`api`**: Browser-facing AST for policies (storage, messaging, etc.).

CLI integration lives under **`crepus webext`** (see `docs/webext.md`).

### 9.3 Target architecture (web + webext unified)

```
.crepus  ──►  crepuscularity-core (AST)
                    │
        ┌───────────┼───────────┬────────────────┐
        ▼           ▼           ▼                ▼
   crepuscularity-web   popup/options UI   content.js bridge
   (SSR / WASM site)    (extension page)   (JSON → TemplateValue)
        │                    │                    │
        └─────────── shared: virtual file map, parse cache, codegen ───┘
```

- **Popup / options / side panel:** Same **client** story as §10 (reactive graph + batched DOM) when feasible.  
- **Service worker:** Prefer **thin** message router + **offscreen** document or **WASM** for heavy work if **wake time** budget demands.  
- **Content scripts:** **String/html** render or **minimal** patch; **scan** for capability declarations.

### 9.4 Dev workflow for webext

- **`.crepus` hot reload** on extension **pages** via same WS / virtual-map approach as web (where CSP permits).  
- **`webext.toml`** / manifest regen on capability or version change.  
- **Rust / WASM** changes: **`crepus webext build`** → incremental `cargo`; **reload extension** in `chrome://extensions` (full ext reload often required—surface in CLI output).  
- **Parallelism:** `webext` WASM crate can build **in parallel** with other workspace members (L2).

### 9.5 Prod workflow for webext

- **Speed-first:** `-O3`, same as web WASM; **threads** only if extension policy + `crossOriginIsolated` story allows in that context.  
- **Precompile** template hotspots to Rust in **published** builds if QPS inside iframe widgets matters.

---

## 10. Crepuscularity target architecture (what to build)

### 10.1 Data flow

```
.crepus + Rust app
      ↓
  parse → AST (crepuscularity-core)
      ↓
  analyze → { static_regions[], binding_sites[], include_graph }
      ↓
┌─────┴─────┐
▼           ▼
host SSR    wasm client
(string or   (reactive graph +
 streaming)  batched DOM ops)
```

### 10.2 Dev: two-speed workflow

| Input change | Expected behavior |
|--------------|-------------------|
| `.crepus` | **Hot reload:** watch → reparse changed files → update virtual file map → rerender / send patch over WS; **no rustc** on pure markup paths. |
| `*.rs`, `Cargo.toml`, codegen outputs | **Recompile:** debounced `cargo build` / `cargo check`; parallel FE/BE if two artifacts. |
| Template change that **adds new typed props** | Escalate to **recompile** with user-visible reason. |

Reuse **Dioxus RSX** lesson: **conservative** automatic hot reload—if unsure, **full rebuild**.

### 10.3 Prod: speed-first compile flags

- Host and WASM: **`-C opt-level=3`**, **LTO** when link time acceptable; **PGO** optional later.
- **WASM threads** allowed if profiling shows CPU-bound client work.
- **`wasm-opt`** tuned for **speed**, not **`-Oz`**, unless shipping size-constrained embeds.

### 10.4 Hydration contract (minimum)

1. **Markers** in HTML for roots and binding ids (data attributes or comments—pick one grammar and version it).
2. **Serialized `SharedContext` analogue** for server→client data (see Leptos `hydration_context` purpose).
3. **Keyed lists** for reconciling `for` when client attaches.

### 10.5 DOM updates

Preferred order of implementation:

1. **Batched mutations** buffer (Rust side) → flush to `web_sys` or thin JS shim (Dioxus Sledgehammer lesson).
2. **Static region cache:** first render builds template **shell**; updates touch **dynamic** slots only (Dioxus `Template` + paths lesson).
3. **Property vs attribute** handling per Leptos gotchas for inputs.

---

## 11. Driver, cache, compilation units

- **Fingerprint:** `(file hash)` → optional `(section #Name)` → `(target: wasm | host)` → **pipeline stage** (parse / bind / emit).
- **Driver cache dir:** e.g. `.crepus-cache/` — same role as Turbopack’s **value cells** / on-disk dev cache: map **input hash** → **output bytes + semantic hash**; **skip** rewriting `generated/*.rs` when **unchanged** (avoids **rustc** churn).
- **Turborepo alignment:** declare those paths as **`outputs`** in `turbo.json` (Phase 9); **`inputs`** include `**/*.crepus`, `Cargo.toml`, driver version constant.
- **Rayon (L1):** parallel over **independent** files/components; **topologically sort** the include DAG before emit.
- **Parallel `cargo` (L2):** `myapp-client` (`wasm32-unknown-unknown`) in parallel with `myapp-server` (host) when both exist.

---

## 12. Testing & CI expectations

- **Golden HTML** / DOM snapshots per fixture template (SSR).
- **Hydration** tests: server render → client attach → event → expected DOM.
- **Webext:** `widgets` unit tests (`build_frame_doc`, `json_to_template`); manifest JSON golden files; scanner snapshots for **capability** inference from `.crepus` samples.
- **cargo** workspace `resolver = "2"` for WASM member graphs (per Leptos common bugs doc).
- CI: `cargo fmt`, `clippy -D warnings`, **`cargo test --workspace`**, optional **`wasm32-unknown-unknown`** smoke build (web + **webext** runtime crate when present).

---

## 13. Open choices (record decisions here as you go)

| Decision | Options | Recommendation to start |
|----------|---------|-------------------------|
| Reactive core | In-tree vs use `reactive_graph` crate | Spike **in-tree** minimal graph (~signals + memos + effects) mirroring Reactively semantics; swap later if dependency OK |
| Client DOM | Raw `web_sys` vs small JS interpreter | Prototype `web_sys` batching first; evaluate binary protocol if profiling demands |
| Codegen vs runtime in prod | Full codegen | **Speed-first prod:** codegen hot templates; **dev** can stay interpreter |
| HTTP stack for dev/SSR | axum / hyper / actix | Pick what the repo already trends toward; tune after flamegraph |
| Webext HMR | WS vs extension messaging only | Prefer **reload extension page** + **virtual file map** in dev; respect **CSP** per surface |
| Monorepo orchestration | Raw `cargo` only vs **Turborepo** | Phase 9: **`turbo.json`** for **cacheable** `crepus` + `cargo` tasks; remote cache in CI |

---

## 14. Implementation phases (for AI / task breakdown)

Execute **in order**. Each phase has **acceptance criteria** so the runtime/framework is shippable, not “half a demo.”

---

**Phase 0 — Observability baseline**  
- Add **`tracing`** spans: `parse_template`, `eval_expr` (sampled), `render_html`, `dev_request_total`.  
- **`cargo build --timings`** documented for one reference app.  
- **Acceptance:** flamegraph / timing doc can answer “where did the ms go?” for one SSR request.

**Phase 1 — Unified watch + `.crepus` hot reload**  
- Filesystem watch → **virtual file map** → `crepuscularity-web` / runtime re-render.  
- **WS** (or Veb-style **HTML inject**) to refresh browser; **debounce** bursts.  
- **Acceptance:** edit `.crepus` → visible update **without** `rustc`; Rust edit still requires **cargo**.

**Phase 2 — Binding analysis + driver fingerprints**  
- AST pass: **static regions** vs **binding sites**; emit **metadata** (JSON or Rust).  
- **Fingerprint** `(file, section?, stage)` → cache under **`.crepus-cache/`**; **skip write** if output hash unchanged (Turbopack-style).  
- **Acceptance:** second save with no logical change does **not** touch generated glue / does not force full rustc rebuild.

**Phase 3 — Reactive client MVP (WASM)**  
- Minimal **signal / effect** graph; one **text** + one **class** binding; **`web_sys`** batching.  
- **Acceptance:** click increments counter with **no** full-template `String` rebuild in hot path (profiled).

**Phase 4 — SSR hydration**  
- **Markers** in HTML + **serialized props/context** slice; client **`hydrate_root`**.  
- **Acceptance:** server HTML + hydration → **event** works; golden DOM test.

**Phase 5 — Parallel driver + overlapped targets**  
- **Rayon** batch parse/codegen (L1); **`cargo -p client --target wasm32`** ∥ **`cargo -p server`** (L2).  
- **Acceptance:** documented `crepus build` flags; CI runs both artifacts.

**Phase 6 — Production hardening**  
- Streamed SSR where wins; **release** profile **`-O3`**, LTO optional; DOM **mutation batch** under stress.  
- **Acceptance:** benchmark table vs Phase 1 string-only baseline (even if crude).

**Phase 7 — Webext unified pipeline**  
- Shared **`.crepus-cache/`** with web; **`crepus webext dev`**; CSP-safe reload story.  
- **Acceptance:** one sample extension: popup **hot** `.crepus`, full reload message when manifest/capability changes.

**Phase 8 — Extension tests & policy**  
- Golden **manifest**, **scanner**, **`build_frame_doc`** tests; SW wake notes.  
- **Acceptance:** CI covers `crepuscularity-webext` crate.

**Phase 9 — Monorepo task graph (Turborepo-style)**  
- Add **`turbo.json`** (or documented equivalent) with tasks: `crepus-codegen`, `build:wasm`, `build:server`, `test`, **`inputs`/`outputs`** for `.crepus` and `target/crepus-out`.  
- Wire **remote cache** (Vercel Turborepo or **custom** S3/GitHub Actions cache) for **codegen** artifacts.  
- **Acceptance:** clean CI on second run is **mostly cache hits** for unchanged templates.

**Phase 10 — Advanced incremental (optional)**  
- **Demand-driven** driver: only active **route/entry** recomputes in `crepus dev` (Turbopack **active query**).  
- **Tag-based** invalidation for data deps (Next **revalidateTag** analogue).  
- **Acceptance:** doc + prototype on one multi-route example.

---

## 15. Primary references (URLs)

### Frameworks & libraries (direct mechanics)

- Leptos `reactive_graph`: https://github.com/leptos-rs/leptos/tree/main/reactive_graph  
- Leptos `hydration_context`: https://github.com/leptos-rs/leptos/tree/main/hydration_context  
- Dioxus architecture notes: https://github.com/DioxusLabs/dioxus/tree/main/notes/architecture  
- Reactively (reactive algorithm): https://github.com/modderme123/reactively  
- **Svelte** compiler docs: https://svelte.dev/docs/svelte/svelte-compiler  
- **Solid** docs (reactivity): https://docs.solidjs.com/advanced-concepts/fine-grained-reactivity  
- **Bun**: https://bun.sh/docs — runtime + `Bun.serve`  
- **Hono**: https://hono.dev/docs  
- **V / Veb** module docs: https://docs.vlang.io/veb.html — source `github.com/vlang/v` → `vlib/veb/`  
- **Sanic**: https://sanic.dev/  
- **Phoenix** (incl. LiveView): https://www.phoenixframework.org/docs  
- **Next.js** — Turbopack / incremental: https://nextjs.org/blog/turbopack-incremental-computation  
- **Next.js** — caching journey / concepts: https://nextjs.org/blog/our-journey-with-caching  
- **Turborepo** (tasks, caching, CI): https://turbo.build/repo/docs  
- **Turbopack** API / architecture hub: https://turbo.build/pack/docs  

### Cross-language & systems (patterns)

- nginx architecture (worker/event): official **beginner’s guide** / docs — https://nginx.org/en/docs/  
- Netty user guide (ByteBuf, threading): https://netty.io/wiki/user-guide-for-4.x.html  
- Valyala **fasthttp** (Go, `[]byte`-oriented HTTP): https://github.com/valyala/fasthttp  
- OpenResty: https://openresty.org/  
- Phoenix framework: https://www.phoenixframework.org/  
- Cloudflare blog (**V8 isolates** / Workers model): search `workers.dev` “isolates” for architecture posts  
- MDN **Manifest V3** overview: https://developer.mozilla.org/en-US/docs/Mozilla/Add-ons/WebExtensions/manifest.json/manifest_version  
- Chrome **MV3 migration** guidance: https://developer.chrome.com/docs/extensions/develop/migrate  

---

## 16. Other documentation

- **`docs/webext.md`** — User-facing browser extension setup; **MV3 implementation** is **§9** here.  
- **`docs/dsl.md`**, **`docs/components.md`**, **`docs/cli.md`** — DSL and CLI reference.  
- **Execution:** **§0** (master plan) + **§14** (phases 0–10).  

---

*End of specification.*
