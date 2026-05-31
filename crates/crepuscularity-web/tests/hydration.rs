//! Tests for the `hydration` feature flag.
#![cfg(feature = "hydration")]

use base64::{engine::general_purpose::STANDARD, Engine as _};
use crepuscularity_core::TemplateContext;
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
