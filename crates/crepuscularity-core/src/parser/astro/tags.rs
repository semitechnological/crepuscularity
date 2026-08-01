//! Astro element and attribute parsing.

use crate::ast::*;

use super::super::jsx::attrs::{jsx_parse_attrs, JsxAttr, JsxAttrValue};
use super::super::jsx::builders::jsx_build_element;
use super::super::jsx::text::jsx_close;
use super::super::vue::builders::apply_dynamic_class;
use super::super::RawParseError;
use super::{astro_err, parse_astro_nodes};

const VOID_TAGS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
    "track", "wbr",
];

/// Directives whose runtime semantics this frontend cannot honour.
const UNSUPPORTED_DIRECTIVES: &[&str] = &["transition:", "define:vars", "is:raw", "is:inline"];

/// Attributes extracted from an Astro tag that do not map onto a JSX attribute.
#[derive(Default)]
struct AstroExtras {
    classes: Vec<String>,
    conditional_classes: Vec<ConditionalClass>,
    replacement_children: Option<Vec<Node>>,
}

pub(crate) fn parse_astro_tag<'a>(
    root: &'a str,
    src: &'a str,
    depth: usize,
) -> Result<(Vec<Node>, &'a str), RawParseError> {
    let after_lt = &src[1..];
    let name_end = after_lt
        .find(|c: char| c.is_whitespace() || c == '>' || c == '/')
        .unwrap_or(after_lt.len());
    let tag = &after_lt[..name_end];

    if tag == "slot" {
        return Err(astro_err(
            root,
            src,
            "`<slot />` is not supported by the Astro frontend \
             (use crepuscularity `include` slots instead)",
        ));
    }
    if tag != "Fragment" && tag.starts_with(|c: char| c.is_ascii_uppercase()) {
        return Err(astro_err(
            root,
            src,
            format!(
                "component tag `<{tag}>` is not supported: this frontend compiles markup only \
                 and performs no module resolution"
            ),
        ));
    }

    let attrs_src = after_lt[name_end..].trim_start();
    let (attrs, after_gt, self_closing) = jsx_parse_attrs(root, attrs_src)?;
    let (attrs, extras) = rewrite_astro_attrs(root, src, attrs)?;

    let (children, rest) = if self_closing || VOID_TAGS.contains(&tag) {
        (Vec::new(), after_gt)
    } else {
        let (children, rest) = parse_astro_nodes(root, after_gt, depth + 1)?;
        let rest = jsx_close(root, rest, tag)?;
        (children, rest)
    };

    let children = extras.replacement_children.unwrap_or(children);

    // `<Fragment>` is a structural wrapper: it contributes its children rather
    // than an element of its own.
    if tag == "Fragment" {
        if !attrs.is_empty() || !extras.classes.is_empty() || !extras.conditional_classes.is_empty()
        {
            return Err(astro_err(
                root,
                src,
                "`<Fragment>` accepts only `set:html` / `set:text`: it produces no element, \
                 so other attributes would be dropped",
            ));
        }
        return Ok((children, rest));
    }

    let mut element = jsx_build_element(tag, attrs, children);
    element.classes.extend(extras.classes);
    element
        .conditional_classes
        .extend(extras.conditional_classes);
    Ok((vec![Node::Element(element)], rest))
}

/// Rewrite Astro attribute syntax into the keys `jsx_build_element` understands.
///
/// `client:*` hydration directives are recorded verbatim as bindings rather than
/// dropped: the shared IR has no hydration concept, but keeping the directive on
/// the element means no information is lost silently.
fn rewrite_astro_attrs(
    root: &str,
    src: &str,
    attrs: Vec<JsxAttr>,
) -> Result<(Vec<JsxAttr>, AstroExtras), RawParseError> {
    let mut out = Vec::with_capacity(attrs.len());
    let mut extras = AstroExtras::default();

    for attr in attrs {
        let key = attr.key.clone();
        let key = key.as_str();

        // `{value}` shorthand and `{...spread}`.
        if let Some(inner) = key.strip_prefix('{') {
            let Some(name) = inner.strip_suffix('}') else {
                return Err(astro_err(
                    root,
                    src,
                    format!("malformed shorthand attribute `{key}`"),
                ));
            };
            let name = name.trim();
            if name.starts_with("...") {
                return Err(astro_err(
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
            return Err(astro_err(
                root,
                src,
                format!(
                    "`{prefix}` directives are not supported by the Astro frontend \
                     (found `{key}`)"
                ),
            ));
        }

        if key == "class:list" {
            let expr = match &attr.value {
                JsxAttrValue::Expr(e) => e.trim().to_string(),
                JsxAttrValue::Str(s) => {
                    extras
                        .classes
                        .extend(s.split_whitespace().map(|c| c.to_string()));
                    continue;
                }
                JsxAttrValue::Bool(_) => continue,
            };
            apply_dynamic_class(&expr, &mut extras.classes, &mut extras.conditional_classes)
                .map_err(|e| astro_err(root, src, e.message))?;
            continue;
        }

        if key == "set:html" {
            extras.replacement_children = Some(vec![Node::RawHtml(directive_expr(&attr))]);
            continue;
        }
        if key == "set:text" {
            extras.replacement_children = Some(vec![Node::Text(vec![TextPart::Expr(
                directive_expr(&attr),
            )])]);
            continue;
        }

        if key.starts_with("class:") {
            return Err(astro_err(
                root,
                src,
                format!("`{key}` is not an Astro directive (Astro spells it `class:list`)"),
            ));
        }

        out.push(attr);
    }

    Ok((out, extras))
}

/// Directive values are expression source whether they are quoted or braced.
fn directive_expr(attr: &JsxAttr) -> String {
    match &attr.value {
        JsxAttrValue::Str(s) => s.trim().to_string(),
        JsxAttrValue::Expr(e) => e.trim().to_string(),
        JsxAttrValue::Bool(_) => String::new(),
    }
}
