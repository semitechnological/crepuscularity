//! Include and embed directive parsing.

use crate::ast::*;

pub(crate) fn try_parse_include(line: &str) -> Option<IncludeNode> {
    let rest = line.strip_prefix("include ")?;
    // First token is the path (no spaces in path), rest are props
    let (path, props_str) = match rest.find(' ') {
        Some(pos) => (rest[..pos].trim().to_string(), rest[pos + 1..].trim()),
        None => (rest.trim().to_string(), ""),
    };
    if path.is_empty() {
        return None;
    }
    let props = parse_props(props_str);
    Some(IncludeNode {
        path,
        props,
        slot: vec![],
    })
}

pub(crate) fn try_parse_embed(line: &str) -> Option<EmbedNode> {
    let rest = line.strip_prefix("embed ")?;
    let (src, props_str) = match rest.find(' ') {
        Some(pos) => (rest[..pos].trim().to_string(), rest[pos + 1..].trim()),
        None => (rest.trim().to_string(), ""),
    };
    if src.is_empty() {
        return None;
    }
    let mut props = parse_props(props_str);
    let adapter = take_literal_prop(&mut props, "adapter");
    Some(EmbedNode {
        src,
        adapter,
        props,
    })
}

fn take_literal_prop(props: &mut Vec<(String, String)>, key: &str) -> Option<String> {
    let pos = props.iter().position(|(k, _)| k == key)?;
    let (_, value) = props.remove(pos);
    Some(unquote_expr_string(&value).unwrap_or(value))
}

fn unquote_expr_string(value: &str) -> Option<String> {
    if value.len() >= 2 && value.starts_with('"') && value.ends_with('"') {
        Some(value[1..value.len() - 1].replace("\\\"", "\""))
    } else {
        None
    }
}

fn parse_props(s: &str) -> Vec<(String, String)> {
    let mut props = Vec::new();
    let mut remaining = s.trim();

    while !remaining.is_empty() {
        // Find key= (key is an identifier, no spaces)
        let eq_pos = match remaining.find('=') {
            Some(p) => p,
            None => break,
        };
        let key = remaining[..eq_pos].trim().to_string();
        if key.is_empty() || key.contains(' ') {
            break;
        }
        remaining = remaining[eq_pos + 1..].trim_start();

        // Extract value
        let (expr_str, rest) = extract_prop_value(remaining);
        props.push((key, expr_str));
        remaining = rest.trim_start();
    }

    props
}

/// Extract a prop value token from the start of `s`.
/// Returns `(expr_string, remaining)`.
/// - `"quoted"` → returns the string content wrapped in quotes for the evaluator
/// - `{expr}` → returns the inner expr string
/// - `bare_token` → returns the token as-is (treated as a variable name / literal)
fn extract_prop_value(s: &str) -> (String, &str) {
    if s.is_empty() {
        return (String::new(), s);
    }

    if s.starts_with('"') || s.starts_with('\'') {
        let quote = s.as_bytes()[0];
        let mut i = 1;
        let mut escaped = false;
        while i < s.len() {
            let byte = s.as_bytes()[i];
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == quote {
                let content = &s[1..i];
                let escaped_content = content.replace('\\', "\\\\").replace('"', "\\\"");
                let expr = format!("\"{}\"", escaped_content);
                let rest = if i < s.len() { &s[i + 1..] } else { "" };
                return (expr, rest);
            }
            i += 1;
        }

        let content = &s[1..];
        let escaped_content = content.replace('\\', "\\\\").replace('"', "\\\"");
        let expr = format!("\"{}\"", escaped_content);
        return (expr, "");
    }

    if s.starts_with('{') {
        let mut depth = 0usize;
        for (i, c) in s.char_indices() {
            match c {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        let expr = s[1..i].trim().to_string();
                        return (expr, &s[i + 1..]);
                    }
                }
                _ => {}
            }
        }
        return (s.to_string(), "");
    }

    // Bare token: ends at next space
    let end = s.find(' ').unwrap_or(s.len());
    (s[..end].to_string(), &s[end..])
}
