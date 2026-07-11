use crepuscularity_core::analysis::{analyze_template, classify_node, Region};
use crepuscularity_core::ast::*;
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

#[test]
fn for_block_with_braced_iterator() {
    let nodes =
        parse_template("for tab in {self.visible_tabs()}\n  div px-2\n    \"{tab}\"").unwrap();
    assert_eq!(nodes.len(), 1);
    match &nodes[0] {
        Node::For(block) => {
            assert_eq!(block.pattern, "tab");
            assert_eq!(block.iterator, "self.visible_tabs()");
            assert_eq!(block.body.len(), 1);
        }
        other => panic!("expected For, got {other:?}"),
    }
}

#[test]
fn for_block_with_plain_identifier() {
    let nodes = parse_template("for item in items\n  div\n    \"{item}\"").unwrap();
    assert_eq!(nodes.len(), 1);
    match &nodes[0] {
        Node::For(block) => {
            assert_eq!(block.pattern, "item");
            assert_eq!(block.iterator, "items");
        }
        other => panic!("expected For, got {other:?}"),
    }
}

#[test]
fn event_handler_with_method_name() {
    let nodes = parse_template("div @mousedown=start_resize").unwrap();
    assert_eq!(nodes.len(), 1);
    match &nodes[0] {
        Node::Element(el) => {
            assert_eq!(el.event_handlers.len(), 1);
            assert_eq!(el.event_handlers[0].event, "mousedown");
            assert_eq!(el.event_handlers[0].handler, "start_resize");
        }
        other => panic!("expected Element, got {other:?}"),
    }
}

#[test]
fn event_handler_with_closure() {
    let nodes = parse_template(
        "div @mousedown={cx.listener(|this, _, window, cx| this.focus(id.clone(), window, cx))}",
    )
    .unwrap();
    assert_eq!(nodes.len(), 1);
    match &nodes[0] {
        Node::Element(el) => {
            assert_eq!(el.event_handlers.len(), 1);
            assert_eq!(el.event_handlers[0].event, "mousedown");
            assert!(el.event_handlers[0].handler.contains("cx.listener"));
            assert!(el.event_handlers[0].handler.contains("focus"));
        }
        other => panic!("expected Element, got {other:?}"),
    }
}

#[test]
fn if_block_with_negation() {
    let nodes = parse_template("if {!collapsed}\n  div\n    \"expanded\"").unwrap();
    assert_eq!(nodes.len(), 1);
    match &nodes[0] {
        Node::If(block) => {
            assert_eq!(block.condition, "!collapsed");
            assert_eq!(block.then_children.len(), 1);
        }
        other => panic!("expected If, got {other:?}"),
    }
}

#[test]
fn if_block_with_method_condition() {
    let nodes = parse_template("if {self.agents_collapsed}\n  div\n    \"collapsed\"").unwrap();
    assert_eq!(nodes.len(), 1);
    match &nodes[0] {
        Node::If(block) => {
            assert_eq!(block.condition, "self.agents_collapsed");
        }
        other => panic!("expected If, got {other:?}"),
    }
}

#[test]
fn nested_for_in_element() {
    let src = "div\n  for line in {frame.lines}\n    div\n      \"{line}\"";
    let nodes = parse_template(src).unwrap();
    assert_eq!(nodes.len(), 1);
    match &nodes[0] {
        Node::Element(el) => {
            assert_eq!(el.children.len(), 1);
            match &el.children[0] {
                Node::For(block) => {
                    assert_eq!(block.pattern, "line");
                    assert_eq!(block.iterator, "frame.lines");
                }
                other => panic!("expected For child, got {other:?}"),
            }
        }
        other => panic!("expected Element, got {other:?}"),
    }
}

#[test]
fn mixed_if_and_for_in_element() {
    let src = "div\n  if {show_tabs}\n    div\n      \"tabs\"\n  for tab in {tabs}\n    div\n      \"{tab}\"";
    let nodes = parse_template(src).unwrap();
    assert_eq!(nodes.len(), 1);
    match &nodes[0] {
        Node::Element(el) => {
            assert_eq!(el.children.len(), 2);
            assert!(matches!(el.children[0], Node::If(_)));
            assert!(matches!(el.children[1], Node::For(_)));
        }
        other => panic!("expected Element, got {other:?}"),
    }
}

#[test]
fn multiple_event_handlers() {
    let nodes = parse_template("div @mousedown=down @mouseup=up @mousemove=move").unwrap();
    assert_eq!(nodes.len(), 1);
    match &nodes[0] {
        Node::Element(el) => {
            assert_eq!(el.event_handlers.len(), 3);
            assert_eq!(el.event_handlers[0].event, "mousedown");
            assert_eq!(el.event_handlers[1].event, "mouseup");
            assert_eq!(el.event_handlers[2].event, "mousemove");
        }
        other => panic!("expected Element, got {other:?}"),
    }
}

#[test]
fn event_handler_with_modifiers() {
    let nodes = parse_template("div @keydown|ctrl+s=save").unwrap();
    assert_eq!(nodes.len(), 1);
    match &nodes[0] {
        Node::Element(el) => {
            assert_eq!(el.event_handlers.len(), 1);
            assert_eq!(el.event_handlers[0].event, "keydown");
            assert_eq!(el.event_handlers[0].modifiers, vec!["ctrl+s"]);
            assert_eq!(el.event_handlers[0].handler, "save");
        }
        other => panic!("expected Element, got {other:?}"),
    }
}

#[test]
fn conditional_class_attribute() {
    let nodes = parse_template("div class:focused={is_focused}").unwrap();
    assert_eq!(nodes.len(), 1);
    match &nodes[0] {
        Node::Element(el) => {
            assert_eq!(el.conditional_classes.len(), 1);
            assert_eq!(el.conditional_classes[0].class, "focused");
            assert_eq!(el.conditional_classes[0].condition, "is_focused");
        }
        other => panic!("expected Element, got {other:?}"),
    }
}
