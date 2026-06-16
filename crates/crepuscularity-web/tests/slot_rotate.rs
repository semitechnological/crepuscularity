use crepuscularity_core::context::TemplateContext;
use crepuscularity_web::render_template_to_html;

#[test]
fn slot_rotate_error_not_enough_children() {
    let tpl = "slot-rotate\n  \"one\"";
    let ctx = TemplateContext::new();
    let result = render_template_to_html(tpl, &ctx);

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("slot-rotate needs at least two plain-text phrase children"));
}

#[cfg(feature = "ssr")]
#[test]
fn slot_rotate_error_not_enough_children_ssr() {
    let tpl = "slot-rotate\n  \"one\"";
    let ctx = TemplateContext::new();
    let result = crepuscularity_web::render_template_to_html_with_ssr(tpl, &ctx, false);

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("slot-rotate needs at least two plain-text phrase children"));
}
