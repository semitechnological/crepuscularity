use std::fs;

use crepuscularity_core::TemplateContext;
use crepuscularity_web::render_template_to_html;

#[test]
fn file_include_rejects_parent_dir() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("templates");
    fs::create_dir(&root).unwrap();
    fs::write(temp.path().join("secret.crepus"), "div\n  \"secret\"").unwrap();

    let mut ctx = TemplateContext::new();
    ctx.base_dir = Some(root);
    let err = render_template_to_html("include ../secret.crepus", &ctx)
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
    let err = render_template_to_html(&format!("include {}", secret.display()), &ctx)
        .unwrap_err()
        .to_string();
    assert!(
        err.contains("include path outside base dir"),
        "expected absolute-path rejection, got: {err}"
    );
}
