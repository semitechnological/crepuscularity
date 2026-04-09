#[test]
fn ssr_renders_template() {
    use crepuscularity_core::{TemplateContext, TemplateValue};
    use std::collections::HashMap;

    let template = r#"div p-4
  "Hello {name}""#;
    let mut vars = HashMap::new();
    vars.insert("name".to_string(), TemplateValue::Str("World".to_string()));
    let ctx = TemplateContext {
        vars,
        base_dir: None,
        slot: None,
        virtual_files: HashMap::new(),
    };
    let html = crepuscularity_web::render_template_to_html(template, &ctx).unwrap();
    assert!(html.contains("Hello World"));
}
