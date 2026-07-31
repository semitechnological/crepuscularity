//! Svelte element and attribute parsing.

use crate::ast::Node;

use super::super::jsx::attrs::{jsx_parse_attrs, JsxAttr, JsxAttrValue};
use super::super::jsx::builders::jsx_build_element;
use super::super::jsx::text::jsx_close;
use super::super::RawParseError;
use super::{parse_svelte_nodes, svelte_err};

const VOID_TAGS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
    "track", "wbr",
];

/// Directives whose runtime semantics this frontend cannot honour.
const UNSUPPORTED_DIRECTIVES: &[&str] = &[
    "use:",
    "transition:",
    "in:",
    "out:",
    "animate:",
    "let:",
    "style:",
];

/// Lowercase DOM events recognised in Svelte 5 attribute form (`onclick={…}`).
const DOM_EVENTS: &[&str] = &[
    "click",
    "dblclick",
    "input",
    "change",
    "submit",
    "reset",
    "select",
    "focus",
    "focusin",
    "focusout",
    "blur",
    "keydown",
    "keyup",
    "keypress",
    "mousedown",
    "mouseup",
    "mousemove",
    "mouseenter",
    "mouseleave",
    "mouseover",
    "mouseout",
    "contextmenu",
    "wheel",
    "scroll",
    "toggle",
    "load",
    "error",
    "copy",
    "cut",
    "paste",
    "pointerdown",
    "pointerup",
    "pointermove",
    "pointerenter",
    "pointerleave",
    "touchstart",
    "touchend",
    "touchmove",
    "touchcancel",
    "dragstart",
    "dragend",
    "dragover",
    "dragenter",
    "dragleave",
    "drop",
    "animationend",
    "transitionend",
];

pub(crate) fn parse_svelte_tag<'a>(
    root: &'a str,
    src: &'a str,
    depth: usize,
) -> Result<(Node, &'a str), RawParseError> {
    let src = src.trim_start();
    let after_lt = &src[1..];
    let name_end = after_lt
        .find(|c: char| c.is_whitespace() || c == '>' || c == '/')
        .unwrap_or(after_lt.len());
    let tag = &after_lt[..name_end];

    if tag.starts_with("svelte:") {
        return Err(svelte_err(
            root,
            src,
            format!("`<{tag}>` special elements are not supported by the Svelte frontend"),
        ));
    }
    if tag == "slot" {
        return Err(svelte_err(
            root,
            src,
            "`<slot>` is not supported by the Svelte frontend \
             (use crepuscularity `include` slots instead)",
        ));
    }
    if tag.starts_with(|c: char| c.is_ascii_uppercase()) {
        return Err(svelte_err(
            root,
            src,
            format!(
                "component tag `<{tag}>` is not supported: this frontend compiles markup only \
                 and performs no module resolution"
            ),
        ));
    }

    let rest = after_lt[name_end..].trim_start();
    let (attrs, after_gt, self_closing) = jsx_parse_attrs(root, rest)?;
    let attrs = rewrite_svelte_attrs(root, src, attrs)?;

    if self_closing || VOID_TAGS.contains(&tag) {
        return Ok((
            Node::Element(jsx_build_element(tag, attrs, vec![])),
            after_gt,
        ));
    }

    let (children, rest) = parse_svelte_nodes(root, after_gt, depth + 1)?;
    let rest = jsx_close(root, rest, tag)?;
    Ok((Node::Element(jsx_build_element(tag, attrs, children)), rest))
}

/// Rewrite Svelte attribute syntax into the keys `jsx_build_element` understands.
fn rewrite_svelte_attrs(
    root: &str,
    src: &str,
    attrs: Vec<JsxAttr>,
) -> Result<Vec<JsxAttr>, RawParseError> {
    let mut out = Vec::with_capacity(attrs.len());

    for attr in attrs {
        let key = attr.key.clone();
        let key = key.as_str();

        // `{value}` shorthand and `{...spread}`.
        if let Some(inner) = key.strip_prefix('{') {
            let Some(name) = inner.strip_suffix('}') else {
                return Err(svelte_err(
                    root,
                    src,
                    format!("malformed shorthand attribute `{key}`"),
                ));
            };
            let name = name.trim();
            if name.starts_with("...") {
                return Err(svelte_err(
                    root,
                    src,
                    "spread attributes `{...props}` are not supported: the shared AST \
                     records named bindings only",
                ));
            }
            out.push(JsxAttr {
                key: name.to_string(),
                value: JsxAttrValue::Expr(name.to_string()),
            });
            continue;
        }

        if let Some(prefix) = UNSUPPORTED_DIRECTIVES.iter().find(|p| key.starts_with(**p)) {
            return Err(svelte_err(
                root,
                src,
                format!(
                    "`{prefix}` directives are not supported by the Svelte frontend \
                     (found `{key}`)"
                ),
            ));
        }

        // `on:click={h}` / `on:click|once={h}` → `@click` / `@click|once`.
        if let Some(event) = key.strip_prefix("on:") {
            out.push(JsxAttr {
                key: format!("@{event}"),
                value: attr.value,
            });
            continue;
        }

        // Svelte 5 `onclick={h}` → `@click`.
        if let Some(event) = key.strip_prefix("on") {
            if DOM_EVENTS.contains(&event) {
                out.push(JsxAttr {
                    key: format!("@{event}"),
                    value: attr.value,
                });
                continue;
            }
        }

        // `class:foo` / `bind:value` shorthand — the bare attribute means the
        // expression is the directive name itself.
        let mut attr = attr;
        if (key.starts_with("class:") || key.starts_with("bind:"))
            && matches!(attr.value, JsxAttrValue::Bool(true))
        {
            let name = key
                .split_once(':')
                .map(|(_, n)| n)
                .unwrap_or("")
                .to_string();
            attr.value = JsxAttrValue::Expr(name);
        }

        // Svelte's two-way form bindings are the shared AST's `bind=` binding,
        // which every renderer already wires up for inputs, toggles and pickers.
        if matches!(
            attr.key.as_str(),
            "bind:value" | "bind:checked" | "bind:group" | "bind:files"
        ) {
            attr.key = "bind:bind".to_string();
        }

        out.push(attr);
    }

    Ok(out)
}
