use crepuscularity_embedded::{ui, Ui};

const UI: &str = include_str!("../ui.crepus");

#[test]
fn ui_macro_renders_dashboard() {
    let mut ui = ui!(UI, 128, 64, "cpu" => 10, "status" => "test");
    ui.render().expect("render");
    assert_eq!(ui.rgb565().len(), 128 * 64 * 2);
    assert!(ui.document().unwrap().node_by_id("status-label").is_some());
}

#[test]
fn ui_builder_with_method() {
    let mut ui = Ui::new(64, 32, "motion w-full h-full bg-zinc-950\n  \"x\"").with("cpu", 1);
    ui.render().expect("render");
}
