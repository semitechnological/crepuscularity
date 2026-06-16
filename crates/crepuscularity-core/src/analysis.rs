//! Binding analysis and fingerprinting for the `.crepus` driver pipeline.
//!
//! # Binding analysis
//!
//! [`analyze_template`] walks the AST and classifies each top-level node as
//! [`Region::Static`] (no runtime expressions anywhere in the subtree) or
//! [`Region::Dynamic`] (contains at least one binding site: text interpolation,
//! dynamic attribute, conditional class, `if`, `for`, `match`, or `include`).
//!
//! # Fingerprinting
//!
//! [`Fingerprint`] gives every `(source, section?, stage)` triple a stable,
//! content-addressed identity (SHA-256 of the source bytes). The driver cache
//! ([`crate::cache::DriverCache`]) uses this as the cache key so that a second
//! save with no logical change does not rewrite generated output and therefore
//! does not bump `rustc`'s input mtime.

use sha2::{Digest, Sha256};

use crate::ast::*;
use crate::util::bytes_to_hex;

// ── Region ───────────────────────────────────────────────────────────────────

/// Whether a node subtree contains any runtime-evaluated binding sites.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Region {
    /// No runtime expressions anywhere in this subtree — fully static HTML.
    Static,
    /// Contains at least one binding site: expression, control flow, or include.
    Dynamic,
}

// ── BindingMap ───────────────────────────────────────────────────────────────

/// A flat summary of the static vs dynamic regions in a template.
#[derive(Debug, Clone)]
pub struct BindingMap {
    /// Indices of top-level nodes that are entirely static.
    pub static_indices: Vec<usize>,
    /// Indices of top-level nodes that have at least one binding site.
    pub dynamic_indices: Vec<usize>,
    /// Total number of expression binding sites across the whole template.
    pub expr_count: usize,
}

/// Classify all top-level nodes in a template and return a [`BindingMap`].
pub fn analyze_template(nodes: &[Node]) -> BindingMap {
    let mut static_indices = Vec::new();
    let mut dynamic_indices = Vec::new();
    let mut expr_count = 0;

    for (i, node) in nodes.iter().enumerate() {
        let (region, count) = classify_node_inner(node);
        expr_count += count;
        if region == Region::Dynamic {
            dynamic_indices.push(i);
        } else {
            static_indices.push(i);
        }
    }

    BindingMap {
        static_indices,
        dynamic_indices,
        expr_count,
    }
}

/// Classify a single node as [`Region::Static`] or [`Region::Dynamic`].
pub fn classify_node(node: &Node) -> Region {
    classify_node_inner(node).0
}

fn classify_node_inner(node: &Node) -> (Region, usize) {
    match node {
        Node::Text(parts) => {
            let expr_count = parts
                .iter()
                .filter(|p| matches!(p, TextPart::Expr(_)))
                .count();
            if expr_count > 0 {
                (Region::Dynamic, expr_count)
            } else {
                (Region::Static, 0)
            }
        }

        Node::Element(el) => classify_element(el),

        // Control flow and dynamic node types are always dynamic.
        Node::If(block) => {
            let mut count = 1; // the condition itself
            for n in &block.then_children {
                count += classify_node_inner(n).1;
            }
            if let Some(els) = &block.else_children {
                for n in els {
                    count += classify_node_inner(n).1;
                }
            }
            (Region::Dynamic, count)
        }

        Node::For(block) => {
            let mut count = 1; // the iterator expression
            for n in &block.body {
                count += classify_node_inner(n).1;
            }
            (Region::Dynamic, count)
        }

        Node::Match(block) => {
            let mut count = 1; // the match expression
            for arm in &block.arms {
                for n in &arm.body {
                    count += classify_node_inner(n).1;
                }
            }
            (Region::Dynamic, count)
        }

        Node::LetDecl(_) => (Region::Dynamic, 1),
        Node::Include(inc) => (Region::Dynamic, inc.props.len().max(1)),
        Node::Embed(embed) => (Region::Dynamic, embed.props.len().max(1)),
        Node::RawText(_) | Node::RawHtml(_) => (Region::Dynamic, 1),
    }
}

fn classify_element(el: &Element) -> (Region, usize) {
    let mut expr_count = 0;
    let mut is_dynamic = false;

    if el.tag == "slot-rotate" {
        return (Region::Dynamic, 1);
    }

    // Dynamic attribute bindings.
    expr_count += el.bindings.len();
    if !el.bindings.is_empty() {
        is_dynamic = true;
    }

    // Conditional classes.
    expr_count += el.conditional_classes.len();
    if !el.conditional_classes.is_empty() {
        is_dynamic = true;
    }

    // Class tokens that use `{…}` interpolation are dynamic.
    if el.classes.iter().any(|c| c.contains('{')) {
        is_dynamic = true;
    }

    // GPUI / web animations participate in runtime binding.
    if !el.animations.is_empty() {
        is_dynamic = true;
        expr_count += el.animations.len();
    }

    // Event handlers make the element dynamic (client-side binding site).
    if !el.event_handlers.is_empty() {
        is_dynamic = true;
        expr_count += el.event_handlers.len();
    }

    // Recurse into children.
    for child in &el.children {
        let (child_region, child_count) = classify_node_inner(child);
        expr_count += child_count;
        if child_region == Region::Dynamic {
            is_dynamic = true;
        }
    }

    if is_dynamic {
        (Region::Dynamic, expr_count)
    } else {
        (Region::Static, 0)
    }
}

// ── Fingerprint ──────────────────────────────────────────────────────────────

/// A content-addressed identity for a `(source, section?, stage)` triple.
///
/// The `content_hash` is the SHA-256 hex digest of the source bytes.
/// `stage` is one of `"parse"`, `"analyze"`, `"render"`, or `"codegen"`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Fingerprint {
    /// Full hex SHA-256 of the source content.
    pub content_hash: String,
    /// Component/section name for multi-component files, or `None` for whole-file ops.
    pub section: Option<String>,
    /// Pipeline stage: `"parse"`, `"analyze"`, `"render"`, `"codegen"`.
    pub stage: String,
}

impl Fingerprint {
    /// Build a fingerprint from raw source text.
    pub fn new(source: &str, section: Option<&str>, stage: &str) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(source.as_bytes());
        let hash = bytes_to_hex(hasher.finalize().as_ref());
        Self {
            content_hash: hash,
            section: section.map(|s| s.to_string()),
            stage: stage.to_string(),
        }
    }

    /// A filesystem-safe cache key derived from this fingerprint.
    ///
    /// Format: `{first_16_hex_chars}-{section_or_underscore}-{stage}`
    pub fn cache_key(&self) -> String {
        let sec = self.section.as_deref().unwrap_or("_");
        format!("{}-{}-{}", &self.content_hash[..16], sec, self.stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fingerprint_is_deterministic() {
        let a = Fingerprint::new("hello", None, "render");
        let b = Fingerprint::new("hello", None, "render");
        assert_eq!(a.content_hash, b.content_hash);
        assert_eq!(a.cache_key(), b.cache_key());
    }

    #[test]
    fn different_source_different_fingerprint() {
        let a = Fingerprint::new("hello", None, "render");
        let b = Fingerprint::new("world", None, "render");
        assert_ne!(a.content_hash, b.content_hash);
    }

    #[test]
    fn section_in_cache_key() {
        let fp = Fingerprint::new("src", Some("Card"), "codegen");
        assert!(fp.cache_key().contains("Card"));
    }
}
