use crepuscularity_core::ast::Node;
use crepuscularity_core::parser::parse_template;

#[test]
fn parse_template_strips_google_font_and_expands_aliases() {
    let src = r#"google-font Inter

div center
  "hi"
.center items-center justify-center flex
"#;
    let nodes = parse_template(src).unwrap();
    let Node::Element(el) = &nodes[0] else {
        panic!("expected root element");
    };
    assert_eq!(el.tag, "div");
    assert!(el.classes.contains(&"items-center".to_string()));
    assert!(el.classes.contains(&"justify-center".to_string()));
    assert!(el.classes.contains(&"flex".to_string()));
    assert!(!el.classes.iter().any(|c| c == "center"));
}
