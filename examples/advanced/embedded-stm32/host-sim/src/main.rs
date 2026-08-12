//! Host simulator for the STM32 ILI9341 integration (no board required).
//!
//! ```bash
//! cargo run -p embedded-stm32-host-sim
//! ```

use crepuscularity_embedded::{
    embedded_template, PanelPreset, Rgb565Display, ScreenSize, Ui, DEFAULT_BG,
};

struct RamDisplay {
    bytes: Vec<u8>,
    size: ScreenSize,
}

impl RamDisplay {
    fn new(size: ScreenSize) -> Self {
        let len = size.pixel_count() * 2;
        Self {
            bytes: vec![0; len],
            size,
        }
    }
}

impl Rgb565Display for RamDisplay {
    fn screen_size(&self) -> ScreenSize {
        self.size
    }

    fn flush_rgb565_rect(
        &mut self,
        _x: u16,
        _y: u16,
        w: u16,
        h: u16,
        pixels: &[u8],
    ) -> Result<(), crepuscularity_embedded::DisplayError> {
        assert_eq!(pixels.len(), (w as usize) * (h as usize) * 2);
        self.bytes.copy_from_slice(pixels);
        Ok(())
    }
}

fn main() {
    const W: u16 = 240;
    const H: u16 = 320;
    let template = embedded_template!("../ui.crepus");
    let mut ui = Ui::new(W, H, template)
        .panel(PanelPreset::Ili9341_240x320.config())
        .with("cpu", 55)
        .with("status", "ok");

    let mut panel = RamDisplay::new(ScreenSize::new(W, H));
    for cpu in [55i64, 72, 91, 48] {
        ui.set("cpu", cpu);
        ui.flush(&mut panel).expect("flush");
    }

    assert_eq!(panel.bytes.len(), (W as usize) * (H as usize) * 2);
    assert!(panel.bytes.iter().any(|b| *b != 0));

    if let Ok(path) = std::env::var("WRITE_FRAME") {
        std::fs::write(&path, &panel.bytes).expect("write frame.bin");
        println!(
            "wrote {} ({} bytes) for STM32 include_bytes!",
            path,
            panel.bytes.len()
        );
    }

    println!(
        "embedded-stm32-host-sim ok: {}x{} panel RAM ({} bytes), same path as STM32 firmware",
        W,
        H,
        panel.bytes.len()
    );
    let _ = DEFAULT_BG;
}
