# IDE extensions

This doc sketches how editors can integrate with **Crepuscularity** and **Aurorality** for a smoother `.crepus` workflow.

## Goals

- **Syntax highlighting** for `.crepus` (indentation trees + optional JSX-style regions).
- **Snippets** for common blocks (`navigationsplitview`, `for` / `if`, Tailwind-like classes).
- **Tasks / runners** wired to the CLI:
  - `crepus` for web / GPUI targets.
  - `aurorality swiftgen` for SwiftUI emission.
  - `aurorality dev` for hybrid hot reload (IR + optional swiftgen status over WebSocket).

## Aurorality dev server

When `aurorality dev` runs with `--swiftgen-view` / `--swiftgen-out`, saves to the watched template can trigger:

1. **IR push** over the existing hot-reload WebSocket (unless `--no-ir`).
2. **`SwiftgenStatus`** envelopes after `swiftgen` runs (success + diagnostics).

Editors can surface diagnostics from `SwiftgenStatus.errors` next to the template path without doing a full Swift build.

## UniFFI / JSON envelopes

Hot reload messages are defined in `crepuscularity-native` (`HotReloadMessage`). Swift clients decode the same shapes as the CLI emits (`DevHello`, template reload JSON, `SwiftgenStatus`).

## Related

- [Aurorality (SwiftUI engine)](aurorality.md)
- [DSL reference](dsl.md)
- [CLI](cli.md)
