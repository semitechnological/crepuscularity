//! JSX template entry point and node sequence parsing.

use crate::ast::*;

use super::super::indent::try_parse_let_decl;
use super::super::RawParseError;
use super::attrs::jsx_brace_expr;
use super::tags::parse_jsx_tag;
use super::text::{jsx_text_node, map_jsx_offset, normalize_jsx_mapped};

pub(crate) fn parse_jsx_template(src: &str) -> Result<Vec<Node>, RawParseError> {
    let (norm, map) = normalize_jsx_mapped(src);
    let root = norm.as_str();
    match parse_jsx_nodes(root, root) {
        Ok((nodes, _)) => Ok(nodes),
        Err(mut err) => {
            if let Some(off) = err.byte_offset.take() {
                err.byte_offset = Some(map_jsx_offset(&map, off));
            }
            Err(err)
        }
    }
}

pub(crate) fn parse_jsx_nodes<'a>(
    norm_root: &'a str,
    src: &'a str,
) -> Result<(Vec<Node>, &'a str), RawParseError> {
    let mut nodes = Vec::new();
    let mut rest = src;

    loop {
        let t = rest.trim_start();

        if t.is_empty() {
            rest = t;
            break;
        }
        if t.starts_with("</") || t.starts_with("<else") {
            rest = t;
            break;
        }
        if t.starts_with("$:") {
            let end = t.find('\n').unwrap_or(t.len());
            let line = t[..end].trim();
            rest = &t[end..];
            if let Some(decl) = try_parse_let_decl(line) {
                nodes.push(Node::LetDecl(decl));
            }
            continue;
        }
        if t.starts_with('<') {
            rest = t;
            let (node, next) = parse_jsx_tag(norm_root, rest)?;
            nodes.push(node);
            rest = next;
            continue;
        }
        if t.starts_with("{=") {
            rest = t;
            let (mut expr, next) = jsx_brace_expr(norm_root, rest)?;
            if expr.starts_with('=') {
                expr = expr[1..].trim().to_string();
            }
            nodes.push(Node::RawHtml(expr));
            rest = next;
            continue;
        }
        if t.starts_with('{') {
            rest = t;
            let (expr, next) = jsx_brace_expr(norm_root, rest)?;
            nodes.push(Node::RawText(expr));
            rest = next;
            continue;
        }
        let prev_len = rest.len();
        let (node_opt, next) = jsx_text_node(rest);
        if let Some(node) = node_opt {
            nodes.push(node);
        }
        rest = next;
        if rest.len() == prev_len {
            let skip = rest
                .char_indices()
                .nth(1)
                .map(|(i, _)| i)
                .unwrap_or(rest.len());
            rest = &rest[skip..];
        }
    }

    Ok((nodes, rest))
}
