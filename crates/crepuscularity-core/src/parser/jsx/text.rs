//! JSX text nodes and offset mapping helpers.

use crate::ast::*;

use super::super::indent::parse_text_template;
use super::super::{subslice_byte_offset, RawParseError};

pub(crate) fn normalize_jsx_mapped(s: &str) -> (String, Vec<usize>) {
    let mut norm = String::with_capacity(s.len());
    let mut map: Vec<usize> = Vec::with_capacity(s.len());
    for (orig_b, ch) in s.char_indices() {
        match ch {
            '\u{FF5B}' => {
                norm.push('{');
                map.push(orig_b);
            }
            '\u{FF5D}' => {
                norm.push('}');
                map.push(orig_b);
            }
            c => {
                let mut buf = [0u8; 4];
                let enc = c.encode_utf8(&mut buf);
                for _ in 0..enc.len() {
                    map.push(orig_b);
                }
                norm.push_str(enc);
            }
        }
    }
    debug_assert_eq!(norm.len(), map.len());
    (norm, map)
}

pub(crate) fn map_jsx_offset(map: &[usize], off: usize) -> usize {
    map.get(off).copied().unwrap_or(off)
}

#[inline]
pub(crate) fn jsx_err(norm_root: &str, at: &str, message: impl Into<String>) -> RawParseError {
    RawParseError {
        message: message.into(),
        byte_offset: Some(subslice_byte_offset(norm_root, at)),
    }
}

/// Consume text content up to the next `<` tag, parsing `{expr}` interpolations.
pub(crate) fn jsx_text_node(src: &str) -> (Option<Node>, &str) {
    let end = src.find('<').unwrap_or(src.len());
    if end == 0 {
        return (None, src);
    }
    let text = &src[..end];
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return (None, &src[end..]);
    }
    let parts = parse_text_template(trimmed);
    (Some(Node::Text(parts)), &src[end..])
}

pub(crate) fn jsx_close<'a>(
    norm_root: &str,
    src: &'a str,
    tag: &str,
) -> Result<&'a str, RawParseError> {
    let src = src.trim_start();
    let prefix = format!("</{}", tag);
    if let Some(rest) = src.strip_prefix(&prefix) {
        let rest = rest.trim_start();
        if let Some(rest) = rest.strip_prefix('>') {
            return Ok(rest);
        }
    }
    Err(jsx_err(
        norm_root,
        src,
        format!("expected </{}>, got: {}", tag, &src[..src.len().min(40)]),
    ))
}
