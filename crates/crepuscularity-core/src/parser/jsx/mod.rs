//! JSX / HTML tag syntax parser.
//!
//! Activated automatically when `parse_template()` detects JSX mode (first content
//! line starts with `<`). Produces the same `Node`/`Element` AST as the indentation
//! parser, so every backend — GPUI, web, webext — works unchanged.

mod attrs;
mod builders;
mod tags;
mod template;
mod text;

use super::RawParseError;
use crate::ast::Node;
use template::parse_jsx_template as parse_jsx_template_inner;

/// Maximum nesting depth for JSX tag recursion.
/// Prevents stack overflow on pathologically deep input.
const MAX_JSX_DEPTH: usize = 256;

pub(crate) fn parse_jsx_template(src: &str) -> Result<Vec<Node>, RawParseError> {
    // Pre-parse heuristic: count maximum open-tag nesting depth in the raw source.
    // This catches deeply-nested input before the recursive descent parser runs,
    // preventing stack overflow. The recursive calls in `parse_jsx_nodes` and
    // `parse_jsx_tag` (in `template.rs` / `tags.rs`) would otherwise be unbounded.
    let nesting = count_max_tag_nesting(src);
    if nesting > MAX_JSX_DEPTH {
        return Err(RawParseError {
            message: format!("maximum JSX nesting depth ({MAX_JSX_DEPTH}) exceeded"),
            byte_offset: None,
        });
    }
    parse_jsx_template_inner(src)
}

/// Count the maximum nesting depth of opening tags in the source string.
/// This is a fast heuristic that scans for `<` (not `</`) and tracks depth.
fn count_max_tag_nesting(src: &str) -> usize {
    let mut depth = 0usize;
    let mut max_depth = 0usize;
    let bytes = src.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'<' {
            // Check for closing tag `</`
            if i + 1 < bytes.len() && bytes[i + 1] == b'/' {
                if depth > 0 {
                    depth -= 1;
                }
                i += 2;
                continue;
            }
            // Check for comment `<!--` — skip to `-->`
            if i + 3 < bytes.len() && &bytes[i..i + 4] == b"<!--" {
                // Find end of comment
                if let Some(end) = src[i + 4..].find("-->") {
                    i += 4 + end + 3;
                } else {
                    i = bytes.len();
                }
                continue;
            }
            // Opening tag (including self-closing, which we still count
            // as a nesting level for safety — the heuristic is conservative).
            depth += 1;
            if depth > max_depth {
                max_depth = depth;
            }
        } else if bytes[i] == b'>' && i > 0 && bytes[i - 1] == b'/' {
            // Self-closing tag `/>` — pop the depth we just pushed.
            if depth > 0 {
                depth -= 1;
            }
        }
        i += 1;
    }
    max_depth
}
