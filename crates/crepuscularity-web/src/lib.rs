use std::path::{Path, PathBuf};

use crepuscularity_core::ast::*;
use crepuscularity_core::context::{value_to_str, TemplateContext, TemplateValue};
use crepuscularity_core::eval::eval_expr;
use crepuscularity_core::parser::{parse_component_file, parse_template};
use crepuscularity_core::preprocess::{slot_rotate_child_phrases, slot_rotate_words_json_attr};

mod bundle;

pub use bundle::render_bundle;
pub use crepuscularity_core::preprocess::google_fonts_head_markup;

/// Render an entry point from an in-memory file map — no filesystem access.
///
/// `entry` is `"path/to/file.crepus"` or `"path/to/file.crepus#ComponentName"`.
/// `files` maps paths to `.crepus` source strings.
/// All `include` directives within the templates are resolved from `files`.
#[tracing::instrument(skip(files, ctx), fields(entry = entry))]
pub fn render_from_files(
    files: &std::collections::HashMap<String, String>,
    entry: &str,
    ctx: &TemplateContext,
) -> Result<String, String> {
    let mut ctx = ctx.clone();
    ctx.virtual_files = files.clone();

    if let Some((file_part, comp_name)) = entry.split_once('#') {
        let content = files
            .get(file_part)
            .ok_or_else(|| format!("file not found in virtual fs: {file_part}"))?;
        return render_component_file_to_html(content, comp_name, &ctx);
    }

    let content = files
        .get(entry)
        .ok_or_else(|| format!("file not found in virtual fs: {entry}"))?;
    render_template_to_html(content, &ctx)
}

/// Render multiple entry points from a virtual file map in parallel (requires `parallel` feature).
///
/// Returns a `Vec` of `(entry, Result<String, String>)` in the same order as `entries`.
/// Each entry is independent — rendering runs on a Rayon thread pool.
/// Falls back to sequential iteration when the `parallel` feature is disabled (e.g. WASM).
pub fn par_render_from_files(
    files: &std::collections::HashMap<String, String>,
    entries: &[&str],
    ctx: &TemplateContext,
) -> Vec<(String, Result<String, String>)> {
    #[cfg(feature = "parallel")]
    {
        use rayon::prelude::*;
        entries
            .par_iter()
            .map(|&entry| (entry.to_string(), render_from_files(files, entry, ctx)))
            .collect()
    }
    #[cfg(not(feature = "parallel"))]
    {
        entries
            .iter()
            .map(|&entry| (entry.to_string(), render_from_files(files, entry, ctx)))
            .collect()
    }
}

/// Render all named components from a multi-component `.crepus` file in parallel.
///
/// Returns a `HashMap<component_name, Result<html, error>>`.
/// Falls back to sequential iteration when the `parallel` feature is disabled.
pub fn par_render_component_file(
    content: &str,
    ctx: &TemplateContext,
) -> Result<std::collections::HashMap<String, Result<String, String>>, String> {
    let file = parse_component_file(content)?;

    #[cfg(feature = "parallel")]
    {
        use rayon::prelude::*;
        let results = file
            .components
            .par_iter()
            .map(|(name, comp)| {
                let mut child_ctx = ctx.clone();
                for (key, expr) in &comp.meta.defaults {
                    child_ctx
                        .vars
                        .entry(key.clone())
                        .or_insert_with(|| eval_expr(expr, &TemplateContext::new()));
                }
                let html = render_nodes_to_html(&comp.nodes, &child_ctx);
                (name.clone(), html)
            })
            .collect();
        Ok(results)
    }
    #[cfg(not(feature = "parallel"))]
    {
        let results = file
            .components
            .iter()
            .map(|(name, comp)| {
                let mut child_ctx = ctx.clone();
                for (key, expr) in &comp.meta.defaults {
                    child_ctx
                        .vars
                        .entry(key.clone())
                        .or_insert_with(|| eval_expr(expr, &TemplateContext::new()));
                }
                let html = render_nodes_to_html(&comp.nodes, &child_ctx);
                (name.clone(), html)
            })
            .collect();
        Ok(results)
    }
}

#[tracing::instrument(skip(template, ctx), fields(template_len = template.len()))]
pub fn render_template_to_html(template: &str, ctx: &TemplateContext) -> Result<String, String> {
    let nodes = parse_template(template)?;
    render_nodes_to_html(&nodes, ctx)
}

pub fn render_component_file_to_html(
    content: &str,
    component_name: &str,
    ctx: &TemplateContext,
) -> Result<String, String> {
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

    render_nodes_to_html(&component.nodes, &child_ctx)
}

pub fn render_nodes_to_html(nodes: &[Node], ctx: &TemplateContext) -> Result<String, String> {
    render_nodes_with_ctx(nodes, ctx.clone())
}

fn render_nodes_with_ctx(nodes: &[Node], mut ctx: TemplateContext) -> Result<String, String> {
    let _span = tracing::debug_span!("render_html", node_count = nodes.len()).entered();
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
        html.push_str(&render_node(node, &ctx)?);
    }

    Ok(html)
}

fn render_node(node: &Node, ctx: &TemplateContext) -> Result<String, String> {
    match node {
        Node::Element(el) => render_element(el, ctx),
        Node::Text(parts) => Ok(escape_html(&render_text(parts, ctx))),
        Node::If(block) => render_if(block, ctx),
        Node::For(block) => render_for(block, ctx),
        Node::Match(block) => render_match(block, ctx),
        Node::LetDecl(_) => Ok(String::new()),
        Node::Include(inc) => render_include(inc, ctx),
        Node::RawText(expr) => Ok(escape_html(&value_to_str(&eval_expr(expr, ctx)))),
    }
}

fn render_element(el: &Element, ctx: &TemplateContext) -> Result<String, String> {
    if el.tag == "slot" {
        return if let Some((slot_nodes, slot_ctx)) = &ctx.slot {
            render_nodes_to_html(slot_nodes, slot_ctx)
        } else {
            render_nodes_to_html(&el.children, ctx)
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

        let mut class_names = vec!["crepus-slot".to_string()];
        class_names.extend(el.classes.clone());
        for cc in &el.conditional_classes {
            if ctx.eval_condition(&cc.condition) {
                class_names.push(cc.class.clone());
            }
        }

        let mut out = String::new();
        out.push_str("<span");
        out.push_str(" class=\"");
        out.push_str(&escape_html(&ctx.interpolate(&class_names.join(" "))));
        out.push('"');
        out.push_str(" data-slot-words=\"");
        out.push_str(&escape_html(&words_json));
        out.push('"');
        out.push_str(" data-slot-interval=\"");
        out.push_str(&escape_html(&interval_ms.to_string()));
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
            out.push_str(&escape_html(&value));
            out.push('"');
        }

        for handler in &el.event_handlers {
            out.push(' ');
            out.push_str("data-on");
            out.push_str(&handler.event);
            out.push_str("=\"");
            out.push_str(&escape_html(&handler.handler));
            out.push('"');
        }

        for animation in &el.animations {
            out.push(' ');
            out.push_str("data-animate-");
            out.push_str(&animation.property);
            out.push_str("=\"");
            out.push_str(&escape_html(&format!(
                "{} {}",
                animation.duration_expr, animation.easing
            )));
            out.push('"');
        }

        out.push_str("></span>");
        return Ok(out);
    }

    let mut class_names = el.classes.clone();
    for cc in &el.conditional_classes {
        if ctx.eval_condition(&cc.condition) {
            class_names.push(cc.class.clone());
        }
    }

    let mut out = String::new();
    out.push('<');
    out.push_str(&el.tag);

    if !class_names.is_empty() {
        out.push_str(" class=\"");
        out.push_str(&escape_html(&ctx.interpolate(&class_names.join(" "))));
        out.push('"');
    }

    for binding in &el.bindings {
        out.push(' ');
        out.push_str(&binding.prop);
        out.push_str("=\"");
        let value = value_to_str(&eval_expr(&binding.value, ctx));
        out.push_str(&escape_html(&value));
        out.push('"');
    }

    for handler in &el.event_handlers {
        out.push(' ');
        out.push_str("data-on");
        out.push_str(&handler.event);
        out.push_str("=\"");
        out.push_str(&escape_html(&handler.handler));
        out.push('"');
    }

    for animation in &el.animations {
        out.push(' ');
        out.push_str("data-animate-");
        out.push_str(&animation.property);
        out.push_str("=\"");
        out.push_str(&escape_html(&format!(
            "{} {}",
            animation.duration_expr, animation.easing
        )));
        out.push('"');
    }

    out.push('>');

    for child in &el.children {
        out.push_str(&render_node(child, ctx)?);
    }

    out.push_str("</");
    out.push_str(&el.tag);
    out.push('>');
    Ok(out)
}

fn render_text(parts: &[TextPart], ctx: &TemplateContext) -> String {
    let mut result = String::new();
    for part in parts {
        match part {
            TextPart::Literal(text) => result.push_str(text),
            TextPart::Expr(expr) => result.push_str(&value_to_str(&eval_expr(expr, ctx))),
        }
    }
    result
}

fn render_if(block: &IfBlock, ctx: &TemplateContext) -> Result<String, String> {
    if ctx.eval_condition(&block.condition) {
        render_nodes_to_html(&block.then_children, ctx)
    } else if let Some(else_children) = &block.else_children {
        render_nodes_to_html(else_children, ctx)
    } else {
        Ok(String::new())
    }
}

fn render_for(block: &ForBlock, ctx: &TemplateContext) -> Result<String, String> {
    let items = ctx.get_list(&block.iterator);
    let mut out = String::new();

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
            }
        }

        out.push_str(&render_nodes_to_html(&block.body, &child_ctx)?);
    }

    Ok(out)
}

fn render_match(block: &MatchBlock, ctx: &TemplateContext) -> Result<String, String> {
    let val = eval_expr(&block.expr, ctx);
    let value = value_to_str(&val);

    for arm in &block.arms {
        let pattern = arm.pattern.trim();
        if pattern == "_" {
            return render_nodes_to_html(&arm.body, ctx);
        }
        if pattern.starts_with('"') && pattern.ends_with('"') {
            let lit = &pattern[1..pattern.len() - 1];
            if value == lit {
                return render_nodes_to_html(&arm.body, ctx);
            }
        }
        if value == pattern {
            return render_nodes_to_html(&arm.body, ctx);
        }
    }

    Ok(String::new())
}

fn read_file(ctx: &TemplateContext, path: &Path) -> Result<String, String> {
    // Check virtual files first (enables WASM / no-filesystem rendering).
    let key = path.to_string_lossy();
    if let Some(content) = ctx.virtual_files.get(key.as_ref()) {
        return Ok(content.clone());
    }
    // Also check with just the filename portion for relative paths.
    for (vkey, content) in &ctx.virtual_files {
        if vkey.ends_with(key.as_ref()) || key.ends_with(vkey.as_str()) {
            return Ok(content.clone());
        }
    }
    if cfg!(not(target_arch = "wasm32")) {
        std::fs::read_to_string(path).map_err(|e| format!("include error: {:?}: {}", path, e))
    } else {
        Err(format!(
            "include error: file not in virtual bundle: {}",
            key
        ))
    }
}

fn render_include(inc: &IncludeNode, ctx: &TemplateContext) -> Result<String, String> {
    if let Some((file_part, comp_name)) = inc.path.split_once('#') {
        return render_named_component(inc, ctx, file_part, comp_name);
    }

    let file_path = resolve_include_path(ctx.base_dir.as_deref(), &inc.path);
    let content = read_file(ctx, &file_path)?;
    let nodes = parse_template(&content).map_err(|e| format!("include parse error: {}", e))?;

    let mut child_ctx = TemplateContext::new();
    child_ctx.base_dir = file_path.parent().map(|p| p.to_path_buf());
    child_ctx.virtual_files = ctx.virtual_files.clone();
    for (key, expr) in &inc.props {
        child_ctx.vars.insert(key.clone(), eval_expr(expr, ctx));
    }
    if !inc.slot.is_empty() {
        child_ctx.slot = Some((inc.slot.clone(), Box::new(ctx.clone())));
    }

    render_nodes_to_html(&nodes, &child_ctx)
}

fn render_named_component(
    inc: &IncludeNode,
    ctx: &TemplateContext,
    file_part: &str,
    comp_name: &str,
) -> Result<String, String> {
    let file_path = resolve_include_path(ctx.base_dir.as_deref(), file_part);
    let content = read_file(ctx, &file_path)?;
    let comp_file =
        parse_component_file(&content).map_err(|e| format!("component file parse error: {}", e))?;
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

    render_nodes_to_html(&comp.nodes, &child_ctx)
}

fn resolve_include_path(base_dir: Option<&Path>, path: &str) -> PathBuf {
    let candidate = if let Some(base) = base_dir {
        base.join(path)
    } else {
        PathBuf::from(path)
    };
    if cfg!(not(target_arch = "wasm32")) {
        std::fs::canonicalize(&candidate).unwrap_or(candidate)
    } else {
        candidate
    }
}

// ── Hydration ─────────────────────────────────────────────────────────────────

/// Returns `true` if a node is "dynamic" — contains interpolated expressions,
/// control flow, or other content that can change based on context.
fn node_is_dynamic(node: &Node) -> bool {
    match node {
        Node::If(_) | Node::For(_) | Node::Match(_) | Node::RawText(_) => true,
        Node::Text(parts) => parts
            .iter()
            .any(|p| matches!(p, crepuscularity_core::ast::TextPart::Expr(_))),
        Node::Element(el) => {
            !el.conditional_classes.is_empty()
                || !el.bindings.is_empty()
                || el.children.iter().any(node_is_dynamic)
        }
        Node::Include(_) => true,
        Node::LetDecl(_) => false,
    }
}

/// Render a template to HTML with hydration markers injected.
///
/// - The root wrapping element gets `data-crepus-root`.
/// - Each dynamic descendant element gets a unique `data-crepus-id="N"`.
/// - A `<script>window.__crepus_ctx__ = {...};</script>` is appended with the
///   serialized context variables.
///
/// Enable with the `hydration` cargo feature.
#[cfg(feature = "hydration")]
pub fn render_template_to_html_with_hydration(
    template: &str,
    ctx: &TemplateContext,
) -> Result<String, String> {
    use std::sync::atomic::AtomicU32;

    let nodes = parse_template(template)?;

    // Counter for data-crepus-id assignment.
    let counter = AtomicU32::new(0);

    let rendered = render_nodes_with_hydration_impl(&nodes, ctx, &counter, /*is_root=*/ true)?;

    // Serialize context vars to JSON.
    let ctx_json = serialize_ctx_to_json(ctx);
    let script = format!("<script>window.__crepus_ctx__ = {};</script>", ctx_json);

    Ok(format!("{rendered}{script}"))
}

#[cfg(feature = "hydration")]
fn render_nodes_with_hydration_impl(
    nodes: &[Node],
    ctx: &TemplateContext,
    counter: &std::sync::atomic::AtomicU32,
    is_root: bool,
) -> Result<String, String> {
    use std::sync::atomic::Ordering;

    // Process LetDecl nodes first.
    let mut ctx = ctx.clone();
    for node in nodes {
        if let Node::LetDecl(decl) = node {
            if decl.is_default && ctx.vars.contains_key(&decl.name) {
                continue;
            }
            let val = crepuscularity_core::eval::eval_expr(&decl.expr, &ctx);
            ctx.vars.insert(decl.name.clone(), val);
        }
    }

    let mut html = String::new();
    let mut is_first = is_root;

    for node in nodes {
        if let Node::LetDecl(_) = node {
            continue;
        }
        if let Node::Element(el) = node {
            let dyn_id = if node_is_dynamic(node) {
                Some(counter.fetch_add(1, Ordering::Relaxed))
            } else {
                None
            };
            html.push_str(&render_element_with_hydration(
                el, &ctx, counter, is_first, dyn_id,
            )?);
            is_first = false;
        } else {
            html.push_str(&render_node(node, &ctx)?);
            is_first = false;
        }
    }

    Ok(html)
}

#[cfg(feature = "hydration")]
fn render_element_with_hydration(
    el: &crepuscularity_core::ast::Element,
    ctx: &TemplateContext,
    counter: &std::sync::atomic::AtomicU32,
    is_root: bool,
    dyn_id: Option<u32>,
) -> Result<String, String> {
    if el.tag == "slot" {
        return if let Some((slot_nodes, slot_ctx)) = &ctx.slot {
            render_nodes_to_html(slot_nodes, slot_ctx)
        } else {
            render_nodes_to_html(&el.children, ctx)
        };
    }

    let mut class_names = el.classes.clone();
    for cc in &el.conditional_classes {
        if ctx.eval_condition(&cc.condition) {
            class_names.push(cc.class.clone());
        }
    }

    let mut out = String::new();
    out.push('<');
    out.push_str(&el.tag);

    if is_root {
        out.push_str(" data-crepus-root");
    }
    if let Some(id) = dyn_id {
        out.push_str(&format!(" data-crepus-id=\"{id}\""));
    }

    if !class_names.is_empty() {
        out.push_str(" class=\"");
        out.push_str(&escape_html(&ctx.interpolate(&class_names.join(" "))));
        out.push('"');
    }

    for binding in &el.bindings {
        out.push(' ');
        out.push_str(&binding.prop);
        out.push_str("=\"");
        let value = crepuscularity_core::context::value_to_str(
            &crepuscularity_core::eval::eval_expr(&binding.value, ctx),
        );
        out.push_str(&escape_html(&value));
        out.push('"');
    }

    for handler in &el.event_handlers {
        out.push(' ');
        out.push_str("data-on");
        out.push_str(&handler.event);
        out.push_str("=\"");
        out.push_str(&escape_html(&handler.handler));
        out.push('"');
    }

    for animation in &el.animations {
        out.push(' ');
        out.push_str("data-animate-");
        out.push_str(&animation.property);
        out.push_str("=\"");
        out.push_str(&escape_html(&format!(
            "{} {}",
            animation.duration_expr, animation.easing
        )));
        out.push('"');
    }

    out.push('>');

    out.push_str(&render_nodes_with_hydration_impl(
        &el.children,
        ctx,
        counter,
        false,
    )?);

    out.push_str("</");
    out.push_str(&el.tag);
    out.push('>');
    Ok(out)
}

#[cfg(feature = "hydration")]
fn serialize_ctx_to_json(ctx: &TemplateContext) -> String {
    use serde_json::{Map, Value};
    let mut map = Map::new();
    for (key, val) in &ctx.vars {
        let json_val = match val {
            TemplateValue::Str(s) => Value::String(s.clone()),
            TemplateValue::Int(n) => Value::Number((*n).into()),
            TemplateValue::Float(f) => serde_json::Number::from_f64(*f)
                .map(Value::Number)
                .unwrap_or(Value::Null),
            TemplateValue::Bool(b) => Value::Bool(*b),
            TemplateValue::Null => Value::Null,
            TemplateValue::List(items) => Value::Array(
                items
                    .iter()
                    .map(|item_ctx| {
                        let mut item_map = Map::new();
                        for (k, v) in &item_ctx.vars {
                            item_map.insert(
                                k.clone(),
                                match v {
                                    TemplateValue::Str(s) => Value::String(s.clone()),
                                    TemplateValue::Int(n) => Value::Number((*n).into()),
                                    TemplateValue::Float(f) => serde_json::Number::from_f64(*f)
                                        .map(Value::Number)
                                        .unwrap_or(Value::Null),
                                    TemplateValue::Bool(b) => Value::Bool(*b),
                                    _ => Value::Null,
                                },
                            );
                        }
                        Value::Object(item_map)
                    })
                    .collect(),
            ),
        };
        map.insert(key.clone(), json_val);
    }
    serde_json::to_string(&Value::Object(map)).unwrap_or_else(|_| "{}".to_string())
}

fn escape_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
