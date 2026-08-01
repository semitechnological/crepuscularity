//! Angular element parsing.

use crate::ast::*;

use super::super::jsx::{jsx_close, jsx_parse_attrs};
use super::super::RawParseError;
use super::builders::{angular_build_element, AngularElement};
use super::{angular_err, parse_angular_nodes};

const VOID_TAGS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
    "track", "wbr",
];

pub(crate) fn parse_angular_tag<'a>(
    root: &'a str,
    src: &'a str,
    depth: usize,
) -> Result<(Vec<Node>, &'a str), RawParseError> {
    let after_lt = &src[1..];
    let name_end = after_lt
        .find(|c: char| c.is_whitespace() || c == '>' || c == '/')
        .unwrap_or(after_lt.len());
    let tag = &after_lt[..name_end];

    if tag == "ng-template" {
        return Err(angular_err(
            root,
            src,
            "`<ng-template>` is not supported: it renders only when a structural \
             directive instantiates it, which the shared AST cannot express \
             (use `<ng-container *ngIf=\"…\">` instead)",
        ));
    }
    if tag == "ng-content" {
        return Err(angular_err(
            root,
            src,
            "`<ng-content>` is not supported by the Angular frontend \
             (use crepuscularity `include` slots instead)",
        ));
    }

    let attrs_src = after_lt[name_end..].trim_start();
    let (attrs, after_gt, self_closing) = jsx_parse_attrs(root, attrs_src)?;
    let void = VOID_TAGS.contains(&tag.to_ascii_lowercase().as_str());

    let (children, rest) = if self_closing || void {
        (Vec::new(), after_gt)
    } else {
        let (children, rest) = parse_angular_nodes(root, after_gt, depth + 1, false)?;
        let rest = jsx_close(root, rest, tag)?;
        (children, rest)
    };

    let AngularElement {
        mut element,
        if_condition,
        for_spec,
        replacement_children,
    } = angular_build_element(tag, attrs, children).map_err(|mut e| {
        if e.byte_offset.is_none() {
            e.byte_offset = Some(super::super::subslice_byte_offset(root, src));
        }
        e
    })?;

    if let Some(children) = replacement_children {
        element.children = children;
    }

    // `<ng-container>` is a structural wrapper: it contributes its children,
    // not an element of its own.
    let mut nodes = if tag == "ng-container" {
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
    if let Some(condition) = if_condition {
        nodes = vec![Node::If(IfBlock {
            condition,
            then_children: nodes,
            else_children: None,
        })];
    }

    Ok((nodes, rest))
}
