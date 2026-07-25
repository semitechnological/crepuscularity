# crepuscularity-components-gpui stubs

Thin Rust surface that `crepuscularity-gpui` (or Zed) can path-include.
**No `gpui` dependency** — only palette seeds and Bayer sparkline math.
Package name is `crepuscularity-components-gpui` to avoid clashing with the
real `crepuscularity-gpui` backend crate.

## Layout

| File | Role |
|------|------|
| `lib.rs` | Crate root (`crepuscularity-components-gpui`) |
| `mod.rs` | Same exports for `#[path = …] mod …` inclusion |
| `palette.rs` | dither-kit + Kumo RGB seeds, `THEME_NAMES` |
| `sparkline.rs` | Bayer 4×4 → `Vec<u8>` cell alphas |

## How `crepuscularity-gpui` consumes themes

1. **Catalog JSON** — `catalog/themes/{dither-kit,kumo,night,chalk,aurora,dawn,zinc}.json`
   hold `background` / `foreground` / `muted` / `accent` plus per-color
   `seeds.{name}.{fill,line,star}` RGB triples (0–255).
2. **Compile-time seeds** — use `palette::DITHER_*` / `KUMO_*` for the common
   chart colours without parsing JSON in the hot path.
3. **Runtime theme pick** — load one theme name, map `seeds.blue.fill` →
   `gpui::Rgba` / `Hsla`, then pass into a canvas painter.
4. **Dither coverage** — call `sparkline_alphas(values, cols, rows, variant, intensity)`
   to get row-major alphas (`0..=255`). Paint each lit cell with the theme fill
   colour × alpha. Full `gpui::Canvas` painters stay in the host crate.

### Path-include (Zed workspace)

```rust
#[path = "vendor/crepuscularity-components/packages/gpui/mod.rs"]
mod crepus_components;
use crepus_components::{DITHER_BLUE_FILL, sparkline_alphas, Variant};
```

### Path dependency

```toml
[dependencies]
crepuscularity-components-gpui = { path = "plugins/crepuscularity-components/packages/gpui" }
```

```rust
use crepuscularity_components_gpui::{sparkline_alphas_for_size, Variant};

let (cols, rows, alphas) =
    sparkline_alphas_for_size(&[1.0, 4.0, 2.0, 5.0], 120.0, 40.0, Variant::Gradient, 0.0);
assert_eq!(alphas.len(), cols * rows);
```

## Themes

| Name | Role |
|------|------|
| `dither-kit` | green/blue/purple/pink/orange/red/grey seeds |
| `kumo` | Cloudflare Kumo categorical chart colours |
| `night` / `chalk` / `aurora` / `dawn` / `zinc` | surface themes (see JSON catalog) |

## Tests

```bash
cargo test --manifest-path packages/gpui/Cargo.toml
```
