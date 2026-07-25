# crepuscularity-components

Multi-target dither / primitive / motion component registry for Crepuscularity
(Flutter, Svelte, Moonshine, GPUI, and `.crepus`).

## Layout

| Path | Role |
|------|------|
| `catalog/components.json` | Component registry (ids, specs, platforms, themes) |
| `catalog/themes/` | Named theme JSON |
| `specs/` | Per-component source-of-truth specs |
| `packages/{flutter,svelte,moonshine,gpui}/` | Target implementations |

## CLI

```bash
crepus components list
crepus components themes
crepus components add button --target moonshine
crepus moonshine dep   # @crepuscularity/components snippet
```
