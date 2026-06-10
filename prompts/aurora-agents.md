# crepus aurora — SwiftUI (via Aurorality CLI)

Generate SwiftUI views from `.crepus` templates. Delegates to the `aurorality` CLI.

Requirements: `cargo install aurorality-cli`

## Quick start

```bash
crepus aurora new MyProject
crepus aurora dev --watch . --port 8080          # hot-reload preview
crepus aurora build --watch . --out dist         # static build
crepus aurora swiftgen --view my-view.crepus --out Sources/ --view-name MyView
```

## Commands

| Command | Description |
|---------|-------------|
| `crepus aurora new <name>` | Scaffold SwiftUI project |
| `crepus aurora dev --watch DIR --port N` | Hot-reload preview window |
| `crepus aurora build --watch DIR --out DIR` | Static SwiftUI build |
| `crepus aurora swiftgen --view FILE --out DIR --view-name NAME` | Generate Swift view file |

## Workflow

1. Write `.crepus` templates
2. `crepus aurora dev` opens a SwiftUI preview window that hot-reloads on save
3. Generate Swift source files with `swiftgen` for production use

## Key crates

- Delegates to `aurorality` CLI (external)
- Templates rendered through Crepuscularity core pipeline