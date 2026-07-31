//! Svelte template frontend.
//!
//! Compiles `.svelte` markup directly into the shared `Node`/`Element` AST, so
//! Svelte templates flow through the same lowering as every other frontend and
//! reach every renderer (React, Solid, SwiftUI, Compose, TUI, LVGL) unchanged.
//! This is a native frontend — it does not depend on `svelte/compiler`.
//!
//! Scope: this frontend compiles the TEMPLATE only. `<script>` contents are
//! extracted verbatim and never executed: runes (`$state`, `$derived`,
//! `$effect`, `$props`), reactive statements, stores, imports and component
//! resolution are all outside this frontend. Expressions in markup are
//! evaluated by the shared crepuscularity evaluator against the template
//! context, exactly as `{name}` is in the indentation frontend.

mod blocks;
mod tags;
#[cfg(test)]
mod tests;

use super::RawParseError;
use crate::ast::Node;

/// Maximum element nesting depth, mirroring the JSX frontend's guard.
pub(crate) const MAX_SVELTE_DEPTH: usize = 256;

/// A parsed `.svelte` single-file component.
#[derive(Debug, Clone, Default)]
pub struct SvelteComponent {
    /// The compiled markup, as shared-AST nodes.
    pub nodes: Vec<Node>,
    /// Raw `<style>` body, preserved verbatim so a caller can emit it.
    pub style: Option<String>,
    /// Raw `<script>` body, preserved verbatim. It is NOT executed.
    pub script: Option<String>,
    /// Raw `<script context="module">` / `<script module>` body. NOT executed.
    pub module_script: Option<String>,
}

/// Parse a `.svelte` single-file component, keeping `<style>` and `<script>`.
pub fn parse_svelte_component(src: &str) -> Result<SvelteComponent, crate::error::CrepusError> {
    parse_svelte_component_raw(src).map_err(|e| crate::error::CrepusError::parse(e.message))
}

/// Parse `.svelte` markup into shared-AST nodes, discarding script/style.
pub(crate) fn parse_svelte_template(src: &str) -> Result<Vec<Node>, RawParseError> {
    parse_svelte_component_raw(src).map(|c| c.nodes)
}

pub(crate) fn parse_svelte_component_raw(src: &str) -> Result<SvelteComponent, RawParseError> {
    let extracted = extract_top_level_blocks(src);
    let body = extracted.blanked;
    let (nodes, _) = parse_svelte_nodes(&body, &body, 0)?;
    Ok(SvelteComponent {
        nodes,
        style: extracted.style,
        script: extracted.script,
        module_script: extracted.module_script,
    })
}

struct Extracted {
    blanked: String,
    style: Option<String>,
    script: Option<String>,
    module_script: Option<String>,
}

/// Replace `<script>` / `<style>` regions with spaces (newlines preserved) so
/// byte offsets into the blanked body still map onto the original source.
fn extract_top_level_blocks(src: &str) -> Extracted {
    let mut blanked = src.to_string();
    let mut style = None;
    let mut script = None;
    let mut module_script = None;

    let lower = src.to_ascii_lowercase();
    let mut i = 0usize;
    while i < lower.len() {
        let Some(rel) = lower[i..].find('<') else {
            break;
        };
        let at = i + rel;
        let name = if lower[at..].starts_with("<script") {
            "script"
        } else if lower[at..].starts_with("<style") {
            "style"
        } else {
            i = at + 1;
            continue;
        };

        let Some(open_end_rel) = lower[at..].find('>') else {
            break;
        };
        let open_end = at + open_end_rel + 1;
        let open_tag = &src[at..open_end];
        let close = format!("</{name}");
        let (content_end, block_end) = match lower[open_end..].find(&close) {
            Some(rel) => {
                let c = open_end + rel;
                let e = lower[c..].find('>').map(|g| c + g + 1).unwrap_or(src.len());
                (c, e)
            }
            None => (src.len(), src.len()),
        };

        let content = src[open_end..content_end].to_string();
        if name == "style" {
            style.get_or_insert(content);
        } else if open_tag.contains("module") {
            module_script.get_or_insert(content);
        } else {
            script.get_or_insert(content);
        }

        blank_range(&mut blanked, at, block_end);
        i = block_end;
    }

    Extracted {
        blanked,
        style,
        script,
        module_script,
    }
}

fn blank_range(buf: &mut String, start: usize, end: usize) {
    let replacement: String = buf[start..end]
        .chars()
        .map(|c| if c == '\n' { '\n' } else { ' ' })
        .collect();
    // `replacement` keeps one char per source char; ASCII-only output can be
    // shorter in bytes than multi-byte input, so pad to preserve offsets.
    let mut replacement = replacement;
    while replacement.len() < end - start {
        replacement.push(' ');
    }
    buf.replace_range(start..end, &replacement);
}

pub(crate) fn svelte_err(root: &str, at: &str, message: impl Into<String>) -> RawParseError {
    RawParseError {
        message: message.into(),
        byte_offset: Some(super::subslice_byte_offset(root, at)),
    }
}

pub(crate) fn parse_svelte_nodes<'a>(
    root: &'a str,
    src: &'a str,
    depth: usize,
) -> Result<(Vec<Node>, &'a str), RawParseError> {
    if depth > MAX_SVELTE_DEPTH {
        return Err(svelte_err(
            root,
            src,
            format!("maximum Svelte nesting depth ({MAX_SVELTE_DEPTH}) exceeded"),
        ));
    }

    let mut nodes = Vec::new();
    let mut rest = src;

    loop {
        let t = rest.trim_start();
        rest = t;

        if t.is_empty() || t.starts_with("</") || t.starts_with("{:") || t.starts_with("{/") {
            break;
        }

        if let Some(after) = t.strip_prefix("<!--") {
            rest = match after.find("-->") {
                Some(end) => &after[end + 3..],
                None => "",
            };
            continue;
        }

        if t.starts_with("{#") {
            let (node, next) = blocks::parse_svelte_block(root, t, depth)?;
            nodes.push(node);
            rest = next;
            continue;
        }

        if t.starts_with("{@") {
            let (node, next) = parse_at_tag(root, t)?;
            nodes.push(node);
            rest = next;
            continue;
        }

        if t.starts_with('<') {
            let (node, next) = tags::parse_svelte_tag(root, t, depth)?;
            nodes.push(node);
            rest = next;
            continue;
        }

        let (node_opt, next) = svelte_text_node(t);
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

fn parse_at_tag<'a>(root: &'a str, src: &'a str) -> Result<(Node, &'a str), RawParseError> {
    let (inner, rest) = blocks::read_brace_tag(root, src)?;
    if let Some(expr) = inner.strip_prefix("@html ") {
        return Ok((Node::RawHtml(expr.trim().to_string()), rest));
    }
    let name = inner
        .split_whitespace()
        .next()
        .unwrap_or("@")
        .trim_start_matches('@')
        .to_string();
    Err(svelte_err(
        root,
        src,
        format!("`{{@{name}}}` is not supported by the Svelte frontend (only `{{@html expr}}` is)"),
    ))
}

/// Consume a text run, stopping at the next tag or block/`{@…}` tag.
/// `{expr}` interpolations are consumed into the run and become `TextPart::Expr`.
fn svelte_text_node(src: &str) -> (Option<Node>, &str) {
    let bytes = src.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        match bytes[i] {
            b'<' => break,
            b'{' => {
                let next = bytes.get(i + 1).copied();
                if matches!(next, Some(b'#') | Some(b':') | Some(b'/') | Some(b'@')) {
                    break;
                }
                let mut depth = 0usize;
                let mut j = i;
                while j < bytes.len() {
                    match bytes[j] {
                        b'{' => depth += 1,
                        b'}' => {
                            depth -= 1;
                            if depth == 0 {
                                break;
                            }
                        }
                        _ => {}
                    }
                    j += 1;
                }
                if j >= bytes.len() {
                    i = bytes.len();
                } else {
                    i = j + 1;
                }
            }
            _ => i += 1,
        }
    }

    let text = &src[..i];
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return (None, &src[i..]);
    }
    let parts = super::indent::parse_text_template(trimmed);
    (Some(Node::Text(parts)), &src[i..])
}
