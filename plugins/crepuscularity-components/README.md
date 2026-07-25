# crepuscularity-components

Multi-target dither / primitive / motion component registry for Crepuscularity
(Flutter, Svelte, Moonshine, GPUI, and `.crepus`).

## Layout

| Path | Role |
|------|------|
| `catalog/components.json` | Component registry (ids, specs, platforms, themes) |
| `catalog/themes/` | Named theme JSON (7 themes) |
| `specs/` | Per-component source-of-truth specs (44) |
| `packages/{flutter,svelte,moonshine,gpui}/` | Target implementations |

## Themes

`dither-kit`, `kumo`, `night`, `chalk`, `aurora`, `dawn`, `zinc`

## CLI

```bash
crepus components list
crepus components themes
crepus components add button --target moonshine
crepus moonshine dep   # @crepuscularity/components snippet
```

## Validate / test

```bash
bun run scripts/validate-catalog.ts
(cd packages/flutter && flutter test)
(cd packages/svelte && bun test)
(cd packages/moonshine && bun test)
cargo test --manifest-path packages/gpui/Cargo.toml
```
