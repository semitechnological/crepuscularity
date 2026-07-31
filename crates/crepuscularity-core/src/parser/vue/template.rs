//! Vue template block parsing into the shared AST.

use crate::ast::*;

use super::super::jsx::{jsx_close, jsx_err, jsx_parse_attrs};
use super::super::RawParseError;
use super::builders::{vue_build_element, VueBranch, VueElement};

const VOID_ELEMENTS: [&str; 14] = [
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
    "track", "wbr",
];

const MAX_VUE_DEPTH: usize = 256;

pub(crate) fn parse_vue_nodes<'a>(
    root: &'a str,
    src: &'a str,
    depth: usize,
) -> Result<(Vec<Node>, &'a str), RawParseError> {
    if depth > MAX_VUE_DEPTH {
        return Err(jsx_err(
            root,
            src,
            format!("maximum Vue template nesting depth ({MAX_VUE_DEPTH}) exceeded"),
        ));
    }

    let mut nodes: Vec<Node> = Vec::new();
    let mut rest = src;

    loop {
        let t = rest.trim_start();
        if t.is_empty() {
            rest = t;
            break;
        }
        if t.starts_with("</") {
            rest = t;
            break;
        }
        if let Some(after) = t.strip_prefix("<!--") {
            rest = match after.find("-->") {
                Some(end) => &after[end + 3..],
                None => "",
            };
            continue;
        }
        if t.starts_with('<') {
            let (node, next) = parse_vue_tag(root, t, depth)?;
            push_node(&mut nodes, node, root, t)?;
            rest = next;
            continue;
        }

        let (text, next) = vue_text_node(rest);
        if let Some(text) = text {
            nodes.push(text);
        }
        if next.len() == rest.len() {
            let skip = next
                .char_indices()
                .nth(1)
                .map(|(i, _)| i)
                .unwrap_or(next.len());
            rest = &next[skip..];
        } else {
            rest = next;
        }
    }

    Ok((nodes, rest))
}

/// Parsed nodes together with the `v-if` chain slot they occupy.
struct VueItem {
    nodes: Vec<Node>,
    branch: Option<VueBranch>,
}

fn push_node(
    nodes: &mut Vec<Node>,
    item: VueItem,
    root: &str,
    at: &str,
) -> Result<(), RawParseError> {
    match item.branch {
        None => nodes.extend(item.nodes),
        Some(VueBranch::If(condition)) => nodes.push(Node::If(IfBlock {
            condition,
            then_children: item.nodes,
            else_children: None,
        })),
        Some(VueBranch::ElseIf(condition)) => {
            let slot = innermost_else_slot(nodes)
                .ok_or_else(|| jsx_err(root, at, "v-else-if without a preceding v-if"))?;
            *slot = Some(vec![Node::If(IfBlock {
                condition,
                then_children: item.nodes,
                else_children: None,
            })]);
        }
        Some(VueBranch::Else) => {
            let slot = innermost_else_slot(nodes)
                .ok_or_else(|| jsx_err(root, at, "v-else without a preceding v-if"))?;
            *slot = Some(item.nodes);
        }
    }
    Ok(())
}

/// Walk to the deepest unfilled `else_children` slot of the trailing `v-if` chain.
fn innermost_else_slot(nodes: &mut [Node]) -> Option<&mut Option<Vec<Node>>> {
    let Node::If(block) = nodes.last_mut()? else {
        return None;
    };
    let mut current = block;
    loop {
        let filled_with_if = matches!(current.else_children.as_deref(), Some([Node::If(_)]));
        if !filled_with_if {
            if current.else_children.is_some() {
                return None;
            }
            return Some(&mut current.else_children);
        }
        let Some([Node::If(next)]) = current.else_children.as_deref_mut() else {
            return None;
        };
        current = next;
    }
}

fn parse_vue_tag<'a>(
    root: &'a str,
    src: &'a str,
    depth: usize,
) -> Result<(VueItem, &'a str), RawParseError> {
    let after_lt = &src[1..];
    let name_end = after_lt
        .find(|c: char| c.is_whitespace() || c == '>' || c == '/')
        .unwrap_or(after_lt.len());
    let tag = &after_lt[..name_end];
    let attrs_src = after_lt[name_end..].trim_start();

    let (attrs, after_gt, self_closing) = jsx_parse_attrs(root, attrs_src)?;
    let void = VOID_ELEMENTS.contains(&tag.to_ascii_lowercase().as_str());

    let (children, rest) = if self_closing || void {
        (Vec::new(), after_gt)
    } else {
        let (children, rest) = parse_vue_nodes(root, after_gt, depth + 1)?;
        let rest = jsx_close(root, rest, tag)?;
        (children, rest)
    };

    let VueElement {
        mut element,
        branch,
        for_spec,
        replacement_children,
    } = vue_build_element(tag, attrs, children).map_err(|mut e| {
        if e.byte_offset.is_none() {
            e.byte_offset = Some(super::super::subslice_byte_offset(root, src));
        }
        e
    })?;

    if let Some(children) = replacement_children {
        element.children = children;
    }

    // `<template>` is a structural wrapper: it contributes its children, not an element.
    let mut nodes = if tag == "template" {
        std::mem::take(&mut element.children)
    } else {
        vec![Node::Element(element)]
    };

    if let Some((pattern, iterator)) = for_spec {
        nodes = vec![Node::For(ForBlock {
            pattern,
            iterator,
            body: nodes,
        })];
    }

    Ok((VueItem { nodes, branch }, rest))
}

/// Consume text up to the next `<`, turning `{{ expr }}` into expression parts.
fn vue_text_node(src: &str) -> (Option<Node>, &str) {
    let end = src.find('<').unwrap_or(src.len());
    let text = &src[..end];
    if text.trim().is_empty() {
        return (None, &src[end..]);
    }
    let parts = parse_vue_text(text.trim());
    if parts.is_empty() {
        (None, &src[end..])
    } else {
        (Some(Node::Text(parts)), &src[end..])
    }
}

pub(crate) fn parse_vue_text(text: &str) -> Vec<TextPart> {
    let mut parts = Vec::new();
    let mut literal = String::new();
    let mut rest = text;

    while let Some(open) = rest.find("{{") {
        let Some(close_rel) = rest[open + 2..].find("}}") else {
            break;
        };
        literal.push_str(&rest[..open]);
        if !literal.is_empty() {
            parts.push(TextPart::Literal(std::mem::take(&mut literal)));
        }
        let expr = rest[open + 2..open + 2 + close_rel].trim();
        if !expr.is_empty() {
            parts.push(TextPart::Expr(expr.to_string()));
        }
        rest = &rest[open + 2 + close_rel + 2..];
    }

    literal.push_str(rest);
    if !literal.is_empty() {
        parts.push(TextPart::Literal(literal));
    }
    parts
}
