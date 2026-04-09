use std::collections::HashMap;

use crepuscularity_core::TemplateContext;
use serde_json::json;

use crate::{render_from_files, render_template_to_ir, to_json, ViewIr, IR_VERSION};

#[test]
fn plain_text_stack() {
    let mut ctx = TemplateContext::new();
    ctx.set("name", "Ada");
    let tpl = "div flex flex-col gap-4\n  span\n    \"Hello {name}\"";
    let ir = render_template_to_ir(tpl, &ctx).unwrap();
    assert_eq!(ir.version, IR_VERSION);

    let expected = json!({
        "version": IR_VERSION,
        "root": [{
            "kind": "stack",
            "axis": "column",
            "spacing": 16.0,
            "children": [{
                "kind": "text",
                "content": "Hello Ada"
            }]
        }]
    });
    let v: serde_json::Value = serde_json::to_value(&ir).unwrap();
    assert_eq!(v, expected);
}

fn round_trip(ir: &ViewIr) {
    let s = to_json(ir).unwrap();
    let back: ViewIr = serde_json::from_str(&s).unwrap();
    assert_eq!(*ir, back);
}

#[test]
fn serde_round_trip() {
    let mut ctx = TemplateContext::new();
    ctx.set("show", true);
    let ir = render_template_to_ir(
        "div flex flex-row\n if {show}\n  \"yes\"\n else\n  \"no\"",
        &ctx,
    )
    .unwrap();
    round_trip(&ir);
}

#[test]
fn for_loop() {
    let mut ctx = TemplateContext::new();
    let mut a = TemplateContext::new();
    a.set("value", "a");
    let mut b = TemplateContext::new();
    b.set("value", "b");
    ctx.set(
        "items",
        crepuscularity_core::TemplateValue::List(vec![a, b]),
    );
    let tpl = "div\n for item in {items}\n  span\n    \"{item}\"";
    let ir = render_template_to_ir(tpl, &ctx).unwrap();
    let v = serde_json::to_value(&ir).unwrap();
    assert_eq!(
        v["root"][0]["children"][0]["children"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    round_trip(&ir);
}

#[test]
fn include_virtual_file() {
    let mut ctx = TemplateContext::new();
    let mut files = HashMap::new();
    files.insert(
        "child.crepus".into(),
        "span text-green-400\n  \"In child\"".into(),
    );
    ctx.virtual_files = files;
    let tpl = "include child.crepus";
    let ir = render_template_to_ir(tpl, &ctx).unwrap();
    let s = serde_json::to_string(&ir).unwrap();
    assert!(s.contains("In child"));
    assert!(s.contains("#4ade80") || s.contains("green"));
}

#[test]
fn match_arm() {
    let mut ctx = TemplateContext::new();
    ctx.set("status", "on");
    let tpl = "div\n match {status}\n \"on\" =>\n  \"OK\"\n _ =>\n  \"?\"";
    let ir = render_template_to_ir(tpl, &ctx).unwrap();
    let v = serde_json::to_value(&ir).unwrap();
    assert!(v.to_string().contains("OK"), "expected OK in {}", v);
    round_trip(&ir);
}

#[test]
fn button_and_dynamic_color() {
    let mut ctx = TemplateContext::new();
    ctx.set("surface", "18181b");
    let tpl = "button @click=\"go\" bg-{surface}\n  \"Tap\"";
    let ir = render_template_to_ir(tpl, &ctx).unwrap();
    let v = serde_json::to_value(&ir).unwrap();
    assert_eq!(v["root"][0]["kind"], "button");
    assert_eq!(v["root"][0]["label"], "Tap");
    round_trip(&ir);
}

#[test]
fn render_from_files_entry() {
    let mut files = HashMap::new();
    files.insert("main.crepus".into(), "div\n  \"ok\"".into());
    let ir = render_from_files(&files, "main.crepus", &TemplateContext::new()).unwrap();
    assert_eq!(ir.root.len(), 1);
}
