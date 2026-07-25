# GPUI palette stubs

Minimal RGB seed constants for a future `crepuscularity-gpui` integration.
These mirror `catalog/themes/dither-kit.json` and Kumo categorical colours so
GPUI charts can share the same dither family without pulling Flutter/JS deps.

## Usage

Copy or path-include `palette.rs` into a Zed / GPUI crate:

```rust
use crepuscularity_components_gpui::{DITHER_BLUE_FILL, KUMO_BLUE};
```

Full Bayer paint + `gpui::Canvas` painters are intentionally deferred — this
directory only ships the seed arrays and theme names for now.

## Themes

| Name | Role |
|------|------|
| `dither-kit` | green/blue/purple/pink/orange/red/grey seeds |
| `kumo` | Cloudflare Kumo categorical chart colours |
| `night` / `chalk` / `aurora` | surface themes (see JSON catalog) |
