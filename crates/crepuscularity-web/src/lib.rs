//! HTML and WASM rendering backend for `.crepus` templates.
//!
//! Use [`render_from_files`] for virtual-file builds, [`render_template_to_html`] for one-off
//! templates, and [`render_bundle`] for the `crepus web build` JSON bundle format.

use std::path::Path;
use std::sync::Arc;

use crepuscularity_core::ast::*;
use crepuscularity_core::context::{value_to_str, TemplateContext, TemplateValue};
use crepuscularity_core::eval::eval_expr;
pub use crepuscularity_core::include_paths::resolve_include_path;
use crepuscularity_core::parser::parse_component_file;
use crepuscularity_core::preprocess::{slot_rotate_child_phrases, slot_rotate_words_json_attr};
use crepuscularity_core::virtual_files::lookup_virtual_file;

mod bundle;
#[cfg(all(target_arch = "wasm32", feature = "dom"))]
pub mod dom;
mod void_html;

/// Reactive DOM bindings re-exported from `crepuscularity-reactive`.
/// Enable the `reactive` feature to use signals, memos, and effects in WASM.
#[cfg(all(target_arch = "wasm32", feature = "reactive"))]
pub mod reactive {
    pub use crepuscularity_reactive::dom::{bind_attr, bind_class, bind_text};
    pub use crepuscularity_reactive::{batch_begin, batch_end, flush, Effect, Memo, Signal};
}

pub use bundle::{parse_bundle, render_bundle, render_bundle_with_context, Bundle};
pub use crepuscularity_core::build;
pub use crepuscularity_core::preprocess::google_fonts_head_markup;
pub use crepuscularity_core::CrepusError;
pub use crepuscularity_macros::crepus_refs;

/// Maximum include nesting depth to prevent infinite recursion / stack overflow.
const MAX_INCLUDE_DEPTH: usize = 64;

#[cfg(feature = "ssr")]
mod ssr;

#[cfg(feature = "ssr")]
pub use ssr::{
    render_bundle_with_ssr, render_from_files_with_ssr, render_nodes_ssr, render_ssr_document,
    render_ssr_document_with_nodes, render_template_to_html_with_ssr, serialize_ctx_for_ssr,
    wrap_ssr_document, BindMap, SsrDocument,
};

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
) -> Result<String, CrepusError> {
    let mut ctx = ctx.clone();
    ctx.virtual_files = std::sync::Arc::new(files.clone());

    if let Some((file_part, comp_name)) = entry.split_once('#') {
        let content = files.get(file_part).ok_or_else(|| {
            CrepusError::render(format!("file not found in virtual fs: {file_part}"))
        })?;
        return render_component_file_to_html(content, comp_name, &ctx);
    }

    let content = files
        .get(entry)
        .ok_or_else(|| CrepusError::render(format!("file not found in virtual fs: {entry}")))?;
    render_template_to_html(content, &ctx)
}

/// Render multiple entry points from a virtual file map in parallel (requires `parallel` feature).
///
/// Returns a `Vec` of `(entry, Result<String, CrepusError>)` in the same order as `entries`.
/// Each entry is independent — rendering runs on a Rayon thread pool.
/// Falls back to sequential iteration when the `parallel` feature is disabled (e.g. WASM).
pub fn par_render_from_files(
    files: &std::collections::HashMap<String, String>,
    entries: &[&str],
    ctx: &TemplateContext,
) -> Vec<(String, Result<String, CrepusError>)> {
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
) -> Result<std::collections::HashMap<String, Result<String, CrepusError>>, CrepusError> {
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
                    if !child_ctx.vars.contains_key(key) {
                        if let Ok(val) = eval_expr(expr, &TemplateContext::new()) {
                            child_ctx.vars.insert(key.clone(), val);
                        }
                    }
                }
                let html = render_nodes_to_html(&comp.nodes, &child_ctx);
                (name.clone(), html)
            })
            .collect();
        Ok(results)
    }
    #[cfg(not(feature = "parallel"))]
    {
        let mut results = std::collections::HashMap::new();
        for (name, comp) in &file.components {
            let mut child_ctx = ctx.clone();
            for (key, expr) in &comp.meta.defaults {
                if !child_ctx.vars.contains_key(key) {
                    child_ctx
                        .vars
                        .insert(key.clone(), eval_expr(expr, &TemplateContext::new())?);
                }
            }
            let html = render_nodes_to_html(&comp.nodes, &child_ctx);
            results.insert(name.clone(), html);
        }
        Ok(results)
    }
}

/// Parse a single `.crepus` template string and render it to escaped HTML.
///
/// This entry point does not resolve filesystem includes. For component trees that use
/// `include`, prefer [`render_from_files`] so every source file is supplied through the same
/// virtual file map used by WASM and static-site builds.
#[tracing::instrument(skip(template, ctx), fields(template_len = template.len()))]
pub fn render_template_to_html(
    template: &str,
    ctx: &TemplateContext,
) -> Result<String, CrepusError> {
    crepuscularity_core::render_entry::render_template_with(
        template,
        ctx,
        |t| Ok(crepuscularity_core::ast_cache::parse_content(t)?.to_vec()),
        render_nodes_to_html,
    )
}

/// Render one named component from a multi-component `.crepus` source string.
///
/// `component_name` is the section name after a `--- Name` separator. Component defaults are
/// evaluated before rendering and do not override variables already present in `ctx`.
pub fn render_component_file_to_html(
    content: &str,
    component_name: &str,
    ctx: &TemplateContext,
) -> Result<String, CrepusError> {
    crepuscularity_core::render_entry::render_component_file(
        content,
        component_name,
        ctx,
        render_nodes_to_html,
    )
}

/// Render an already parsed AST node list to escaped HTML.
///
/// Use this when a caller owns parsing or analysis separately from rendering. `$: let` declarations
/// are applied in source order via an optional context overlay, so sibling nodes after a declaration can see it.
pub fn render_nodes_to_html(nodes: &[Node], ctx: &TemplateContext) -> Result<String, CrepusError> {
    render_nodes_with_ctx(nodes, None, ctx, 0)
}

fn render_nodes_with_ctx(
    nodes: &[Node],
    mut overlay: Option<TemplateContext>,
    base: &TemplateContext,
    depth: usize,
) -> Result<String, CrepusError> {
    let _span = tracing::debug_span!("render_html", node_count = nodes.len()).entered();
    let mut html = String::new();

    for node in nodes {
        if let Node::LetDecl(decl) = node {
            let cur = overlay.as_ref().unwrap_or(base);
            if decl.is_default && cur.vars.contains_key(&decl.name) {
                continue;
            }
            let overlay_ctx = overlay.get_or_insert_with(|| base.clone());
            let val = eval_expr(&decl.expr, overlay_ctx)?;
            overlay_ctx.vars.insert(decl.name.clone(), val);
            continue;
        }
        let cur = overlay.as_ref().unwrap_or(base);
        html.push_str(&render_node(node, cur, depth)?);
    }

    Ok(html)
}

fn render_node(node: &Node, ctx: &TemplateContext, depth: usize) -> Result<String, CrepusError> {
    match node {
        Node::Element(el) => render_element(el, ctx, depth),
        Node::Text(parts) => Ok(escape_html(&render_text(parts, ctx)?)),
        Node::If(block) => render_if(block, ctx, depth),
        Node::For(block) => render_for(block, ctx, depth),
        Node::Match(block) => render_match(block, ctx, depth),
        Node::LetDecl(_) => Ok(String::new()),
        Node::Include(inc) => render_include(inc, ctx, depth),
        Node::Embed(embed) => render_embed(embed, ctx),
        Node::RawText(expr) => Ok(escape_html(&value_to_str(&eval_expr(expr, ctx)?))),
        Node::RawHtml(expr) => {
            let inner = value_to_str(&eval_expr(expr, ctx)?);
            Ok(ammonia::clean(&inner))
        }
    }
}

fn render_embed(embed: &EmbedNode, ctx: &TemplateContext) -> Result<String, CrepusError> {
    let mut props = serde_json::Map::new();
    for (key, expr) in &embed.props {
        props.insert(key.clone(), template_value_to_json(&eval_expr(expr, ctx)?));
    }
    let props_json = serde_json::Value::Object(props).to_string();
    let adapter = embed.adapter.as_deref().unwrap_or("module");
    let scope_attr = if adapter == "rust" {
        format!(" data-crepus-scope=\"{}\"", escape_html_attr(&embed.src))
    } else {
        String::new()
    };
    Ok(format!(
        "<div data-crepus-island=\"\" data-crepus-island-src=\"{}\" data-crepus-island-adapter=\"{}\"{scope_attr} data-crepus-island-props=\"{}\"></div>",
        escape_html_attr(&embed.src),
        escape_html_attr(adapter),
        escape_html_attr(&props_json)
    ))
}

fn template_value_to_json(value: &TemplateValue) -> serde_json::Value {
    match value {
        TemplateValue::Str(s) => serde_json::Value::String(s.clone()),
        TemplateValue::Int(n) => serde_json::Value::Number((*n).into()),
        TemplateValue::Float(f) => serde_json::Number::from_f64(*f)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null),
        TemplateValue::Bool(b) => serde_json::Value::Bool(*b),
        TemplateValue::List(items) => serde_json::Value::Array(
            items
                .iter()
                .map(|item| {
                    let mut object = serde_json::Map::new();
                    for (key, value) in &item.vars {
                        object.insert(key.clone(), template_value_to_json(value));
                    }
                    serde_json::Value::Object(object)
                })
                .collect(),
        ),
        TemplateValue::Scope(ctx) => {
            let mut object = serde_json::Map::new();
            for (key, value) in &ctx.vars {
                object.insert(key.clone(), template_value_to_json(value));
            }
            serde_json::Value::Object(object)
        }
        TemplateValue::Null => serde_json::Value::Null,
    }
}

fn render_element(
    el: &Element,
    ctx: &TemplateContext,
    depth: usize,
) -> Result<String, CrepusError> {
    if el.tag == "slot" {
        return if let Some((slot_nodes, slot_ctx)) = &ctx.slot {
            render_nodes_with_ctx(slot_nodes, None, slot_ctx, depth)
        } else {
            render_nodes_with_ctx(&el.children, None, ctx, depth)
        };
    }

    if el.tag == "slot-rotate" {
        let phrases = slot_rotate_child_phrases(&el.children).map_err(CrepusError::render)?;
        if phrases.len() < 2 {
            return Err(CrepusError::render(
                "slot-rotate needs at least two plain-text phrase children",
            ));
        }
        let mut interval_ms = 3200u64;
        for b in &el.bindings {
            if b.prop == "interval" {
                let v = value_to_str(&eval_expr(&b.value, ctx)?);
                let v = v.trim_matches('"').trim();
                interval_ms = v.parse().unwrap_or(3200);
            }
        }
        let words_json = slot_rotate_words_json_attr(&phrases);

        // Build class list with the "crepus-slot" prefix prepended.
        let mut class_names: Vec<String> = Vec::with_capacity(el.classes.len() + 1);
        class_names.push("crepus-slot".to_string());
        class_names.extend(el.classes.iter().cloned());

        let mut out = String::new();
        out.push_str("<span");
        if let Some(id) = &el.id {
            out.push_str(" id=\"");
            out.push_str(&escape_html(id));
            out.push('"');
        }
        if let Some(class_attr) = build_class_attr(&class_names, &el.conditional_classes, ctx)? {
            out.push_str(" class=\"");
            out.push_str(&class_attr);
            out.push('"');
        }
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
            let value = value_to_str(&eval_expr(&binding.value, ctx)?);
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

    let mut out = String::new();
    out.push('<');
    out.push_str(&el.tag);

    if let Some(id) = &el.id {
        out.push_str(" id=\"");
        out.push_str(&escape_html(id));
        out.push('"');
    }

    if let Some(class_attr) = build_class_attr(&el.classes, &el.conditional_classes, ctx)? {
        out.push_str(" class=\"");
        out.push_str(&class_attr);
        out.push('"');
    }

    for binding in &el.bindings {
        out.push(' ');
        out.push_str(&binding.prop);
        out.push_str("=\"");
        let value = value_to_str(&eval_expr(&binding.value, ctx)?);
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

    if void_html::is_void_html_tag(&el.tag) {
        out.push_str(" />");
        return Ok(out);
    }

    out.push('>');

    for child in &el.children {
        out.push_str(&render_node(child, ctx, depth)?);
    }

    out.push_str("</");
    out.push_str(&el.tag);
    out.push('>');
    Ok(out)
}

pub(crate) fn render_text(
    parts: &[TextPart],
    ctx: &TemplateContext,
) -> Result<String, CrepusError> {
    let mut result = String::new();
    for part in parts {
        match part {
            TextPart::Literal(text) => result.push_str(text),
            TextPart::Expr(expr) => {
                result.push_str(&value_to_str(&eval_expr(expr, ctx)?));
            }
        }
    }
    Ok(result)
}

fn render_if(block: &IfBlock, ctx: &TemplateContext, depth: usize) -> Result<String, CrepusError> {
    if ctx.eval_condition(&block.condition)? {
        render_nodes_with_ctx(&block.then_children, None, ctx, depth)
    } else if let Some(else_children) = &block.else_children {
        render_nodes_with_ctx(else_children, None, ctx, depth)
    } else {
        Ok(String::new())
    }
}

fn render_for(
    block: &ForBlock,
    ctx: &TemplateContext,
    depth: usize,
) -> Result<String, CrepusError> {
    let items = ctx.get_list_ref(&block.iterator);
    let mut out = String::new();
    let pattern = block.pattern.trim();
    let has_pattern = !pattern.is_empty();
    let mut child_ctx = ctx.clone();

    for item_ctx in items {
        child_ctx.vars.clone_from(&ctx.vars);
        for (k, v) in &item_ctx.vars {
            child_ctx.vars.insert(k.clone(), v.clone());
        }
        if has_pattern {
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
        out.push_str(&render_nodes_with_ctx(
            &block.body,
            None,
            &child_ctx,
            depth,
        )?);
    }

    Ok(out)
}

fn render_match(
    block: &MatchBlock,
    ctx: &TemplateContext,
    depth: usize,
) -> Result<String, CrepusError> {
    let val = eval_expr(&block.expr, ctx)?;
    let value = value_to_str(&val);

    for arm in &block.arms {
        let pattern = arm.pattern.trim();
        if pattern == "_" {
            return render_nodes_with_ctx(&arm.body, None, ctx, depth);
        }
        if pattern.starts_with('"') && pattern.ends_with('"') {
            let lit = &pattern[1..pattern.len() - 1];
            if value == lit {
                return render_nodes_with_ctx(&arm.body, None, ctx, depth);
            }
        }
        if value == pattern {
            return render_nodes_with_ctx(&arm.body, None, ctx, depth);
        }
    }

    Ok(String::new())
}

pub(crate) fn read_file(ctx: &TemplateContext, path: &Path) -> Result<String, CrepusError> {
    if let Some(content) = lookup_virtual_file(ctx, path) {
        return Ok(content);
    }
    if cfg!(not(target_arch = "wasm32")) {
        std::fs::read_to_string(path)
            .map_err(|e| CrepusError::render(format!("include error: {:?}: {}", path, e)))
    } else {
        Err(CrepusError::render(format!(
            "include error: file not in virtual bundle: {}",
            path.to_string_lossy()
        )))
    }
}

fn render_include(
    inc: &IncludeNode,
    ctx: &TemplateContext,
    depth: usize,
) -> Result<String, CrepusError> {
    if depth >= MAX_INCLUDE_DEPTH {
        return Err(CrepusError::render(format!(
            "maximum include depth ({MAX_INCLUDE_DEPTH}) exceeded; possible circular include involving '{}'",
            inc.path
        )));
    }

    if let Some((file_part, comp_name)) = inc.path.split_once('#') {
        return render_named_component(inc, ctx, file_part, comp_name, depth);
    }

    let file_path = resolve_include_path(ctx.base_dir.as_deref(), &inc.path)?;
    let content = read_file(ctx, &file_path)?;
    let nodes = crepuscularity_core::ast_cache::parse_content(&content)
        .map_err(|e| CrepusError::render(format!("include parse error: {e}")))?;

    let mut child_ctx = TemplateContext::new();
    child_ctx.base_dir = file_path.parent().map(|p| p.to_path_buf());
    child_ctx.virtual_files = ctx.virtual_files.clone();
    for (key, expr) in &inc.props {
        child_ctx.vars.insert(key.clone(), eval_expr(expr, ctx)?);
    }
    if !inc.slot.is_empty() {
        child_ctx.slot = Some((inc.slot.clone(), Arc::new(ctx.clone())));
    }

    render_nodes_with_ctx(&nodes, None, &child_ctx, depth + 1)
}

fn render_named_component(
    inc: &IncludeNode,
    ctx: &TemplateContext,
    file_part: &str,
    comp_name: &str,
    depth: usize,
) -> Result<String, CrepusError> {
    if depth >= MAX_INCLUDE_DEPTH {
        return Err(CrepusError::render(format!(
            "maximum include depth ({MAX_INCLUDE_DEPTH}) exceeded; possible circular include involving '{file_part}#{comp_name}'"
        )));
    }

    let file_path = resolve_include_path(ctx.base_dir.as_deref(), file_part)?;
    let content = read_file(ctx, &file_path)?;
    let comp_file = parse_component_file(&content)
        .map_err(|e| CrepusError::render(format!("component file parse error: {e}")))?;
    let comp = comp_file.components.get(comp_name).ok_or_else(|| {
        CrepusError::render(format!(
            "component '{}' not found in {}",
            comp_name, file_part
        ))
    })?;

    let mut child_ctx = TemplateContext::new();
    child_ctx.base_dir = file_path.parent().map(|p| p.to_path_buf());
    child_ctx.virtual_files = ctx.virtual_files.clone();
    for (key, expr) in &comp.meta.defaults {
        child_ctx
            .vars
            .insert(key.clone(), eval_expr(expr, &TemplateContext::new())?);
    }
    for (key, expr) in &inc.props {
        child_ctx.vars.insert(key.clone(), eval_expr(expr, ctx)?);
    }
    if !inc.slot.is_empty() {
        child_ctx.slot = Some((inc.slot.clone(), Arc::new(ctx.clone())));
    }

    render_nodes_with_ctx(&comp.nodes, None, &child_ctx, depth + 1)
}

// ── Hydration ─────────────────────────────────────────────────────────────────

/// Returns `true` if a node is "dynamic" — contains interpolated expressions,
/// control flow, or other content that can change based on context.
#[cfg(feature = "hydration")]
fn node_is_dynamic(node: &Node) -> bool {
    match node {
        Node::If(_)
        | Node::For(_)
        | Node::Match(_)
        | Node::RawText(_)
        | Node::RawHtml(_)
        | Node::Embed(_) => true,
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
/// - A non-executable JSON `<script>` is appended with the serialized context
///   variables encoded as base64.
///
/// Enable with the `hydration` cargo feature.
#[cfg(feature = "hydration")]
pub fn render_template_to_html_with_hydration(
    template: &str,
    ctx: &TemplateContext,
) -> Result<String, CrepusError> {
    use std::sync::atomic::AtomicU32;

    let nodes = crepuscularity_core::ast_cache::parse_content(template)?;

    // Counter for data-crepus-id assignment.
    let counter = AtomicU32::new(0);

    let rendered = render_nodes_with_hydration_impl(&nodes, ctx, &counter, /*is_root=*/ true)?;

    // Serialize context vars to JSON.
    use base64::{engine::general_purpose::STANDARD, Engine as _};

    let raw = hydration_payload_bytes(serialize_ctx_to_value(ctx), serde_json::json!({}))?;
    let ctx_b64 = STANDARD.encode(raw);
    let script = format!(
        r#"<script id="__crepus_hydration__" type="application/json" data-crepus-encoding="base64">{ctx_b64}</script>"#
    );

    Ok(format!("{rendered}{script}"))
}

#[cfg(feature = "hydration")]
fn render_nodes_with_hydration_impl(
    nodes: &[Node],
    ctx: &TemplateContext,
    counter: &std::sync::atomic::AtomicU32,
    is_root: bool,
) -> Result<String, CrepusError> {
    use std::sync::atomic::Ordering;

    // Process LetDecl nodes first.
    let mut overlay = Some(ctx.clone());
    for node in nodes {
        if let Node::LetDecl(decl) = node {
            let overlay_ctx = overlay.as_mut().expect("overlay for hydration");
            if decl.is_default && overlay_ctx.vars.contains_key(&decl.name) {
                continue;
            }
            let val = crepuscularity_core::eval::eval_expr(&decl.expr, overlay_ctx)?;
            overlay_ctx.vars.insert(decl.name.clone(), val);
        }
    }
    let ctx = overlay.as_ref().expect("overlay for hydration");

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
            html.push_str(&render_node(node, &ctx, 0)?);
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
) -> Result<String, CrepusError> {
    if el.tag == "slot" {
        return if let Some((slot_nodes, slot_ctx)) = &ctx.slot {
            render_nodes_to_html(slot_nodes, slot_ctx)
        } else {
            render_nodes_to_html(&el.children, ctx)
        };
    }

    let mut class_names = el.classes.clone();
    for cc in &el.conditional_classes {
        if ctx.eval_condition(&cc.condition)? {
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
        out.push_str(&escape_html(&ctx.interpolate(&class_names.join(" "))?));
        out.push('"');
    }

    for binding in &el.bindings {
        out.push(' ');
        out.push_str(&binding.prop);
        out.push_str("=\"");
        let value = crepuscularity_core::context::value_to_str(
            &crepuscularity_core::eval::eval_expr(&binding.value, ctx)?,
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

    if void_html::is_void_html_tag(&el.tag) {
        out.push_str(" />");
        return Ok(out);
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

#[cfg(any(feature = "hydration", feature = "ssr"))]
pub(crate) fn hydration_payload_bytes(
    ctx: serde_json::Value,
    bind: serde_json::Value,
) -> Result<Vec<u8>, CrepusError> {
    let payload = serde_json::json!({
        "v": 1,
        "ctx": ctx,
        "bind": bind,
    });
    serde_json::to_vec(&payload).map_err(|e| CrepusError::render(e.to_string()))
}

#[cfg(feature = "hydration")]
fn serialize_ctx_to_value(ctx: &TemplateContext) -> serde_json::Value {
    use serde_json::{Map, Value};
    let mut map = Map::new();
    for (key, val) in &ctx.vars {
        map.insert(key.clone(), template_value_to_json(val));
    }
    Value::Object(map)
}

pub(crate) fn escape_html(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            c => out.push(c),
        }
    }
    out
}

pub(crate) fn escape_html_attr(s: &str) -> String {
    escape_html(s)
}

/// Build the `class="..."` attribute value: join classes with spaces, interpolate,
/// and escape — in a single pass, avoiding the intermediate Vec + join + escape.
pub(crate) fn build_class_attr(
    base_classes: &[String],
    conditional: &[ConditionalClass],
    ctx: &TemplateContext,
) -> Result<Option<String>, CrepusError> {
    // First pass: interpolate each class and collect into one space-joined string.
    let mut joined = String::new();
    let mut first = true;
    for class in base_classes {
        if !first {
            joined.push(' ');
        }
        joined.push_str(&ctx.interpolate(class)?);
        first = false;
    }
    for cc in conditional {
        if ctx.eval_condition(&cc.condition)? {
            if !first {
                joined.push(' ');
            }
            joined.push_str(&ctx.interpolate(&cc.class)?);
            first = false;
        }
    }
    if first {
        return Ok(None);
    }
    Ok(Some(escape_html(&joined)))
}
