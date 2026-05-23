use std::path::PathBuf;

use crepuscularity_core::TemplateContext;
use crepuscularity_lvgl::{render_template_to_lvgl_xml_with_options, LvglOptions, LvglRoot};

fn main() {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("manifest dir"));
    let template_path = manifest_dir.join("ui.crepus");
    let template = std::fs::read_to_string(&template_path).expect("read ui.crepus");

    let mut ctx = TemplateContext::new();
    ctx.set("device", "STM32F411");
    ctx.set("panel", "ILI9341");
    ctx.set("cpu", 72);
    ctx.set("status", "nominal");

    let xml = render_template_to_lvgl_xml_with_options(
        &template,
        &ctx,
        &LvglOptions {
            name: "Stm32Dashboard".into(),
            root: LvglRoot::Screen,
        },
    )
    .expect("render LVGL XML");

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").expect("out dir"));
    let out_file = out_dir.join("stm32_dashboard.xml");
    std::fs::write(&out_file, xml).expect("write LVGL XML");

    println!("cargo:rerun-if-changed={}", template_path.display());
}
