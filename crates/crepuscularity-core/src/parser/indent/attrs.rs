//! Attribute-only element merging.

use crate::ast::*;

use super::element::parse_element_line;

pub(crate) fn merge_attr_only_children(mut el: Element) -> Element {
    let mut kept = Vec::new();
    for child in std::mem::take(&mut el.children) {
        if let Node::Element(c) = child {
            if is_attr_only_element(&c) {
                hoist_attr_element(&mut el, &c);
                continue;
            }
            kept.push(Node::Element(merge_attr_only_children(c)));
        } else {
            kept.push(child);
        }
    }
    el.children = kept;
    el
}

fn is_attr_only_element(el: &Element) -> bool {
    el.tag.contains('=')
        && el.classes.is_empty()
        && el.children.is_empty()
        && el.bindings.is_empty()
}

fn hoist_attr_element(parent: &mut Element, attr_el: &Element) {
    let fake = parse_element_line(&format!("div {}", attr_el.tag), vec![]);
    parent.bindings.extend(fake.bindings);
    if let Some(id) = fake.id {
        parent.id = Some(id);
    }
    parent.classes.extend(fake.classes);
}
