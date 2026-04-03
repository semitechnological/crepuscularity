use std::path::{Path, PathBuf};

use crepuscularity_core::ast::*;
use crepuscularity_core::context::{value_to_str, TemplateContext, TemplateValue};
use crepuscularity_core::eval::eval_expr;
use crepuscularity_core::parser::{parse_component_file, parse_template};

/// Render an entry point from an in-memory file map — no filesystem access.
///
/// `entry` is `"path/to/file.crepus"` or `"path/to/file.crepus#ComponentName"`.
/// `files` maps paths to `.crepus` source strings.
/// All `include` directives within the templates are resolved from `files`.
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
    std::fs::read_to_string(path).map_err(|e| format!("include error: {:?}: {}", path, e))
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
    std::fs::canonicalize(&candidate).unwrap_or(candidate)
}

fn escape_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

// ── JSX / TSX output ──────────────────────────────────────────────────────────
//
// Same DSL, different output target. Use these when you want to embed .crepus
// templates in a React / TSX codebase. No separate crate required.

pub fn render_template_to_jsx(template: &str, ctx: &TemplateContext) -> Result<String, String> {
    let nodes = parse_template(template)?;
    render_nodes_to_jsx(&nodes, ctx)
}

pub fn render_component_file_to_jsx(
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

    render_nodes_to_jsx(&component.nodes, &child_ctx)
}

pub fn render_nodes_to_jsx(nodes: &[Node], ctx: &TemplateContext) -> Result<String, String> {
    jsx_render_nodes_with_ctx(nodes, ctx.clone())
}

fn jsx_render_nodes_with_ctx(nodes: &[Node], mut ctx: TemplateContext) -> Result<String, String> {
    let mut jsx = String::new();

    for node in nodes {
        if let Node::LetDecl(decl) = node {
            if decl.is_default && ctx.vars.contains_key(&decl.name) {
                continue;
            }
            let val = eval_expr(&decl.expr, &ctx);
            ctx.vars.insert(decl.name.clone(), val);
            continue;
        }
        jsx.push_str(&jsx_render_node(node, &ctx)?);
    }

    Ok(jsx)
}

fn jsx_render_node(node: &Node, ctx: &TemplateContext) -> Result<String, String> {
    match node {
        Node::Element(el) => jsx_render_element(el, ctx),
        Node::Text(parts) => Ok(escape_jsx_text(&render_text(parts, ctx))),
        Node::If(block) => jsx_render_if(block, ctx),
        Node::For(block) => jsx_render_for(block, ctx),
        Node::Match(block) => jsx_render_match(block, ctx),
        Node::LetDecl(_) => Ok(String::new()),
        Node::Include(inc) => jsx_render_include(inc, ctx),
        Node::RawText(expr) => Ok(escape_jsx_text(&value_to_str(&eval_expr(expr, ctx)))),
    }
}

fn jsx_render_element(el: &Element, ctx: &TemplateContext) -> Result<String, String> {
    if el.tag == "slot" {
        return if let Some((slot_nodes, slot_ctx)) = &ctx.slot {
            render_nodes_to_jsx(slot_nodes, slot_ctx)
        } else {
            render_nodes_to_jsx(&el.children, ctx)
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

    if !class_names.is_empty() {
        out.push_str(" className=");
        out.push_str(&jsx_quote_string(&ctx.interpolate(&class_names.join(" "))));
    }

    for binding in &el.bindings {
        out.push(' ');
        out.push_str(&jsx_prop_name(&binding.prop));
        out.push('=');
        let value = value_to_str(&eval_expr(&binding.value, ctx));
        out.push_str(&jsx_quote_string(&value));
    }

    for handler in &el.event_handlers {
        out.push(' ');
        out.push_str(&jsx_event_name(&handler.event));
        out.push_str("={");
        out.push_str(&handler.handler);
        out.push('}');
    }

    for animation in &el.animations {
        out.push(' ');
        out.push_str("data-animate-");
        out.push_str(&animation.property);
        out.push('=');
        out.push_str(&jsx_quote_string(&format!(
            "{} {}",
            animation.duration_expr, animation.easing
        )));
    }

    out.push('>');

    for child in &el.children {
        out.push_str(&jsx_render_node(child, ctx)?);
    }

    out.push_str("</");
    out.push_str(&el.tag);
    out.push('>');
    Ok(out)
}

fn jsx_render_if(block: &IfBlock, ctx: &TemplateContext) -> Result<String, String> {
    if ctx.eval_condition(&block.condition) {
        render_nodes_to_jsx(&block.then_children, ctx)
    } else if let Some(else_children) = &block.else_children {
        render_nodes_to_jsx(else_children, ctx)
    } else {
        Ok(String::new())
    }
}

fn jsx_render_for(block: &ForBlock, ctx: &TemplateContext) -> Result<String, String> {
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

        out.push_str(&render_nodes_to_jsx(&block.body, &child_ctx)?);
    }

    Ok(out)
}

fn jsx_render_match(block: &MatchBlock, ctx: &TemplateContext) -> Result<String, String> {
    let val = eval_expr(&block.expr, ctx);
    let value = value_to_str(&val);

    for arm in &block.arms {
        let pattern = arm.pattern.trim();
        if pattern == "_" {
            return render_nodes_to_jsx(&arm.body, ctx);
        }
        if pattern.starts_with('"') && pattern.ends_with('"') {
            let lit = &pattern[1..pattern.len() - 1];
            if value == lit {
                return render_nodes_to_jsx(&arm.body, ctx);
            }
        }
        if value == pattern {
            return render_nodes_to_jsx(&arm.body, ctx);
        }
    }

    Ok(String::new())
}

fn jsx_render_include(inc: &IncludeNode, ctx: &TemplateContext) -> Result<String, String> {
    if let Some((file_part, comp_name)) = inc.path.split_once('#') {
        return jsx_render_named_component(inc, ctx, file_part, comp_name);
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

    render_nodes_to_jsx(&nodes, &child_ctx)
}

fn jsx_render_named_component(
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

    render_nodes_to_jsx(&comp.nodes, &child_ctx)
}

fn jsx_prop_name(name: &str) -> String {
    match name {
        "class" => "className".to_string(),
        "for" => "htmlFor".to_string(),
        other => other.to_string(),
    }
}

fn jsx_event_name(event: &str) -> String {
    let mut chars = event.chars();
    match chars.next() {
        Some(first) => format!("on{}{}", first.to_ascii_uppercase(), chars.as_str()),
        None => "onUnknown".to_string(),
    }
}

fn jsx_quote_string(value: &str) -> String {
    format!("{{\"{}\"}}", jsx_escape_string(value))
}

fn jsx_escape_string(input: &str) -> String {
    input
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

fn escape_jsx_text(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('{', "&#123;")
        .replace('}', "&#125;")
}
