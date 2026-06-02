use std::collections::HashMap;
use std::fs;

use crepuscularity_core::context::{TemplateContext, TemplateValue};
use serde_json::json;

use crate::{
    apply_mutations, ast_shape_compatible, diff_ir, plan_hot_reload, render_from_files,
    render_template_to_ir, to_json, HotReloadMessage, ViewIr, IR_VERSION,
};

#[test]
fn plain_text_stack() {
    let mut ctx = TemplateContext::new();
    ctx.set("name", "Ada");
    let tpl = "div flex flex-col gap-4\n  span\n    \"Hello {name}\"";
    let ir = render_template_to_ir(tpl, &ctx).unwrap();
    assert_eq!(ir.version, IR_VERSION);

    // flex-col explicitly sets flexDirection so web consumers see it alongside axis.
    let expected = json!({
        "version": IR_VERSION,
        "root": [{
            "kind": "stack",
            "axis": "column",
            "spacing": 16.0,
            "style": { "flexDirection": "column" },
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
    ctx.set("items", TemplateValue::List(vec![a, b]));
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
    ctx.virtual_files = std::sync::Arc::new(files);
    let tpl = "include child.crepus";
    let ir = render_template_to_ir(tpl, &ctx).unwrap();
    let s = serde_json::to_string(&ir).unwrap();
    assert!(s.contains("In child"));
    assert!(s.contains("#05df72") || s.contains("green"));
}

#[test]
fn file_include_rejects_parent_dir() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("templates");
    fs::create_dir(&root).unwrap();
    fs::write(temp.path().join("secret.crepus"), "div\n  \"secret\"").unwrap();

    let mut ctx = TemplateContext::new();
    ctx.base_dir = Some(root);
    let err = render_template_to_ir("include ../secret.crepus", &ctx).unwrap_err();
    assert!(
        err.contains("include path outside base dir"),
        "expected base-dir rejection, got: {err}"
    );
}

#[test]
fn file_include_rejects_absolute_path() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("templates");
    fs::create_dir(&root).unwrap();
    let secret = temp.path().join("secret.crepus");
    fs::write(&secret, "div\n  \"secret\"").unwrap();

    let mut ctx = TemplateContext::new();
    ctx.base_dir = Some(root);
    let err = render_template_to_ir(&format!("include {}", secret.display()), &ctx).unwrap_err();
    assert!(
        err.contains("include path outside base dir"),
        "expected absolute-path rejection, got: {err}"
    );
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
    assert_eq!(v["root"][0]["onClick"], "go");
    round_trip(&ir);
}

#[test]
fn dropzone_ir() {
    let tpl = r#"
dropzone @drop="files" accept="image/*,application/pdf" border rounded p-4
  span text-sm
    "Drop files"
"#;
    let ir = render_template_to_ir(tpl, &TemplateContext::new()).unwrap();
    let v = serde_json::to_value(&ir).unwrap();
    assert_eq!(v["root"][0]["kind"], "dropzone");
    assert_eq!(v["root"][0]["label"], "Drop files");
    assert_eq!(v["root"][0]["accept"], "image/*,application/pdf");
    assert_eq!(v["root"][0]["onDrop"], "files");
    round_trip(&ir);
}

#[test]
fn standard_controls_ir() {
    let tpl = r#"
div flex flex-col
  toggle bind=photos checked=true @change="sync.photos"
    "Photos"
  checkbox bind=clipboard checked=false
    "Clipboard"
  slider bind=quota value=25 min=0 max=100 step=5 label="Quota"
  progress value=62 max=100 label="Transfer"
  meter value=512 min=0 max=1024 label="Storage"
  badge tone="success"
    "Online"
  divider
  spacer size=16
"#;
    let ir = render_template_to_ir(tpl, &TemplateContext::new()).unwrap();
    let v = serde_json::to_value(&ir).unwrap();
    let children = &v["root"][0]["children"];
    assert_eq!(children[0]["kind"], "toggle");
    assert_eq!(children[0]["checked"], true);
    assert_eq!(children[0]["onChange"], "sync.photos");
    assert_eq!(children[1]["kind"], "checkbox");
    assert_eq!(children[2]["kind"], "slider");
    assert_eq!(children[2]["step"], 5.0);
    assert_eq!(children[3]["kind"], "progress");
    assert_eq!(children[4]["kind"], "meter");
    assert_eq!(children[5]["kind"], "badge");
    assert_eq!(children[6]["kind"], "divider");
    assert_eq!(children[7]["kind"], "spacer");
    round_trip(&ir);
}

#[test]
fn web_and_react_native_style_tags_lower_to_native_ir() {
    let tpl = r#"
<View class="flex flex-col gap-2">
  <Text class="text-lg font-semibold">Acme</Text>
  <p>Paragraph copy</p>
  <h1>Heading</h1>
  <ul>
    <li><span>One</span></li>
    <li><Text>Two</Text></li>
  </ul>
  <Switch checked={true} label="LAN sync" />
  <Progress value={80} max={100} label="Backup" />
</View>
"#;
    let ir = render_template_to_ir(tpl, &TemplateContext::new()).unwrap();
    let v = serde_json::to_value(&ir).unwrap();
    let root = &v["root"][0];
    assert_eq!(root["kind"], "stack");
    assert_eq!(root["children"][0]["kind"], "text");
    assert_eq!(root["children"][3]["kind"], "list");
    assert_eq!(root["children"][3]["children"][0]["kind"], "listItem");
    assert_eq!(root["children"][4]["kind"], "toggle");
    assert_eq!(root["children"][5]["kind"], "progress");
    round_trip(&ir);
}

#[test]
fn render_from_files_entry() {
    let mut files = HashMap::new();
    files.insert("main.crepus".into(), "div\n  \"ok\"".into());
    let ir = render_from_files(&files, "main.crepus", &TemplateContext::new()).unwrap();
    assert_eq!(ir.root.len(), 1);
}

#[test]
fn ir_patch_round_trip_text_update() {
    let old = render_template_to_ir("div\n  \"Hello\"", &TemplateContext::new()).unwrap();
    let new = render_template_to_ir("div\n  \"Hello world\"", &TemplateContext::new()).unwrap();

    let patch = diff_ir(&old, &new);
    assert!(!patch.is_empty());

    let mut applied = old.clone();
    apply_mutations(&mut applied, &patch).unwrap();
    assert_eq!(applied, new);
}

#[test]
fn ast_gate_rejects_control_flow_condition_changes() {
    let old = crepuscularity_core::parse_template("if {a}\n  \"x\"\nelse\n  \"y\"").unwrap();
    let new = crepuscularity_core::parse_template("if {b}\n  \"x\"\nelse\n  \"y\"").unwrap();
    assert!(!ast_shape_compatible(&old, &new));
}

#[test]
fn plan_hot_reload_returns_patch_for_literal_changes() {
    let msg = plan_hot_reload(
        "div\n  \"Hello\"",
        "div\n  \"Hello world\"",
        &TemplateContext::new(),
    );
    match msg {
        HotReloadMessage::Patch { mutations } => assert!(!mutations.is_empty()),
        other => panic!("expected Patch, got {other:?}"),
    }
}

#[test]
fn plan_hot_reload_falls_back_to_full_reload_for_semantic_changes() {
    let msg = plan_hot_reload(
        "if {a}\n  div\n    \"x\"",
        "if {b}\n  div\n    \"x\"",
        &TemplateContext::new(),
    );
    match msg {
        HotReloadMessage::FullReload { reason, .. } => {
            assert!(reason.contains("semantics"));
        }
        other => panic!("expected FullReload, got {other:?}"),
    }
}

#[test]
fn input_and_picker_ir() {
    let tpl = r#"
div flex flex-col
  input bind=note placeholder="Hi"
  picker bind=mode
    span value="matrix" "Matrix"
    span "Stalwart"
"#;
    let ir = render_template_to_ir(tpl, &TemplateContext::new()).unwrap();
    let v = serde_json::to_value(&ir).unwrap();
    assert_eq!(v["root"][0]["children"][0]["kind"], "input");
    assert_eq!(v["root"][0]["children"][1]["kind"], "picker");
}

#[test]
fn image_object_fit_and_position_classes() {
    let tpl = r#"
img object-cover object-right-top src="https://example.com/cat.png" placeholder="Loading image…"
"#;
    let ir = render_template_to_ir(tpl, &TemplateContext::new()).unwrap();
    let v = serde_json::to_value(&ir).unwrap();
    assert_eq!(v["root"][0]["kind"], "image");
    assert_eq!(v["root"][0]["style"]["objectFit"], "cover");
    assert_eq!(v["root"][0]["style"]["objectPosition"], "right-top");
    assert_eq!(v["root"][0]["placeholder"], "Loading image…");
}

#[test]
fn gradient_background_classes() {
    let tpl = r#"
div bg-gradient-to-r from-blue-500 to-red-500
  "Gradient"
"#;
    let ir = render_template_to_ir(tpl, &TemplateContext::new()).unwrap();
    let v = serde_json::to_value(&ir).unwrap();
    assert_eq!(v["root"][0]["kind"], "stack");
    assert_eq!(v["root"][0]["style"]["backgroundGradientDirection"], "to-r");
    assert_eq!(
        v["root"][0]["style"]["backgroundGradientFrom"],
        crate::colors::lookup_named_color("blue-500").unwrap()
    );
    assert_eq!(
        v["root"][0]["style"]["backgroundGradientTo"],
        crate::colors::lookup_named_color("red-500").unwrap()
    );
}
