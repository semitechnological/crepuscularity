use crepuscularity_core::ast::{Node, TextPart};
use crepuscularity_core::parse_template;

#[test]
fn parses_id_shorthand_on_any_tag() {
    let nodes = parse_template(r#"section #hero "Hello""#).unwrap();
    let Node::Element(el) = &nodes[0] else {
        panic!("expected element");
    };
    assert_eq!(el.tag, "section");
    assert_eq!(el.id.as_deref(), Some("hero"));
}

#[test]
fn parses_same_line_text_as_first_child() {
    let nodes = parse_template(r#"div #hero "Hello""#).unwrap();
    let Node::Element(el) = &nodes[0] else {
        panic!("expected element");
    };
    let Node::Text(parts) = &el.children[0] else {
        panic!("expected text child");
    };
    assert!(matches!(&parts[0], TextPart::Literal(text) if text == "Hello"));
}

#[test]
fn same_line_text_precedes_indented_children() {
    let nodes = parse_template(
        r#"div #hero "Hello"
  span
    "World""#,
    )
    .unwrap();
    let Node::Element(el) = &nodes[0] else {
        panic!("expected element");
    };
    assert!(matches!(&el.children[0], Node::Text(_)));
    assert!(matches!(&el.children[1], Node::Element(_)));
}
