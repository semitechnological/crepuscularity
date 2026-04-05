use crepuscularity_core::analysis::{analyze_template, classify_node, Region};
use crepuscularity_core::parser::parse_template;

#[test]
fn static_literal_text_is_static() {
    let nodes = parse_template("div\n  \"hello world\"").unwrap();
    let map = analyze_template(&nodes);
    assert_eq!(map.expr_count, 0);
    assert!(!map.static_indices.is_empty());
    assert!(map.dynamic_indices.is_empty());
}

#[test]
fn interpolated_text_is_dynamic() {
    let nodes = parse_template("div\n  \"hello {name}\"").unwrap();
    let map = analyze_template(&nodes);
    assert!(map.expr_count > 0);
    assert!(!map.dynamic_indices.is_empty());
}

#[test]
fn if_block_is_dynamic() {
    let nodes = parse_template("if {x}\n  div\n    \"yes\"").unwrap();
    let map = analyze_template(&nodes);
    assert!(!map.dynamic_indices.is_empty());
}

#[test]
fn for_block_is_dynamic() {
    let nodes = parse_template("for item in {list}\n  div\n    {item}").unwrap();
    let map = analyze_template(&nodes);
    assert!(!map.dynamic_indices.is_empty());
}

#[test]
fn element_with_binding_is_dynamic() {
    // bind: prefix is the DSL syntax for dynamic attribute bindings
    let nodes = parse_template("div bind:href={url}").unwrap();
    let map = analyze_template(&nodes);
    assert!(!map.dynamic_indices.is_empty());
}

#[test]
fn classify_node_pure_element_is_static() {
    let nodes = parse_template("div class-name\n  \"static text\"").unwrap();
    assert_eq!(classify_node(&nodes[0]), Region::Static);
}
