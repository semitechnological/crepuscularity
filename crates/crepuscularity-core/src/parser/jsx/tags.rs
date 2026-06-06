//! JSX tag parsing (if, for, match, elements).

use crate::ast::*;

use super::super::RawParseError;
use super::attrs::{jsx_parse_attrs, JsxAttr, JsxAttrValue};
use super::builders::{jsx_build_element, jsx_build_embed, jsx_build_include, jsx_build_let};
use super::template::parse_jsx_nodes;
use super::text::{jsx_close, jsx_err};

pub(crate) fn parse_jsx_tag<'a>(
    norm_root: &'a str,
    src: &'a str,
) -> Result<(Node, &'a str), RawParseError> {
    let src = src.trim_start();
    let after_lt = &src[1..];
    let name_end = after_lt
        .find(|c: char| c.is_whitespace() || c == '>' || c == '/')
        .unwrap_or(after_lt.len());
    let tag = &after_lt[..name_end];
    let rest = after_lt[name_end..].trim_start();

    let (attrs, after_gt, self_closing) = jsx_parse_attrs(norm_root, rest)?;

    match tag {
        "if" => parse_jsx_if(norm_root, attrs, after_gt),
        "else" | "else-if" => Err(jsx_err(
            norm_root,
            src,
            format!("<{tag}> encountered outside <if>"),
        )),
        "for" => parse_jsx_for(norm_root, attrs, after_gt),
        "match" => parse_jsx_match(norm_root, attrs, after_gt),
        "island" | "crepus-island" if self_closing => Ok((jsx_build_embed(attrs), after_gt)),
        "include" if self_closing => Ok((jsx_build_include(attrs, vec![]), after_gt)),
        "include" => {
            let (slot, rest) = parse_jsx_nodes(norm_root, after_gt)?;
            let rest = jsx_close(norm_root, rest, "include")?;
            Ok((jsx_build_include(attrs, slot), rest))
        }
        "let" => Ok((Node::LetDecl(jsx_build_let(attrs, false)), after_gt)),
        "let-default" => Ok((Node::LetDecl(jsx_build_let(attrs, true)), after_gt)),
        _ if self_closing => Ok((
            Node::Element(jsx_build_element(tag, attrs, vec![])),
            after_gt,
        )),
        _ => {
            let (children, rest) = parse_jsx_nodes(norm_root, after_gt)?;
            let rest = jsx_close(norm_root, rest, tag)?;
            Ok((Node::Element(jsx_build_element(tag, attrs, children)), rest))
        }
    }
}

fn parse_jsx_if<'a>(
    norm_root: &'a str,
    attrs: Vec<JsxAttr>,
    children_src: &'a str,
) -> Result<(Node, &'a str), RawParseError> {
    let condition = attrs
        .iter()
        .find(|a| matches!(a.key.as_str(), "condition" | "test" | "cond"))
        .and_then(|a| a.as_expr())
        .unwrap_or_default();

    let (then_children, rest) = parse_jsx_nodes(norm_root, children_src)?;
    let rest = rest.trim_start();

    let (else_children, rest) = if rest.starts_with("<else-if") {
        let after_name = rest.strip_prefix("<else-if").unwrap_or("").trim_start();
        let (ei_attrs, ei_body, _) = jsx_parse_attrs(norm_root, after_name)?;
        let (nested, next) = parse_jsx_if(norm_root, ei_attrs, ei_body)?;
        (Some(vec![nested]), next)
    } else if rest.starts_with("<else") {
        let after_name = rest.strip_prefix("<else").unwrap_or("").trim_start();
        let (_, else_body, self_closing) = jsx_parse_attrs(norm_root, after_name)?;
        if self_closing {
            (Some(vec![]), else_body)
        } else {
            let (else_nodes, after_nodes) = parse_jsx_nodes(norm_root, else_body)?;
            let after_close = jsx_close(norm_root, after_nodes, "else")?;
            (Some(else_nodes), after_close)
        }
    } else {
        (None, rest)
    };

    let rest = jsx_close(norm_root, rest, "if")?;
    Ok((
        Node::If(IfBlock {
            condition,
            then_children,
            else_children,
        }),
        rest,
    ))
}

fn parse_jsx_for<'a>(
    norm_root: &'a str,
    attrs: Vec<JsxAttr>,
    children_src: &'a str,
) -> Result<(Node, &'a str), RawParseError> {
    let pattern = attrs
        .iter()
        .find(|a| matches!(a.key.as_str(), "let" | "var"))
        .and_then(|a| a.as_str())
        .unwrap_or("")
        .to_string();
    let iterator = attrs
        .iter()
        .find(|a| a.key == "in")
        .and_then(|a| a.as_expr())
        .unwrap_or_default();

    let (body, rest) = parse_jsx_nodes(norm_root, children_src)?;
    let rest = jsx_close(norm_root, rest, "for")?;
    Ok((
        Node::For(ForBlock {
            pattern,
            iterator,
            body,
        }),
        rest,
    ))
}

fn parse_jsx_match<'a>(
    norm_root: &'a str,
    attrs: Vec<JsxAttr>,
    children_src: &'a str,
) -> Result<(Node, &'a str), RawParseError> {
    let expr = attrs
        .iter()
        .find(|a| matches!(a.key.as_str(), "on" | "value"))
        .and_then(|a| a.as_expr())
        .unwrap_or_default();

    let mut arms = Vec::new();
    let mut rest = children_src.trim_start();

    while rest.starts_with("<case") {
        let after_name = &rest["<case".len()..].trim_start();
        let (case_attrs, case_body, self_closing) = jsx_parse_attrs(norm_root, after_name)?;
        let pattern = case_attrs
            .iter()
            .find(|a| matches!(a.key.as_str(), "pattern" | "match" | "when"))
            .and_then(|a| match &a.value {
                JsxAttrValue::Str(s) => Some(s.clone()),
                JsxAttrValue::Expr(e) => Some(e.clone()),
                JsxAttrValue::Bool(_) => None,
            })
            .unwrap_or_else(|| "_".to_string());
        let (body, after_body): (Vec<Node>, &str) = if self_closing {
            (vec![], case_body)
        } else {
            let (b, r) = parse_jsx_nodes(norm_root, case_body)?;
            let r = jsx_close(norm_root, r, "case")?;
            (b, r)
        };
        arms.push(MatchArm { pattern, body });
        rest = after_body.trim_start();
    }

    let rest = jsx_close(norm_root, rest, "match")?;
    Ok((Node::Match(MatchBlock { expr, arms }), rest))
}
