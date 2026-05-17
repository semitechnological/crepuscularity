# ESP32 + SPI TFT (ST7789 / ILI9341)

> **UNSTABLE** — same host → device workflow as [`embedded-stm32`](../embedded-stm32/).

## Stack

| Layer | Crate / feature |
| --- | --- |
| UI | [`crepuscularity-embedded`](../../crates/crepuscularity-embedded/) |
| Panel | `crepuscularity-embedded` feature **`esp-idf`** |
| Driver | ESP-IDF `esp_lcd` (you init in board code) |

## Host (no ESP32 required)

```bash
cargo run -p embedded-stm32-host-sim
WRITE_FRAME=embedded-esp32/esp-idf-st7789/frame.bin cargo run -p embedded-stm32-host-sim
```

For ST7789, use [`PanelPreset::St7789_240x320`](../../crates/crepuscularity-embedded/src/panel/preset.rs) on the host:

```rust
use crepuscularity_embedded::{PanelPreset, Ui};

Ui::new(240, 320, template).panel(PanelPreset::St7789_240x320.config());
```

## On-device (ESP-IDF)

```toml
[dependencies]
crepuscularity-embedded = { path = "../../crates/crepuscularity-embedded", features = ["std", "esp-idf"] }
esp-idf-sys = "0.36"
```

```rust
use crepuscularity_embedded::panel::esp_idf::EspIdfLcdPanel;
use crepuscularity_embedded::{PanelPreset, Ui};

let mut panel = unsafe { EspIdfLcdPanel::new(panel_handle, PanelPreset::St7789_240x320) };
ui.render()?;
ui.flush(&mut panel)?;
```

See [`esp-idf-st7789/`](esp-idf-st7789/) for a minimal `main.rs` sketch.
