use crepuscularity_core::TemplateContext;
use crepuscularity_web::render_template_to_html;

#[test]
fn renders_id_shorthand_and_same_line_text() {
    let tpl = r#"
div #hero "Hi"
  span
    "there"
"#;
    let html = render_template_to_html(tpl, &TemplateContext::new()).unwrap();
    assert!(
        html.contains(r#"<div id="hero">Hi<span>there</span></div>"#),
        "{html}"
    );
}

#[test]
fn preserves_same_line_text_before_indented_children() {
    let tpl = r#"
div #hero "Hi"
  span
    "there"
"#;
    let html = render_template_to_html(tpl, &TemplateContext::new()).unwrap();
    let hi_ix = html.find("Hi").unwrap();
    let span_ix = html.find("<span>").unwrap();
    assert!(hi_ix < span_ix, "{html}");
}

#[test]
fn quoted_event_handlers_render_without_quotes() {
    let tpl = r#"
button #status @click="on_refresh_status"
  "Click"
"#;
    let html = render_template_to_html(tpl, &TemplateContext::new()).unwrap();
    assert!(
        html.contains(r#"data-onclick="on_refresh_status""#),
        "{html}"
    );
}
