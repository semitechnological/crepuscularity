//! Tests for the `hydration` feature flag.
#![cfg(feature = "hydration")]

use crepuscularity_core::TemplateContext;
use crepuscularity_web::render_template_to_html_with_hydration;

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
        html.contains("__crepus_ctx__"),
        "expected __crepus_ctx__, got: {html}"
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
    assert!(
        html.contains("\"city\""),
        "expected city key in ctx JSON, got: {html}"
    );
    assert!(
        html.contains("\"London\""),
        "expected London value in ctx JSON, got: {html}"
    );
}
