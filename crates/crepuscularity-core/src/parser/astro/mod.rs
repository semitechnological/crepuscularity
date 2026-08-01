//! Astro template frontend.
//!
//! Compiles `.astro` markup into the shared `Node`/`Element` AST, so Astro
//! templates flow through the same lowering as every other frontend and reach
//! every renderer unchanged. This is a native frontend — it does not depend on
//! `@astrojs/compiler`.
//!
//! Scope: this frontend compiles the TEMPLATE only. The `---` frontmatter fence
//! is JavaScript/TypeScript that Astro runs on the server; this frontend never
//! executes it, so imports, `Astro.props`, top-level `await` and component
//! resolution are all outside its scope. The fence is blanked out (newlines
//! preserved) before parsing so byte offsets in error messages still point at
//! the original source. Expressions in markup are evaluated by the shared
//! crepuscularity evaluator against the template context.
//!
//! The body is JSX-like, so tag and attribute scanning reuses the JSX
//! frontend's machinery rather than a second expression parser.

mod exprs;
mod tags;
#[cfg(test)]
mod tests;

use super::RawParseError;
use crate::ast::Node;

/// Maximum element nesting depth, mirroring the JSX frontend's guard.
pub(crate) const MAX_ASTRO_DEPTH: usize = 256;

pub(crate) fn parse_astro_template(src: &str) -> Result<Vec<Node>, RawParseError> {
    let body = blank_frontmatter(src);
    let root = body.as_str();
    let (nodes, rest) = parse_astro_nodes(root, root, 0)?;
    let rest = rest.trim_start();
    if !rest.is_empty() {
        return Err(astro_err(
            root,
            rest,
            format!(
                "unexpected trailing markup in Astro template: {}",
                &rest[..rest.len().min(40)]
            ),
        ));
    }
    Ok(nodes)
}

/// Replace the leading `---` frontmatter fence with spaces, keeping newlines so
/// byte offsets into the blanked body still map onto the original source.
fn blank_frontmatter(src: &str) -> String {
    let mut out = src.to_string();
    let lead = src.len() - src.trim_start().len();
    let after_lead = &src[lead..];
    let Some(first_line_end) = after_lead.find('\n') else {
        return out;
    };
    if after_lead[..first_line_end].trim_end() != "---" {
        return out;
    }

    let mut cursor = lead + first_line_end + 1;
    while cursor < src.len() {
        let line_end = src[cursor..]
            .find('\n')
            .map(|n| cursor + n)
            .unwrap_or(src.len());
        if src[cursor..line_end].trim() == "---" {
            blank_range(&mut out, lead, line_end);
            return out;
        }
        cursor = line_end + 1;
    }
    out
}

fn blank_range(buf: &mut String, start: usize, end: usize) {
    let mut replacement: String = buf[start..end]
        .chars()
        .map(|c| if c == '\n' { '\n' } else { ' ' })
        .collect();
    // One char per source char; ASCII output can be shorter in bytes than
    // multi-byte input, so pad to preserve offsets.
    while replacement.len() < end - start {
        replacement.push(' ');
    }
    buf.replace_range(start..end, &replacement);
}

pub(crate) fn astro_err(root: &str, at: &str, message: impl Into<String>) -> RawParseError {
    RawParseError {
        message: message.into(),
        byte_offset: Some(super::subslice_byte_offset(root, at)),
    }
}

pub(crate) fn parse_astro_nodes<'a>(
    root: &'a str,
    src: &'a str,
    depth: usize,
) -> Result<(Vec<Node>, &'a str), RawParseError> {
    if depth > MAX_ASTRO_DEPTH {
        return Err(astro_err(
            root,
            src,
            format!("maximum Astro nesting depth ({MAX_ASTRO_DEPTH}) exceeded"),
        ));
    }

    let mut nodes: Vec<Node> = Vec::new();
    let mut rest = src;

    loop {
        let t = rest.trim_start();
        rest = t;

        if t.is_empty() || t.starts_with("</") {
            break;
        }

        if let Some(after) = t.strip_prefix("<!--") {
            rest = match after.find("-->") {
                Some(end) => &after[end + 3..],
                None => "",
            };
            continue;
        }

        if t.starts_with('{') {
            let (inner, next) = exprs::read_brace_slice(root, t)?;
            rest = next;
            if inner.starts_with("/*") || inner.starts_with("//") {
                continue;
            }
            nodes.extend(exprs::lower_astro_expr(root, inner, depth)?);
            continue;
        }

        if t.starts_with('<') {
            let (produced, next) = tags::parse_astro_tag(root, t, depth)?;
            nodes.extend(produced);
            rest = next;
            continue;
        }

        let (node_opt, next) = astro_text_node(t);
        if let Some(node) = node_opt {
            nodes.push(node);
        }
        if next.len() == t.len() {
            let skip = t.char_indices().nth(1).map(|(i, _)| i).unwrap_or(t.len());
            rest = &t[skip..];
        } else {
            rest = next;
        }
    }

    Ok((nodes, rest))
}

/// Consume a text run, stopping at the next tag or at a `{…}` group that
/// contains markup. Interpolations without markup stay inside the run and are
/// lowered to `TextPart::Expr` by the shared text-template parser.
fn astro_text_node(src: &str) -> (Option<Node>, &str) {
    let bytes = src.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        match bytes[i] {
            b'<' => break,
            b'{' => {
                let Some(end) = exprs::match_brace(&src[i..]) else {
                    i = bytes.len();
                    break;
                };
                let group = &src[i..i + end + 1];
                if group.contains('<') {
                    break;
                }
                i += end + 1;
            }
            _ => i += 1,
        }
    }

    let trimmed = src[..i].trim();
    if trimmed.is_empty() {
        return (None, &src[i..]);
    }
    let parts = super::indent::parse_text_template(trimmed);
    (Some(Node::Text(parts)), &src[i..])
}
