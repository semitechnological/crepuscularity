# Changelog

## 0.1.1 — 2026-07-25

- Catalog expanded to 44 components (Base UI / Spell primitives + spark-bars, heatmap, pixel-grid)
- Themes: dawn, zinc registered alongside dither-kit, kumo, night, chalk, aurora
- GPUI: `lib.rs` / `mod.rs`, Bayer 4×4 `sparkline_alphas` → `Vec<u8>`, theme consumption docs
- Flutter 0.1.1: `DitherSeparator`, `DitherSkeleton`, `DitherEmptyState`; dawn/zinc themes
- validate-catalog checks theme files, platforms, and spec theme refs

## 0.1.0 — 2026-07-25

- Initial multi-target registry: catalog, themes, specs
- Flutter package with dither-kit paint port (sparkline, area, bar) + primitives
- Svelte 5 and Moonshine/React canvas sparklines
- GPUI palette constants
- Catalog validation script
