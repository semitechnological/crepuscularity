//! Regression: named components must forward list props into `for` loops correctly.

use std::collections::HashMap;

use crepuscularity_core::TemplateContext;
use crepuscularity_web::render_from_files;

const SERVICES_CREPUS: &str = include_str!("../../../templates/site/elements/services.crepus");

#[test]
fn services_named_component_renders_item_titles() {
    let mut files = HashMap::new();
    files.insert(
        "elements/services.crepus".to_string(),
        SERVICES_CREPUS.to_string(),
    );
    files.insert(
        "page.crepus".to_string(),
        r#"--- Page
html
  body
    include elements/services.crepus#Services eyebrow="" title="Section" description="" services={svc}
"#
        .to_string(),
    );

    let mut s1 = TemplateContext::new();
    s1.set("title", "Launch setup");
    s1.set("description", "d1");
    s1.set("icon", "Popular");
    let mut s2 = TemplateContext::new();
    s2.set("title", "Ongoing");
    s2.set("description", "d2");
    s2.set("icon", "");

    let mut ctx = TemplateContext::new();
    ctx.set("svc", vec![s1, s2]);

    let html = render_from_files(&files, "page.crepus#Page", &ctx).expect("render");
    assert!(
        html.contains("Launch setup"),
        "expected service title in HTML, got snippet: {}",
        html.chars().take(2000).collect::<String>()
    );
}
