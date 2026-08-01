//! Astro `{…}` expression lowering.
//!
//! Astro shares JSX's expression syntax, so the two markup-producing forms —
//! `{cond && <markup/>}` and `{list.map(item => <markup/>)}` — are recognised
//! structurally and lowered to the shared `IfBlock` / `ForBlock`. Anything else
//! is an ordinary expression and becomes a text interpolation.

use crate::ast::*;

use super::super::{match_delimiter as match_delim, RawParseError};
use super::{astro_err, parse_astro_nodes};

pub(crate) fn match_brace(src: &str) -> Option<usize> {
    match_delim(src, '{', '}')
}

/// Read a `{…}` group, returning the trimmed inner slice and the remaining
/// source. Both are subslices of `root`, so error offsets stay exact.
pub(crate) fn read_brace_slice<'a>(
    root: &'a str,
    src: &'a str,
) -> Result<(&'a str, &'a str), RawParseError> {
    if !src.starts_with('{') {
        return Err(astro_err(root, src, "expected `{`"));
    }
    let end =
        match_brace(src).ok_or_else(|| astro_err(root, src, "unclosed `{` in Astro expression"))?;
    Ok((src[1..end].trim(), &src[end + 1..]))
}

/// Run `f` over every top-level byte position of `src`, skipping string
/// literals and bracketed groups.
fn scan_top_level(src: &str, mut f: impl FnMut(usize, char) -> bool) -> Option<usize> {
    let mut depth = 0i32;
    let mut quote: Option<char> = None;
    for (i, c) in src.char_indices() {
        if let Some(q) = quote {
            if c == q {
                quote = None;
            }
            continue;
        }
        match c {
            '"' | '\'' | '`' => quote = Some(c),
            '{' | '[' | '(' => depth += 1,
            '}' | ']' | ')' => depth -= 1,
            _ if depth == 0 && f(i, c) => return Some(i),
            _ => {}
        }
    }
    None
}

/// Split `cond ? a : b` at its top-level `?` and matching `:`.
fn split_ternary(src: &str) -> Option<(&str, &str, &str)> {
    let bytes = src.as_bytes();
    let q = scan_top_level(src, |i, c| {
        c == '?'
            && bytes.get(i + 1) != Some(&b'?')
            && bytes.get(i + 1) != Some(&b'.')
            && (i == 0 || bytes[i - 1] != b'?')
    })?;
    let tail = &src[q + 1..];
    let tail_bytes = tail.as_bytes();
    let mut nested = 0i32;
    let colon = scan_top_level(tail, |i, c| {
        if c == '?' && tail_bytes.get(i + 1) != Some(&b'?') && tail_bytes.get(i + 1) != Some(&b'.')
        {
            nested += 1;
            return false;
        }
        if c == ':' {
            if nested == 0 {
                return true;
            }
            nested -= 1;
        }
        false
    })?;
    Some((
        src[..q].trim(),
        tail[..colon].trim(),
        tail[colon + 1..].trim(),
    ))
}

/// Split `cond && rhs` at its top-level `&&`.
fn split_and(src: &str) -> Option<(&str, &str)> {
    let pos = scan_top_level(src, |i, c| c == '&' && src[i..].starts_with("&&"))?;
    Some((src[..pos].trim(), src[pos + 2..].trim()))
}

/// Split `iterable.map(arrow)` when the call closes the whole expression.
fn split_map(src: &str) -> Option<(&str, &str)> {
    let pos = scan_top_level(src, |i, c| c == '.' && src[i..].starts_with(".map("))?;
    let open = pos + 4;
    let close = open + match_delim(&src[open..], '(', ')')?;
    if !src[close + 1..].trim().is_empty() {
        return None;
    }
    Some((src[..pos].trim(), src[open + 1..close].trim()))
}

/// Strip one layer of parentheses when they wrap the whole slice.
fn strip_wrapping_parens(src: &str) -> &str {
    if !src.starts_with('(') {
        return src;
    }
    match match_delim(src, '(', ')') {
        Some(end) if src[end + 1..].trim().is_empty() => src[1..end].trim(),
        _ => src,
    }
}

pub(crate) fn lower_astro_expr(
    root: &str,
    expr: &str,
    depth: usize,
) -> Result<Vec<Node>, RawParseError> {
    if depth > super::MAX_ASTRO_DEPTH {
        return Err(astro_err(
            root,
            expr,
            format!(
                "maximum Astro nesting depth ({}) exceeded",
                super::MAX_ASTRO_DEPTH
            ),
        ));
    }
    if expr.is_empty() {
        return Ok(Vec::new());
    }

    if expr.starts_with('<') {
        let (nodes, rest) = parse_astro_nodes(root, expr, depth + 1)?;
        let rest = rest.trim();
        if !rest.is_empty() {
            return Err(astro_err(
                root,
                rest,
                format!(
                    "unexpected trailing markup in Astro expression: {}",
                    &rest[..rest.len().min(40)]
                ),
            ));
        }
        return Ok(nodes);
    }

    if let Some((condition, then_src, else_src)) = split_ternary(expr) {
        return Ok(vec![Node::If(IfBlock {
            condition: condition.to_string(),
            then_children: lower_astro_expr(root, then_src, depth + 1)?,
            else_children: Some(lower_astro_expr(root, else_src, depth + 1)?),
        })]);
    }

    if let Some((condition, then_src)) = split_and(expr) {
        return Ok(vec![Node::If(IfBlock {
            condition: condition.to_string(),
            then_children: lower_astro_expr(root, then_src, depth + 1)?,
            else_children: None,
        })]);
    }

    if let Some((iterator, arrow)) = split_map(expr) {
        return lower_map(root, expr, iterator, arrow, depth);
    }

    Ok(vec![Node::Text(vec![TextPart::Expr(expr.to_string())])])
}

fn lower_map(
    root: &str,
    at: &str,
    iterator: &str,
    arrow: &str,
    depth: usize,
) -> Result<Vec<Node>, RawParseError> {
    let Some(pos) = scan_top_level(arrow, |i, c| c == '=' && arrow[i..].starts_with("=>")) else {
        return Err(astro_err(
            root,
            at,
            "`.map(…)` must be called with an arrow function \
             (the shared ForBlock binds one item variable)",
        ));
    };
    let param = strip_wrapping_parens(arrow[..pos].trim());
    let body = arrow[pos + 2..].trim();

    if param.is_empty() {
        return Err(astro_err(
            root,
            at,
            "`.map(…)` arrow function has no item binding",
        ));
    }
    if param.starts_with('{') || param.starts_with('[') {
        return Err(astro_err(
            root,
            at,
            "destructuring `.map(({a, b}) => …)` is not supported: the shared \
             ForBlock binds a single item variable",
        ));
    }
    if param.contains(',') {
        return Err(astro_err(
            root,
            at,
            "`.map((item, i) => …)` index bindings are not supported: the shared \
             ForBlock has no index variable, so `i` would be undefined at render time",
        ));
    }
    if body.starts_with('{') {
        return Err(astro_err(
            root,
            at,
            "block-bodied `.map(item => { … })` arrow functions are not supported: \
             this frontend compiles markup only and never executes JavaScript",
        ));
    }

    let body = strip_wrapping_parens(body);
    Ok(vec![Node::For(ForBlock {
        pattern: param.to_string(),
        iterator: iterator.to_string(),
        body: lower_astro_expr(root, body, depth + 1)?,
    })])
}
