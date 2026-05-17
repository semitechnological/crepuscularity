//! Firmware-style demo: [`crepuscularity_embedded::Ui`] + `ui!` macro.
//!
//! On hardware (STM32 + ILI9341, ESP32 + ST7789, …) you would call `panel.flush(ui.rgb565())`
//! after `render()` — see [`README.md`](../README.md) for SPI/LTDC/ESP-LCD wiring.
//!
//! ```bash
//! cargo test -p embedded-dashboard
//! cargo run -p embedded-dashboard
//! ```

use crepuscularity_embedded::ui;

const UI: &str = include_str!("../ui.crepus");

fn main() {
    let mut ui = ui!(UI, 128, 64, "cpu" => 42, "status" => "nominal");

    // Simulated frames (sensor poll → render → flush display)
    for (frame, cpu) in [42i64, 67, 91, 73].into_iter().enumerate() {
        let status = if cpu > 85 { "hot" } else { "ok" };
        ui.set("cpu", cpu).set("status", status);
        ui.render().unwrap_or_else(|e| panic!("frame {frame}: {e}"));

        let label = ui
            .document()
            .and_then(|d| d.node_by_id("cpu-value"))
            .and_then(|n| n.text.clone())
            .unwrap_or_default();
        assert!(
            label.contains(&cpu.to_string()),
            "frame {frame}: cpu label should show {cpu}%, got {label:?}"
        );

        let bytes = ui.rgb565();
        assert_eq!(bytes.len(), 128 * 64 * 2);
        assert!(
            bytes.iter().any(|b| *b != 0),
            "frame {frame}: empty framebuffer"
        );
    }

    // Hit-testing: deepest `#id` under a point (touch coordinates in real firmware)
    if let Some(doc) = ui.document() {
        let cpu_node = doc.node_by_id("cpu-value").expect("cpu-value id");
        let cx = cpu_node.bounds.x + cpu_node.bounds.w / 2;
        let cy = cpu_node.bounds.y + cpu_node.bounds.h / 2;
        assert_eq!(ui.hit(cx, cy), Some("cpu-value"));
    }

    println!(
        "embedded-dashboard ok: {}x{} RGB565 ({} bytes), 4 frames rendered",
        ui.screen().width,
        ui.screen().height,
        ui.rgb565().len()
    );
}
