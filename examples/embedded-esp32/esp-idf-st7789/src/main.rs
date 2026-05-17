//! ESP-IDF sketch: wire your `esp_lcd` panel handle, then `ui.flush(&mut panel)`.
//!
//! Copy into an `esp-idf-template` app after SPI / ST7789 init.
//! Host static UI: `WRITE_FRAME=frame.bin cargo run -p embedded-stm32-host-sim`

use crepuscularity_embedded::panel::esp_idf::EspIdfLcdPanel;
use crepuscularity_embedded::{PanelPreset, Ui};

extern "C" {
    static CREPUS_LCD_PANEL: esp_idf_sys::esp_lcd_panel_handle_t;
}

fn main() {
    let template = include_str!("../../embedded-stm32/ui.crepus");
    let preset = PanelPreset::St7789_240x320;
    let mut ui = Ui::new(preset.width(), preset.height(), template)
        .panel(preset.config())
        .with("cpu", 42)
        .with("status", "ok");

    let mut panel = unsafe { EspIdfLcdPanel::new(CREPUS_LCD_PANEL, preset) };

    loop {
        ui.render().expect("render");
        ui.flush(&mut panel).expect("flush");
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
}
