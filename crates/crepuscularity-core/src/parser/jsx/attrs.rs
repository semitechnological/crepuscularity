//! JSX attribute parsing.

use super::super::RawParseError;
use super::text::jsx_err;

pub(crate) struct JsxAttr {
    pub key: String,
    pub value: JsxAttrValue,
}

pub(crate) enum JsxAttrValue {
    Bool(bool),
    Str(String),
    Expr(String),
}

impl JsxAttr {
    pub(crate) fn as_str(&self) -> Option<&str> {
        if let JsxAttrValue::Str(s) = &self.value {
            Some(s)
        } else {
            None
        }
    }

    /// Returns an evaluable expression string for the attribute value.
    pub(crate) fn as_expr(&self) -> Option<String> {
        match &self.value {
            JsxAttrValue::Expr(e) => Some(e.clone()),
            JsxAttrValue::Str(s) => Some(format!("\"{}\"", s.replace('"', "\\\""))),
            JsxAttrValue::Bool(b) => Some(b.to_string()),
        }
    }
}

pub(crate) fn jsx_parse_attrs<'a>(
    norm_root: &'a str,
    src: &'a str,
) -> Result<(Vec<JsxAttr>, &'a str, bool), RawParseError> {
    let mut attrs = Vec::new();
    let mut rest = src.trim_start();
    let mut self_closing = false;

    loop {
        rest = rest.trim_start();
        if rest.is_empty() {
            return Err(jsx_err(norm_root, rest, "unclosed JSX tag"));
        }
        if rest.starts_with("/>") {
            self_closing = true;
            rest = &rest[2..];
            break;
        }
        if rest.starts_with('>') {
            rest = &rest[1..];
            break;
        }

        let key_end = rest
            .find(|c: char| c.is_whitespace() || c == '=' || c == '>' || c == '/')
            .unwrap_or(rest.len());
        if key_end == 0 {
            rest = &rest[1..];
            continue;
        }
        let key = rest[..key_end].to_string();
        rest = rest[key_end..].trim_start();

        if rest.starts_with('=') {
            rest = rest[1..].trim_start();
            let (value, next) = jsx_attr_value(norm_root, rest)?;
            attrs.push(JsxAttr { key, value });
            rest = next;
        } else {
            attrs.push(JsxAttr {
                key,
                value: JsxAttrValue::Bool(true),
            });
        }
    }

    Ok((attrs, rest, self_closing))
}

pub(crate) fn jsx_attr_value<'a>(
    norm_root: &'a str,
    src: &'a str,
) -> Result<(JsxAttrValue, &'a str), RawParseError> {
    if src.starts_with('"') {
        let mut i = 1;
        let bytes = src.as_bytes();
        while i < bytes.len() {
            match bytes[i] {
                b'\\' => i += 2,
                b'"' => {
                    let content = src[1..i].replace("\\\"", "\"");
                    return Ok((JsxAttrValue::Str(content), &src[i + 1..]));
                }
                _ => i += 1,
            }
        }
        let inner = src.strip_prefix('"').unwrap_or("");
        Ok((JsxAttrValue::Str(inner.replace("\\\"", "\"")), ""))
    } else if src.starts_with('\'') {
        let inner = src.strip_prefix('\'').unwrap_or(src);
        let end = inner.find('\'').unwrap_or(inner.len());
        Ok((
            JsxAttrValue::Str(inner[..end].to_string()),
            &inner[end + 1..],
        ))
    } else if src.starts_with('{') {
        let (expr, rest) = jsx_brace_expr(norm_root, src)?;
        Ok((JsxAttrValue::Expr(expr), rest))
    } else {
        let end = src
            .find(|c: char| c.is_whitespace() || c == '>' || c == '/')
            .unwrap_or(src.len());
        let val = &src[..end];
        let value = match val {
            "true" => JsxAttrValue::Bool(true),
            "false" => JsxAttrValue::Bool(false),
            other => JsxAttrValue::Str(other.to_string()),
        };
        Ok((value, &src[end..]))
    }
}

pub(crate) fn jsx_brace_expr<'a>(
    norm_root: &'a str,
    src: &'a str,
) -> Result<(String, &'a str), RawParseError> {
    let src = src.trim_start();
    if !src.starts_with('{') {
        return Err(jsx_err(
            norm_root,
            src,
            format!("expected '{{', got: {}", &src[..src.len().min(10)]),
        ));
    }
    let mut depth = 0usize;
    for (i, c) in src.char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    let expr = src[1..i].trim().to_string();
                    return Ok((expr, &src[i + 1..]));
                }
            }
            _ => {}
        }
    }
    Err(jsx_err(norm_root, src, "unclosed '{' in JSX expression"))
}
