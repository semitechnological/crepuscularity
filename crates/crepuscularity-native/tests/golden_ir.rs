use crepuscularity_core::context::TemplateContext;
use crepuscularity_native::{render_template_to_ir, IR_VERSION};

#[test]
fn golden_hello_matches_fixture_json() {
    let tpl = include_str!("fixtures/hello.crepus");
    let expected: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/hello.json")).expect("fixture JSON");
    let ir = render_template_to_ir(tpl, &TemplateContext::new()).expect("render template");
    assert_eq!(ir.version, IR_VERSION);
    let got = serde_json::to_value(&ir).expect("ir to Value");
    assert_eq!(got, expected);
}
