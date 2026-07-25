# crepuscularity-components

Shared UI component catalog for Crepuscularity targets (Flutter, Svelte, Moonshine, GPUI).

## Catalog

- `catalog/components.json` — component ids, descriptions, and per-target path hints
- `catalog/themes/` — named theme JSON files

## CLI

```bash
crepus components list
crepus components themes
crepus components add button --target moonshine
```

Path hints point at the files under this package. Copy or wire them into your app; the CLI prints install guidance rather than rewriting your project for every target.
