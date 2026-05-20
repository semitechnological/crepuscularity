//! Server-side rendering with **hydration metadata**: stable `data-crepus-id` markers,
//! a JSON manifest (`#__crepus_hydration__`, base64-wrapped for safe embedding), and
//! optional full HTML document shells.
//!
//! Enable the **`ssr`** crate feature (on by default). For minimal WASM builds use
//! `default-features = false` on the `crepuscularity-web` dependency.
//!
//! # Manifest (`#__crepus_hydration__`)
//!
//! The script tag holds base64 (standard alphabet, no wraps) of UTF-8 JSON:
//! ```json
//! { "v": 1, "ctx": { … }, "bind": { "0": { "kind": "text", "parts": … }, … } }
//! ```
//!
//! - `ctx`: scalar template variables plus simple list/object shapes (see [`serialize_ctx_for_ssr`]).
//! - `bind`: maps numeric id strings (`"0"`, `"1"`, …) to binding descriptors. Matches `data-crepus-id="c0"` → `"0"`.

use std::cell::Cell;
use std::collections::HashMap;

use base64::Engine;
use serde_json::{json, Value};

type BindMap = serde_json::Map<String, Value>;

use crepuscularity_core::analysis::{classify_node, Region};
use crepuscularity_core::ast::*;
use crepuscularity_core::context::{value_to_str, TemplateContext, TemplateValue};
use crepuscularity_core::eval::eval_expr;
use crepuscularity_core::parser::{parse_component_file, parse_template};
use crepuscularity_core::preprocess::{slot_rotate_child_phrases, slot_rotate_words_json_attr};

/// Options for [`render_ssr_document`].
#[derive(Debug, Clone)]
pub struct SsrDocument<'a> {
    pub lang: &'a str,
    pub title: &'a str,
    /// Raw HTML inserted before `</head>` (link tags, meta, inline styles).
    pub head_extra: &'a str,
    pub body_class: Option<&'a str>,
}

impl Default for SsrDocument<'static> {
    fn default() -> Self {
        Self {
            lang: "en",
            title: "",
            head_extra: "",
            body_class: None,
        }
    }
}

/// Render like [`crate::render_template_to_html`]. With `markers: true`, emit hydration markers + manifest.
pub fn render_template_to_html_with_ssr(
    template: &str,
    ctx: &TemplateContext,
    markers: bool,
) -> Result<String, String> {
    if !markers {
        return crate::render_template_to_html(template, ctx);
    }
    let nodes = parse_template(template)?;
    let counter = Cell::new(0u32);
    let mut bind = BindMap::new();
    let mut html = render_nodes_ssr(&nodes, ctx, &counter, &mut bind, true)?;
    append_hydration_payload(&mut html, ctx, &bind)?;
    Ok(html)
}

/// Like [`crate::render_from_files`] with optional SSR markers on the entry template.
pub fn render_from_files_with_ssr(
    files: &HashMap<String, String>,
    entry: &str,
    ctx: &TemplateContext,
    markers: bool,
) -> Result<String, String> {
    let mut ctx = ctx.clone();
    ctx.virtual_files = files.clone();

    if let Some((file_part, comp_name)) = entry.split_once('#') {
        let content = files
            .get(file_part)
            .ok_or_else(|| format!("file not found in virtual fs: {file_part}"))?;
        return render_component_file_to_html_with_ssr(content, comp_name, &ctx, markers);
    }

    let content = files
        .get(entry)
        .ok_or_else(|| format!("file not found in virtual fs: {entry}"))?;
    render_template_to_html_with_ssr(content, &ctx, markers)
}

fn render_component_file_to_html_with_ssr(
    content: &str,
    component_name: &str,
    ctx: &TemplateContext,
    markers: bool,
) -> Result<String, String> {
    if !markers {
        return crate::render_component_file_to_html(content, component_name, ctx);
    }

    let file = parse_component_file(content)?;
    let component = file
        .components
        .get(component_name)
        .ok_or_else(|| format!("component not found: {component_name}"))?;

    let mut child_ctx = ctx.clone();
    for (key, expr) in &component.meta.defaults {
        child_ctx
            .vars
            .entry(key.clone())
            .or_insert_with(|| eval_expr(expr, &TemplateContext::new()));
    }

    let counter = Cell::new(0u32);
    let mut bind = BindMap::new();
    let mut html = render_nodes_ssr(&component.nodes, &child_ctx, &counter, &mut bind, true)?;
    append_hydration_payload(&mut html, &child_ctx, &bind)?;
    Ok(html)
}

/// Parse `crepus-bundle.json` and render with optional SSR markers.
pub fn render_bundle_with_ssr(bundle_json: &str, markers: bool) -> Result<String, String> {
    let root: Value = serde_json::from_str(bundle_json).map_err(|e| format!("bundle JSON: {e}"))?;
    let entry = root
        .get("entry")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "bundle missing string field \"entry\"".to_string())?
        .to_string();
    let files_val = root
        .get("files")
        .ok_or_else(|| "bundle missing \"files\" object".to_string())?;
    let files_obj = files_val
        .as_object()
        .ok_or_else(|| "\"files\" must be a JSON object".to_string())?;
    let mut files = HashMap::new();
    for (k, v) in files_obj {
        let s = v
            .as_str()
            .ok_or_else(|| format!("files[{k:?}] must be a string"))?
            .to_string();
        files.insert(k.clone(), s);
    }
    let ctx = TemplateContext::new();
    render_from_files_with_ssr(&files, &entry, &ctx, markers)
}

/// Full HTML5 document: runs the template through [`render_template_to_html_with_ssr`], then wraps result.
pub fn render_ssr_document(
    template: &str,
    ctx: &TemplateContext,
    doc: &SsrDocument<'_>,
    markers: bool,
) -> Result<String, String> {
    let inner = render_template_to_html_with_ssr(template, ctx, markers)?;
    let body_class = doc
        .body_class
        .map(|c| format!(r#" class="{}""#, crate::escape_html_attr(c)))
        .unwrap_or_default();
    let title_esc = crate::escape_html_attr(doc.title);
    Ok(format!(
        r#"<!DOCTYPE html>
<html lang="{}">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>{}</title>
  {}
</head>
<body{}>
{}
</body>
</html>
"#,
        doc.lang, title_esc, doc.head_extra, body_class, inner
    ))
}

fn append_hydration_payload(
    html: &mut String,
    ctx: &TemplateContext,
    bind: &BindMap,
) -> Result<(), String> {
    let ctx_val = serialize_ctx_for_ssr(ctx)?;
    let payload = json!({
        "v": 1,
        "ctx": ctx_val,
        "bind": Value::Object(bind.clone()),
    });
    let raw = serde_json::to_vec(&payload).map_err(|e| e.to_string())?;
    let b64 = base64::engine::general_purpose::STANDARD.encode(raw);
    let script = format!(
        r#"<script type="application/json" id="__crepus_hydration__" data-crepus-encoding="base64">{b64}</script>"#
    );
    if let Some(pos) = html.rfind("</body>") {
        html.insert_str(pos, &script);
    } else {
        html.push_str(&script);
    }
    Ok(())
}

/// Serialize [`TemplateContext::vars`] for the hydration manifest (scalars, lists of flat objects, nested maps of scalars).
pub fn serialize_ctx_for_ssr(ctx: &TemplateContext) -> Result<Value, String> {
    let mut m = BindMap::new();
    for (k, v) in &ctx.vars {
        m.insert(k.clone(), template_value_to_json(v)?);
    }
    Ok(Value::Object(m))
}

fn template_value_to_json(v: &TemplateValue) -> Result<Value, String> {
    Ok(match v {
        TemplateValue::Str(s) => Value::String(s.clone()),
        TemplateValue::Int(n) => Value::Number((*n).into()),
        TemplateValue::Float(f) => serde_json::Number::from_f64(*f)
            .map(Value::Number)
            .unwrap_or(Value::Null),
        TemplateValue::Bool(b) => Value::Bool(*b),
        TemplateValue::Null => Value::Null,
        TemplateValue::List(items) => {
            let mut arr = Vec::with_capacity(items.len());
            for item in items {
                arr.push(flat_context_object(item)?);
            }
            Value::Array(arr)
        }
        TemplateValue::Scope(ctx) => flat_context_object(ctx)?,
    })
}

fn flat_context_object(ctx: &TemplateContext) -> Result<Value, String> {
    let mut m = BindMap::new();
    for (k, v) in &ctx.vars {
        m.insert(k.clone(), template_value_to_json(v)?);
    }
    Ok(Value::Object(m))
}

fn alloc_binding(counter: &Cell<u32>, bind: &mut BindMap, detail: Value) -> u32 {
    let id = counter.get();
    counter.set(id + 1);
    bind.insert(id.to_string(), detail);
    id
}

fn render_nodes_ssr(
    nodes: &[Node],
    ctx: &TemplateContext,
    counter: &Cell<u32>,
    bind: &mut BindMap,
    mut root_element_pending: bool,
) -> Result<String, String> {
    let mut ctx = ctx.clone();
    let mut html = String::new();
    for node in nodes {
        if let Node::LetDecl(decl) = node {
            if decl.is_default && ctx.vars.contains_key(&decl.name) {
                continue;
            }
            let val = eval_expr(&decl.expr, &ctx);
            ctx.vars.insert(decl.name.clone(), val);
            continue;
        }
        html.push_str(&render_node_ssr(
            node,
            &ctx,
            counter,
            bind,
            &mut root_element_pending,
        )?);
    }
    Ok(html)
}

fn render_node_ssr(
    node: &Node,
    ctx: &TemplateContext,
    counter: &Cell<u32>,
    bind: &mut BindMap,
    root_element_pending: &mut bool,
) -> Result<String, String> {
    match node {
        Node::Element(el) => render_element_ssr(el, ctx, counter, bind, root_element_pending),
        Node::Text(parts) => {
            if classify_node(node) == Region::Dynamic {
                let id = alloc_binding(
                    counter,
                    bind,
                    json!({
                        "kind": "text",
                        "parts": text_parts_manifest(parts),
                    }),
                );
                let inner = crate::escape_html(&crate::render_text(parts, ctx));
                Ok(format!(
                    r#"<span style="display:contents" data-crepus-kind="text" data-crepus-id="c{id}">{inner}</span>"#
                ))
            } else {
                Ok(crate::escape_html(&crate::render_text(parts, ctx)))
            }
        }
        Node::If(block) => {
            let id = alloc_binding(
                counter,
                bind,
                json!({
                    "kind": "if",
                    "condition": block.condition,
                }),
            );
            let inner = if ctx.eval_condition(&block.condition) {
                render_nodes_ssr(&block.then_children, ctx, counter, bind, false)?
            } else if let Some(els) = &block.else_children {
                render_nodes_ssr(els, ctx, counter, bind, false)?
            } else {
                String::new()
            };
            Ok(format!(
                r#"<span style="display:contents" data-crepus-kind="if" data-crepus-id="c{id}">{inner}</span>"#
            ))
        }
        Node::For(block) => {
            let id = alloc_binding(
                counter,
                bind,
                json!({
                    "kind": "for",
                    "pattern": block.pattern,
                    "iterator": block.iterator,
                }),
            );
            let items = ctx.get_list(&block.iterator);
            let mut inner = String::new();
            for item_ctx in items {
                let mut child_ctx = ctx.clone();
                for (k, v) in &item_ctx.vars {
                    child_ctx.vars.insert(k.clone(), v.clone());
                }
                let pattern = block.pattern.trim();
                if !pattern.is_empty() {
                    let item_str = item_ctx.get_str("value");
                    if !item_str.is_empty() {
                        child_ctx
                            .vars
                            .insert(pattern.to_string(), TemplateValue::Str(item_str));
                    } else {
                        child_ctx
                            .vars
                            .insert(pattern.to_string(), TemplateValue::Scope(item_ctx.clone()));
                    }
                }
                inner.push_str(&render_nodes_ssr(
                    &block.body,
                    &child_ctx,
                    counter,
                    bind,
                    false,
                )?);
            }
            Ok(format!(
                r#"<span style="display:contents" data-crepus-kind="for" data-crepus-id="c{id}">{inner}</span>"#
            ))
        }
        Node::Match(block) => {
            let id = alloc_binding(
                counter,
                bind,
                json!({
                    "kind": "match",
                    "expr": block.expr,
                }),
            );
            let inner = render_match_body_ssr(block, ctx, counter, bind)?;
            Ok(format!(
                r#"<span style="display:contents" data-crepus-kind="match" data-crepus-id="c{id}">{inner}</span>"#
            ))
        }
        Node::LetDecl(_) => Ok(String::new()),
        Node::Include(inc) => render_include_ssr(inc, ctx, counter, bind),
        Node::Embed(embed) => crate::render_embed(embed, ctx),
        Node::RawText(expr) => {
            let id = alloc_binding(
                counter,
                bind,
                json!({
                    "kind": "raw",
                    "expr": expr,
                }),
            );
            let inner = crate::escape_html(&value_to_str(&eval_expr(expr, ctx)));
            Ok(format!(
                r#"<span style="display:contents" data-crepus-kind="raw" data-crepus-id="c{id}">{inner}</span>"#
            ))
        }
    }
}

fn render_match_body_ssr(
    block: &MatchBlock,
    ctx: &TemplateContext,
    counter: &Cell<u32>,
    bind: &mut BindMap,
) -> Result<String, String> {
    let val = eval_expr(&block.expr, ctx);
    let value = value_to_str(&val);

    for arm in &block.arms {
        let pattern = arm.pattern.trim();
        if pattern == "_" {
            return render_nodes_ssr(&arm.body, ctx, counter, bind, false);
        }
        if pattern.starts_with('"') && pattern.ends_with('"') {
            let lit = &pattern[1..pattern.len() - 1];
            if value == lit {
                return render_nodes_ssr(&arm.body, ctx, counter, bind, false);
            }
        }
        if value == pattern {
            return render_nodes_ssr(&arm.body, ctx, counter, bind, false);
        }
    }
    Ok(String::new())
}

fn text_parts_manifest(parts: &[TextPart]) -> Value {
    let arr: Vec<Value> = parts
        .iter()
        .map(|p| match p {
            TextPart::Literal(s) => json!({"t": "lit", "v": s}),
            TextPart::Expr(e) => json!({"t": "expr", "v": e}),
        })
        .collect();
    Value::Array(arr)
}

fn render_element_ssr(
    el: &Element,
    ctx: &TemplateContext,
    counter: &Cell<u32>,
    bind: &mut BindMap,
    root_element_pending: &mut bool,
) -> Result<String, String> {
    if el.tag == "slot" {
        return if let Some((slot_nodes, slot_ctx)) = &ctx.slot {
            render_nodes_ssr(slot_nodes, slot_ctx, counter, bind, false)
        } else {
            render_nodes_ssr(&el.children, ctx, counter, bind, false)
        };
    }

    if el.tag == "slot-rotate" {
        let phrases = slot_rotate_child_phrases(&el.children)?;
        if phrases.len() < 2 {
            return Err("slot-rotate needs at least two plain-text phrase children".into());
        }
        let mut interval_ms = 3200u64;
        for b in &el.bindings {
            if b.prop == "interval" {
                let v = value_to_str(&eval_expr(&b.value, ctx));
                let v = v.trim_matches('"').trim();
                interval_ms = v.parse().unwrap_or(3200);
            }
        }
        let words_json = slot_rotate_words_json_attr(&phrases);
        let id = alloc_binding(
            counter,
            bind,
            json!({
                "kind": "slot-rotate",
                "intervalMs": interval_ms,
                "phrases": phrases,
            }),
        );

        let mut class_names = vec!["crepus-slot".to_string()];
        class_names.extend(el.classes.clone());
        for cc in &el.conditional_classes {
            if ctx.eval_condition(&cc.condition) {
                class_names.push(cc.class.clone());
            }
        }

        let inject_root = *root_element_pending;
        *root_element_pending = false;

        let mut out = String::new();
        out.push_str("<span");
        if inject_root {
            out.push_str(r#" data-crepus-root="1""#);
        }
        out.push_str(&format!(r#" data-crepus-id="c{id}""#));
        out.push_str(" data-crepus-kind=\"slot-rotate\"");
        out.push_str(" class=\"");
        out.push_str(&crate::escape_html(
            &ctx.interpolate(&class_names.join(" ")),
        ));
        out.push('"');
        out.push_str(" data-slot-words=\"");
        out.push_str(&crate::escape_html(&words_json));
        out.push('"');
        out.push_str(" data-slot-interval=\"");
        out.push_str(&crate::escape_html(&interval_ms.to_string()));
        out.push('"');
        out.push_str(" aria-live=\"polite\"");

        for binding in &el.bindings {
            if binding.prop == "interval" {
                continue;
            }
            out.push(' ');
            out.push_str(&binding.prop);
            out.push_str("=\"");
            let value = value_to_str(&eval_expr(&binding.value, ctx));
            out.push_str(&crate::escape_html(&value));
            out.push('"');
        }

        for handler in &el.event_handlers {
            out.push(' ');
            out.push_str("data-on");
            out.push_str(&handler.event);
            out.push_str("=\"");
            out.push_str(&crate::escape_html(&handler.handler));
            out.push('"');
        }

        for animation in &el.animations {
            out.push(' ');
            out.push_str("data-animate-");
            out.push_str(&animation.property);
            out.push_str("=\"");
            out.push_str(&crate::escape_html(&format!(
                "{} {}",
                animation.duration_expr, animation.easing
            )));
            out.push('"');
        }

        out.push_str("></span>");
        return Ok(out);
    }

    let node = Node::Element(el.clone());
    let inject_id = classify_node(&node) == Region::Dynamic;
    let inject_root = inject_id && *root_element_pending;
    if inject_id {
        *root_element_pending = false;
    }

    let bind_detail = if inject_id {
        Some(json!({
            "kind": "element",
            "tag": el.tag,
        }))
    } else {
        None
    };
    let id_opt = bind_detail
        .as_ref()
        .map(|d| alloc_binding(counter, bind, d.clone()));

    let mut class_names = el.classes.clone();
    for cc in &el.conditional_classes {
        if ctx.eval_condition(&cc.condition) {
            class_names.push(cc.class.clone());
        }
    }

    let mut out = String::new();
    out.push('<');
    out.push_str(&el.tag);
    if inject_root {
        out.push_str(r#" data-crepus-root="1""#);
    }
    if let Some(id) = id_opt {
        out.push_str(&format!(
            r#" data-crepus-id="c{id}" data-crepus-kind="element""#
        ));
    }

    if !class_names.is_empty() {
        out.push_str(" class=\"");
        out.push_str(&crate::escape_html(
            &ctx.interpolate(&class_names.join(" ")),
        ));
        out.push('"');
    }

    for binding in &el.bindings {
        out.push(' ');
        out.push_str(&binding.prop);
        out.push_str("=\"");
        let value = value_to_str(&eval_expr(&binding.value, ctx));
        out.push_str(&crate::escape_html(&value));
        out.push('"');
    }

    for handler in &el.event_handlers {
        out.push(' ');
        out.push_str("data-on");
        out.push_str(&handler.event);
        out.push_str("=\"");
        out.push_str(&crate::escape_html(&handler.handler));
        out.push('"');
    }

    for animation in &el.animations {
        out.push(' ');
        out.push_str("data-animate-");
        out.push_str(&animation.property);
        out.push_str("=\"");
        out.push_str(&crate::escape_html(&format!(
            "{} {}",
            animation.duration_expr, animation.easing
        )));
        out.push('"');
    }

    out.push('>');

    for child in &el.children {
        out.push_str(&render_node_ssr(
            child,
            ctx,
            counter,
            bind,
            root_element_pending,
        )?);
    }

    out.push_str("</");
    out.push_str(&el.tag);
    out.push('>');
    Ok(out)
}

fn render_include_ssr(
    inc: &IncludeNode,
    ctx: &TemplateContext,
    counter: &Cell<u32>,
    bind: &mut BindMap,
) -> Result<String, String> {
    let id = alloc_binding(
        counter,
        bind,
        json!({
            "kind": "include",
            "path": inc.path,
        }),
    );
    let inner = if let Some((file_part, comp_name)) = inc.path.split_once('#') {
        render_named_component_ssr(inc, ctx, file_part, comp_name, counter, bind)?
    } else {
        let file_path = crate::resolve_include_path(ctx.base_dir.as_deref(), &inc.path)?;
        let content = crate::read_file(ctx, &file_path)?;
        let nodes = parse_template(&content).map_err(|e| format!("include parse error: {e}"))?;
        let mut child_ctx = TemplateContext::new();
        child_ctx.base_dir = file_path.parent().map(|p| p.to_path_buf());
        child_ctx.virtual_files = ctx.virtual_files.clone();
        for (key, expr) in &inc.props {
            child_ctx.vars.insert(key.clone(), eval_expr(expr, ctx));
        }
        if !inc.slot.is_empty() {
            child_ctx.slot = Some((inc.slot.clone(), Box::new(ctx.clone())));
        }
        render_nodes_ssr(&nodes, &child_ctx, counter, bind, false)?
    };
    Ok(format!(
        r#"<span style="display:contents" data-crepus-kind="include" data-crepus-id="c{id}">{inner}</span>"#
    ))
}

fn render_named_component_ssr(
    inc: &IncludeNode,
    ctx: &TemplateContext,
    file_part: &str,
    comp_name: &str,
    counter: &Cell<u32>,
    bind: &mut BindMap,
) -> Result<String, String> {
    let file_path = crate::resolve_include_path(ctx.base_dir.as_deref(), file_part)?;
    let content = crate::read_file(ctx, &file_path)?;
    let comp_file =
        parse_component_file(&content).map_err(|e| format!("component file parse error: {e}"))?;
    let comp = comp_file
        .components
        .get(comp_name)
        .ok_or_else(|| format!("component '{}' not found in {}", comp_name, file_part))?;

    let mut child_ctx = TemplateContext::new();
    child_ctx.base_dir = file_path.parent().map(|p| p.to_path_buf());
    child_ctx.virtual_files = ctx.virtual_files.clone();
    for (key, expr) in &comp.meta.defaults {
        child_ctx
            .vars
            .insert(key.clone(), eval_expr(expr, &TemplateContext::new()));
    }
    for (key, expr) in &inc.props {
        child_ctx.vars.insert(key.clone(), eval_expr(expr, ctx));
    }
    if !inc.slot.is_empty() {
        child_ctx.slot = Some((inc.slot.clone(), Box::new(ctx.clone())));
    }

    render_nodes_ssr(&comp.nodes, &child_ctx, counter, bind, false)
}
