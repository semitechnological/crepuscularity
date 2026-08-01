//! Angular component-template frontend.
//!
//! Compiles an Angular component template into the same `Node`/`Element` AST
//! every other frontend produces, so Angular markup reaches every renderer
//! unchanged. This is a native frontend — it does not depend on the Angular
//! compiler.
//!
//! Scope: this frontend compiles the TEMPLATE only. The component class is
//! never read, so `@Input`/`@Output`, pipes, dependency injection and template
//! reference variables are all outside its scope. Expressions in markup are
//! evaluated by the shared crepuscularity evaluator against the template
//! context, exactly as `{name}` is in the indentation frontend.
//!
//! Angular's directive model is close to Vue's, so structural directives lower
//! the same way `v-if` / `v-for` do and `{{ … }}` reuses the Vue text scanner.

mod blocks;
mod builders;
mod tags;
#[cfg(test)]
mod tests;

use super::RawParseError;
use crate::ast::Node;

/// Maximum element nesting depth, mirroring the JSX frontend's guard.
pub(crate) const MAX_ANGULAR_DEPTH: usize = 256;

/// Control-flow blocks that open a new construct.
pub(crate) const BLOCK_STARTERS: &[&str] = &["if", "for", "switch", "defer"];

/// Control-flow blocks that continue or terminate an enclosing construct.
pub(crate) const BLOCK_TERMINATORS: &[&str] = &[
    "else",
    "empty",
    "case",
    "default",
    "placeholder",
    "loading",
    "error",
];

pub(crate) fn parse_angular_template(src: &str) -> Result<Vec<Node>, RawParseError> {
    let (nodes, rest) = parse_angular_nodes(src, src, 0, false)?;
    let rest = rest.trim_start();
    if !rest.is_empty() {
        return Err(angular_err(
            src,
            rest,
            format!(
                "unexpected trailing markup in Angular template: {}",
                &rest[..rest.len().min(40)]
            ),
        ));
    }
    Ok(nodes)
}

pub(crate) fn angular_err(root: &str, at: &str, message: impl Into<String>) -> RawParseError {
    RawParseError {
        message: message.into(),
        byte_offset: Some(super::subslice_byte_offset(root, at)),
    }
}

fn keyword_at(src: &str, table: &[&'static str]) -> Option<&'static str> {
    let rest = src.strip_prefix('@')?;
    table.iter().copied().find(|kw| {
        rest.strip_prefix(*kw)
            .is_some_and(|tail| !tail.starts_with(|c: char| c.is_alphanumeric() || c == '_'))
    })
}

/// The opening `@`-block keyword starting `src`, if any.
pub(crate) fn block_starter_at(src: &str) -> Option<&'static str> {
    keyword_at(src, BLOCK_STARTERS)
}

/// The continuing/terminating `@`-block keyword starting `src`, if any.
pub(crate) fn block_terminator_at(src: &str) -> Option<&'static str> {
    keyword_at(src, BLOCK_TERMINATORS)
}

pub(crate) fn parse_angular_nodes<'a>(
    root: &'a str,
    src: &'a str,
    depth: usize,
    in_block: bool,
) -> Result<(Vec<Node>, &'a str), RawParseError> {
    if depth > MAX_ANGULAR_DEPTH {
        return Err(angular_err(
            root,
            src,
            format!("maximum Angular nesting depth ({MAX_ANGULAR_DEPTH}) exceeded"),
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
        if in_block && (t.starts_with('}') || block_terminator_at(t).is_some()) {
            break;
        }

        if let Some(after) = t.strip_prefix("<!--") {
            rest = match after.find("-->") {
                Some(end) => &after[end + 3..],
                None => "",
            };
            continue;
        }

        if block_starter_at(t).is_some() || block_terminator_at(t).is_some() {
            let (produced, next) = blocks::parse_angular_block(root, t, depth)?;
            nodes.extend(produced);
            rest = next;
            continue;
        }

        if t.starts_with('<') {
            let (produced, next) = tags::parse_angular_tag(root, t, depth)?;
            nodes.extend(produced);
            rest = next;
            continue;
        }

        let (node_opt, next) = angular_text_node(t, in_block);
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

/// Consume a text run, stopping at the next tag, at a control-flow block and —
/// inside a block body — at the closing brace. `{{ … }}` groups are consumed
/// whole so their contents never look like a block terminator.
fn angular_text_node(src: &str, in_block: bool) -> (Option<Node>, &str) {
    let bytes = src.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'<' {
            break;
        }
        if src[i..].starts_with("{{") {
            match src[i + 2..].find("}}") {
                Some(end) => {
                    i += 2 + end + 2;
                    continue;
                }
                None => {
                    i = bytes.len();
                    break;
                }
            }
        }
        if in_block && bytes[i] == b'}' {
            break;
        }
        if bytes[i] == b'@'
            && (block_starter_at(&src[i..]).is_some() || block_terminator_at(&src[i..]).is_some())
        {
            break;
        }
        i += 1;
    }

    let trimmed = src[..i].trim();
    if trimmed.is_empty() {
        return (None, &src[i..]);
    }
    let parts = super::vue::template::parse_vue_text(trimmed);
    if parts.is_empty() {
        (None, &src[i..])
    } else {
        (Some(Node::Text(parts)), &src[i..])
    }
}
