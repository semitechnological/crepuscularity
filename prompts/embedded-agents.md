# crepus embedded — LVGL firmware + embedded displays

Render `.crepus` templates on embedded MCUs (STM32, ESP32) via LVGL XML or direct RGB565 framebuffer.

**UNSTABLE — still in development; largely untested on real hardware.**

## Quick start

```bash
crepus embedded check ui.crepus                  # validate template
crepus embedded snapshot ui.crepus --width 240 --height 320 --out preview.ppm
```

## Template validation (CI)

```bash
crepus embedded check app.crepus
crepus embedded check app.crepus --component MyWidget
```

Validates parsing, includes, and variable resolution without hardware.

## PPM snapshots (debug)

```bash
crepus embedded snapshot ui.crepus -W 240 -H 320 --out out.ppm
```

Use for visual debugging in CI or during development.

## Rust API (firmware)

```rust
use crepuscularity_embedded::Ui;

const UI: &str = "div w-full h-full\n  span #temp\n    \"{temp}\"";
let mut ui = Ui::new(240, 320, UI).with("temp", 72);
ui.render().unwrap();
let bytes = ui.rgb565();    // framebuffer ready for DMA
let hit = ui.hit(10, 10);   // hit-testing
```

## no_std / display RAM

```rust
use crepuscularity_embedded::{Rgb565View, ScreenSize, Template};

let screen = ScreenSize::new(128, 64);
let mut ram = [0u16; 128 * 64];
let mut fb = Rgb565View::new(screen, &mut ram).unwrap();
let mut ui = Template::from_source("div w-full h-full\n  \"Hi\"", screen);
ui.draw(&mut fb).unwrap();
// ram ready for DMA
```

## LVGL XML output

```rust
use crepuscularity_lvgl::{
    render_template_to_lvgl_xml,
    LvglOptions, LvglRoot,
};
```

LVGL root types: `Component` (default), `Screen`.

## Features

- `std` — template parsing, includes (default)
- `no_std` — layout + paint only (build from host, embed pre-rendered)
- `mipidsi` — display-interface SPI support
- `embassy-stm32` — STM32 HAL integration
- `esp-idf` — ESP32 IDF integration
- `macros` — `ui!()` macro for inline template+variables

## Key crates

- `crepuscularity-embedded` — RGB565 framebuffer rendering, Ui/Template
- `crepuscularity-lvgl` — LVGL XML generation
- `crepuscularity-core` — parser, AST