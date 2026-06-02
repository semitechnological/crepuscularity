//! Tests for the `hydration` feature flag.
#![cfg(feature = "hydration")]

use base64::{engine::general_purpose::STANDARD, Engine as _};
use crepuscularity_core::TemplateContext;
use crepuscularity_core::TemplateValue;
use crepuscularity_web::render_template_to_html_with_hydration;

use serde_json::Value;

fn hydration_context_json(html: &str) -> String {
    let needle = "id=\"__crepus_hydration__\"";
    let start = html.find(needle).expect("hydration context script");
    let after = &html[start..];
    let b64_start = after.find('>').expect("script open") + 1;
    let b64_end = after[b64_start..].find("</script>").expect("script close") + b64_start;
    let b64 = &after[b64_start..b64_end];
    String::from_utf8(STANDARD.decode(b64).expect("base64 context")).expect("utf-8 context")
}

fn hydration_context_value(html: &str) -> Value {
    serde_json::from_str(&hydration_context_json(html)).expect("hydration json")
}

#[test]
fn hydration_injects_root_marker() {
    let tpl = r#"div p-4
  "Hello {name}""#;
    let mut ctx = TemplateContext::new();
    ctx.set("name", "World");
    let html = render_template_to_html_with_hydration(tpl, &ctx).unwrap();
    assert!(
        html.contains("data-crepus-root"),
        "expected data-crepus-root, got: {html}"
    );
    assert!(
        html.contains("id=\"__crepus_hydration__\""),
        "expected __crepus_hydration__, got: {html}"
    );
    assert!(
        html.contains("type=\"application/json\""),
        "expected non-executable hydration script, got: {html}"
    );
}

#[test]
fn hydration_injects_dynamic_id() {
    let tpl = r#"div
  p
    "Count: {count}""#;
    let mut ctx = TemplateContext::new();
    ctx.set("count", 42i64);
    let html = render_template_to_html_with_hydration(tpl, &ctx).unwrap();
    assert!(
        html.contains("data-crepus-id="),
        "expected data-crepus-id on dynamic node, got: {html}"
    );
}

#[test]
fn hydration_ctx_json_contains_vars() {
    let tpl = r#"div
  "Hello""#;
    let mut ctx = TemplateContext::new();
    ctx.set("city", "London");
    ctx.set("temp", 14i64);
    let html = render_template_to_html_with_hydration(tpl, &ctx).unwrap();
    let value = hydration_context_value(&html);
    assert!(
        value["ctx"].get("city").is_some(),
        "expected city key in ctx JSON, got: {value}"
    );
    assert!(
        value["ctx"]["city"] == "London",
        "expected London value in ctx JSON, got: {value}"
    );
}

#[test]
fn hydration_payload_uses_canonical_manifest_shape() {
    let tpl = r#"button @click="increment"
  "Count: {count}""#;
    let mut ctx = TemplateContext::new();
    ctx.set("count", 1i64);

    let html = render_template_to_html_with_hydration(tpl, &ctx).unwrap();
    let value = hydration_context_value(&html);

    assert_eq!(value["v"], 1);
    assert!(value["ctx"].is_object());
    assert!(value["bind"].is_object());
    assert_eq!(value["ctx"]["count"], 1);
    assert!(html.contains("data-onclick=\"increment\""));
}

#[test]
fn hydration_context_does_not_allow_script_breakout() {
    let tpl = r#"div
  "Hello""#;
    let mut ctx = TemplateContext::new();
    ctx.set("payload", "</script><script>alert(1)</script>\u{2028}<>&");
    let html = render_template_to_html_with_hydration(tpl, &ctx).unwrap();
    assert!(
        !html.contains("</script><script>"),
        "raw script breakout sequence leaked into hydration HTML: {html}"
    );
    assert!(
        !html.contains("window.__crepus_ctx__"),
        "hydration context must not be emitted as executable JavaScript: {html}"
    );
    let json = hydration_context_json(&html);
    assert!(json.contains("</script><script>alert(1)</script>"));
}

// ── Golden DOM / event reattachment tests ─────────────────────────────────

#[test]
fn golden_static_element_output() {
    let tpl = "div w-full bg-white p-4\n  h1 text-xl font-bold\n    \"Welcome\"\n  span text-sm text-gray-500\n    \"Subtitle text\"";
    let ctx = TemplateContext::new();
    let html = render_template_to_html_with_hydration(tpl, &ctx).unwrap();

    // Root element carries boolean data-crepus-root attribute
    assert!(html.contains("data-crepus-root"), "missing root marker");
    // Static children should NOT carry data-crepus-id (no bindings)
    assert!(html.contains("<h1"), "missing h1");
    assert!(html.contains("Welcome"), "missing text content");
    assert!(html.contains("Subtitle text"), "missing subtitle");
    // Hydration payload must be present
    assert!(
        html.contains("__crepus_hydration__"),
        "missing hydration payload"
    );
    // No dynamic binding markers on fully static template
    assert!(
        !html.contains("data-crepus-id="),
        "static template should not have binding markers"
    );
}

#[test]
fn golden_dynamic_text_with_binding() {
    let tpl = "div\n  span\n    \"Count: {count}\"";
    let mut ctx = TemplateContext::new();
    ctx.set("count", 42i64);
    let html = render_template_to_html_with_hydration(tpl, &ctx).unwrap();

    assert!(html.contains("Count: 42"), "missing rendered value");
    assert!(html.contains("data-crepus-root"), "missing root marker");
    // At least one element should have a binding marker
    assert!(
        html.contains("data-crepus-id="),
        "missing id for client reattachment"
    );
}

#[test]
fn golden_event_handler_emits_data_attribute() {
    let tpl = "button @click=\"increment\" class:hidden={hidden}\n  \"Click me\"";
    let mut ctx = TemplateContext::new();
    ctx.set("hidden", false);
    let html = render_template_to_html_with_hydration(tpl, &ctx).unwrap();

    assert!(
        html.contains("data-onclick=\"increment\""),
        "missing onclick data attribute"
    );
    assert!(html.contains("Click me"), "missing button text");
    // Conditional class binding should produce a marker
    assert!(
        html.contains("data-crepus-id="),
        "missing binding marker for conditional class"
    );
}

#[test]
fn golden_for_loop_output() {
    // For loop with the conventional "value" key — render_for uses
    // item_ctx.get_str("value") to extract the loop variable.
    let tpl = "div\n for item in {items}\n  span item-row\n    \"{item}\"";
    let mut ctx = TemplateContext::new();
    let mut item1 = TemplateContext::new();
    item1.set("value", "A");
    let mut item2 = TemplateContext::new();
    item2.set("value", "B");
    ctx.set("items", TemplateValue::List(vec![item1, item2]));
    let html = render_template_to_html_with_hydration(tpl, &ctx).unwrap();

    assert!(html.contains("A"), "missing first item, html: {html}");
    assert!(html.contains("B"), "missing second item");
    assert!(html.contains("data-crepus-root"), "missing root marker");
}

#[test]
fn hydration_payload_serializes_bind_map() {
    let tpl = "div\n  input type=\"text\" value={name}\n  span\n    \"Hello {name}\"";
    let mut ctx = TemplateContext::new();
    ctx.set("name", "World");
    let html = render_template_to_html_with_hydration(tpl, &ctx).unwrap();
    let value = hydration_context_value(&html);

    assert_eq!(value["v"], 1, "version must be 1");
    assert!(value["ctx"].is_object(), "ctx must be an object");
    // Context must carry the variable value
    assert_eq!(value["ctx"]["name"], "World");
    // Hydration payload exists
    assert!(
        html.contains("__crepus_hydration__"),
        "missing hydration script"
    );
}
