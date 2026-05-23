use crepuscularity_core::TemplateContext;
use crepuscularity_lvgl::{render_template_to_lvgl_xml_with_options, LvglOptions, LvglRoot};

const TEMPLATE: &str = r##"
div #dashboard w-full h-full flex flex-col gap-3 bg-[#101820] p-4
  h1 text-white text-lg
    "LVGL Pro {status}"
  div flex flex-row gap-2
    span text-zinc-100
      "CPU"
    progress #cpu value={cpu}
  button #refresh bg-blue-500 text-white rounded @click="refresh"
    "Refresh"
"##;

fn main() {
    let mut ctx = TemplateContext::new();
    ctx.set("status", "ready");
    ctx.set("cpu", 68);
    let xml = render_template_to_lvgl_xml_with_options(
        TEMPLATE,
        &ctx,
        &LvglOptions {
            name: "Dashboard".into(),
            root: LvglRoot::Component,
        },
    )
    .expect("example template should render to LVGL XML");
    println!("{xml}");
}
