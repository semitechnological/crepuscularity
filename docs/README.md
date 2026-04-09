# Documentation

Guides and references for the `.crepus` DSL, tooling, and browser extensions.

| Guide | Description |
| --- | --- |
| [DSL reference](dsl.md) | Syntax, control flow, attributes, animations |
| [Components](components.md) | `include`, slots, defaults, multi-component files |
| [CLI](cli.md) | `crepus` commands for apps, web, and extensions |
| [Browser extensions](webext.md) | Manifest V3 apps with `crepus webext` |

Implementation details for compilers and agents live in-repo at [`CREPUS_WEB_IMPLEMENTATION_SPEC.md`](CREPUS_WEB_IMPLEMENTATION_SPEC.md) (not published on the docs site).

Historical note: see [web-build-migration.md](web-build-migration.md) for the **`crepus web build`** move from **`site.json`-as-page** to **`.crepus` + WASM**.

**Published HTML**

When you run `crepus web build --site docs-site`, Markdown in this directory is turned into styled pages under `docs-site/dist/docs/` (for example `dsl.html`). The project’s GitHub Pages workflow uses that output for the public documentation site.
