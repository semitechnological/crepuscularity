use crepuscularity_core::TemplateContext;
use crepuscularity_web::render_template_to_html;

#[test]
fn for_loop_merges_list_item_fields() {
    let tpl = r#"
section
  for item in {items}
    p
      {label}
"#;
    let mut row = TemplateContext::new();
    row.set("label", "hello");
    let mut ctx = TemplateContext::new();
    ctx.set("items", vec![row]);
    let out = render_template_to_html(tpl, &ctx).unwrap();
    assert!(out.contains("hello"), "got: {out}");
}

#[test]
fn for_loop_sees_parent_keys_overwritten_by_item() {
    let tpl = r#"
div
  for item in {items}
    span
      {title}
"#;
    let mut row = TemplateContext::new();
    row.set("title", "row-title");
    let mut ctx = TemplateContext::new();
    ctx.set("title", "outer-title");
    ctx.set("items", vec![row]);
    let out = render_template_to_html(tpl, &ctx).unwrap();
    assert!(out.contains("row-title"), "got: {out}");
    assert!(!out.contains("outer-title"));
}
