use std::collections::HashMap;
use std::fs;

use crepuscularity_core::context::{TemplateContext, TemplateValue};
use serde_json::json;

use crate::{
    apply_mutations, ast_shape_compatible, diff_ir, generate_native_source, plan_hot_reload,
    render_from_files, render_template_to_ir, to_json, HotReloadMessage, NativeCodegenTarget,
    ViewIr, IR_VERSION,
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

#[test]
fn codegen_swiftui_emits_standalone_view_source() {
    let ir = render_template_to_ir(
        "div flex flex-col gap-4 p-4 bg-blue-500\n  span text-lg font-bold text-white\n    \"Hello Ada\"",
        &TemplateContext::new(),
    )
    .unwrap();
    let source = generate_native_source(&ir, NativeCodegenTarget::SwiftUi, "HelloScreen");
    assert!(source.contains("import SwiftUI"));
    assert!(source.contains("public enum CrepusActions"));
    assert!(source.contains("public static var dispatch: (String) -> String"));
    assert!(source.contains("public struct HelloScreen: View"));
    assert!(source.contains("VStack(alignment: .leading, spacing: 16.0)"));
    assert!(source.contains("Text(\"Hello Ada\")"));
    assert!(source.contains(".fontWeight(.bold)"));
    assert!(source.contains("Color(red:"));
}

#[test]
fn codegen_compose_emits_composable_source() {
    let ir = render_template_to_ir(
        "div flex flex-row gap-2 p-2\n  span text-lg font-bold\n    \"Hello Ada\"\n  button @click=\"tap\"\n    \"Tap\"",
        &TemplateContext::new(),
    )
    .unwrap();
    let source = generate_native_source(&ir, NativeCodegenTarget::Compose, "HelloScreen");
    assert!(source.contains("@Composable"));
    assert!(source.contains("fun HelloScreen(modifier: Modifier = Modifier)"));
    assert!(source.contains("Row(modifier = modifier.padding(8.dp), horizontalArrangement = Arrangement.spacedBy(8.dp))"));
    assert!(
        source.contains("Text(\"Hello Ada\", fontSize = 18.0.sp, fontWeight = FontWeight.Bold)")
    );
    assert!(source.contains("object CrepusActions"));
    assert!(source.contains("var dispatch: (String) -> String"));
    assert!(source.contains("Button(onClick = { CrepusActions.perform(\"tap\") })"));
}

#[test]
fn web_style_classes_lower_to_typed_native_codegen() {
    let ir = render_template_to_ir(
        "div flex flex-col w-full h-full px-4 pt-6 mb-2 bg-[#101624] border border-[#334155] rounded-xl opacity-75 shadow-lg translate-x-2 translate-y-1 rotate-6 scale-95 overflow-hidden\n  span text-center text-lg font-semibold italic underline line-through leading-tight line-clamp-2 text-[#f8fafc]\n    \"Literal web-style UI\"",
        &TemplateContext::new(),
    )
    .unwrap();
    let value = serde_json::to_value(&ir).unwrap();
    let root_style = &value["root"][0]["style"];
    assert_eq!(root_style["width"], -1.0);
    assert_eq!(root_style["height"], -1.0);
    assert_eq!(root_style["paddingHorizontal"], 16.0);
    assert_eq!(root_style["paddingTop"], 24.0);
    assert_eq!(root_style["marginBottom"], 8.0);
    assert_eq!(root_style["borderWidth"], 1.0);
    assert_eq!(root_style["opacity"], 0.75);
    assert_eq!(root_style["translateX"], 8.0);
    assert_eq!(root_style["translateY"], 4.0);
    assert_eq!(root_style["rotate"], 6.0);
    assert!((root_style["scaleX"].as_f64().unwrap() - 0.95).abs() < 0.001);
    assert!((root_style["scaleY"].as_f64().unwrap() - 0.95).abs() < 0.001);
    let text_style = &value["root"][0]["children"][0]["style"];
    assert_eq!(text_style["textAlign"], "center");
    assert_eq!(text_style["italic"], true);
    assert_eq!(text_style["underline"], true);
    assert_eq!(text_style["strikethrough"], true);
    assert_eq!(text_style["lineClamp"], 2);

    let swift = generate_native_source(&ir, NativeCodegenTarget::SwiftUi, "WebParityView");
    assert!(swift.contains(".padding(.horizontal, 16)"));
    assert!(swift.contains(".padding(.top, 24)"));
    assert!(swift.contains(".padding(.bottom, 8)"));
    assert!(swift.contains(".opacity(0.750)"));
    assert!(swift.contains(".border(Color(red:"));
    assert!(swift.contains(".offset(x: 8.0, y: 4.0)"));
    assert!(swift.contains(".rotationEffect(.degrees(6.0))"));
    assert!(swift.contains(".scaleEffect(x: 0.950, y: 0.950)"));
    assert!(swift.contains(".multilineTextAlignment(.center)"));
    assert!(swift.contains(".italic()"));
    assert!(swift.contains(".underline()"));
    assert!(swift.contains(".strikethrough()"));
    assert!(swift.contains(".lineLimit(2)"));

    let compose = generate_native_source(&ir, NativeCodegenTarget::Compose, "WebParityView");
    assert!(compose.contains(".padding(horizontal = 16.dp, vertical = 0.dp)"));
    assert!(compose.contains(".padding(start = 0.dp, top = 24.dp, end = 0.dp, bottom = 0.dp)"));
    assert!(compose.contains(".padding(start = 0.dp, top = 0.dp, end = 0.dp, bottom = 8.dp)"));
    assert!(compose.contains(".border(1.dp, Color(0xFF334155), RoundedCornerShape(12.dp))"));
    assert!(compose.contains(".alpha(0.750f)"));
    assert!(compose.contains(".offset(x = 8.dp, y = 4.dp)"));
    assert!(compose.contains(".rotate(6.0f)"));
    assert!(compose.contains(".scale(scaleX = 0.950f, scaleY = 0.950f)"));
    assert!(compose.contains("textAlign = TextAlign.Center"));
    assert!(compose.contains("fontStyle = FontStyle.Italic"));
    assert!(compose.contains("TextDecoration.combine"));
    assert!(compose.contains("maxLines = 2"));
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
    let err = render_template_to_ir("include ../secret.crepus", &ctx)
        .unwrap_err()
        .to_string();
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
    let err = render_template_to_ir(&format!("include {}", secret.display()), &ctx)
        .unwrap_err()
        .to_string();
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
  <Text class="text-lg font-semibold">Cupboard</Text>
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
