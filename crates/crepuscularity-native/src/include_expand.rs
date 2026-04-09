//! Expand `include` directives into parsed child nodes + context (no cycles with IR lowering).

use std::path::{Path, PathBuf};

use crepuscularity_core::ast::{IncludeNode, Node};
use crepuscularity_core::context::TemplateContext;
use crepuscularity_core::eval::eval_expr;
use crepuscularity_core::parser::{parse_component_file, parse_template};

pub(crate) fn read_file(ctx: &TemplateContext, path: &Path) -> Result<String, String> {
    let key = path.to_string_lossy();
    if let Some(content) = ctx.virtual_files.get(key.as_ref()) {
        return Ok(content.clone());
    }
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

pub(crate) fn resolve_include_path(base_dir: Option<&Path>, path: &str) -> PathBuf {
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

/// Parsed nodes to render and the child context to use (base_dir, virtual_files, vars, slot).
pub(crate) fn expand_include(
    inc: &IncludeNode,
    ctx: &TemplateContext,
) -> Result<(Vec<Node>, TemplateContext), String> {
    if let Some((file_part, comp_name)) = inc.path.split_once('#') {
        return expand_named_component(inc, ctx, file_part, comp_name);
    }

    let file_path = resolve_include_path(ctx.base_dir.as_deref(), &inc.path);
    let content = read_file(ctx, &file_path)?;
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

    Ok((nodes, child_ctx))
}

fn expand_named_component(
    inc: &IncludeNode,
    ctx: &TemplateContext,
    file_part: &str,
    comp_name: &str,
) -> Result<(Vec<Node>, TemplateContext), String> {
    let file_path = resolve_include_path(ctx.base_dir.as_deref(), file_part);
    let content = read_file(ctx, &file_path)?;
    let comp_file =
        parse_component_file(&content).map_err(|e| format!("component file parse error: {e}"))?;
    let comp = comp_file
        .components
        .get(comp_name)
        .ok_or_else(|| format!("component '{comp_name}' not found in {file_part}"))?;

    let mut child_ctx = TemplateContext::new();
    child_ctx.base_dir = file_path.parent().map(|p| p.to_path_buf());
    child_ctx.virtual_files = ctx.virtual_files.clone();
    for (key, expr) in &comp.meta.defaults {
        child_ctx
            .vars
            .entry(key.clone())
            .or_insert_with(|| eval_expr(expr, &TemplateContext::new()));
    }
    for (key, expr) in &inc.props {
        child_ctx.vars.insert(key.clone(), eval_expr(expr, ctx));
    }
    if !inc.slot.is_empty() {
        child_ctx.slot = Some((inc.slot.clone(), Box::new(ctx.clone())));
    }

    Ok((comp.nodes.clone(), child_ctx))
}
