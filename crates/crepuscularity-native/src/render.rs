//! AST + context → [`crate::ir::ViewIr`] lowering.

use std::collections::HashMap;

use crepuscularity_core::ast::*;
use crepuscularity_core::context::{value_to_str, TemplateContext, TemplateValue};
use crepuscularity_core::eval::eval_expr;
use crepuscularity_core::parser::{parse_component_file, parse_template};
use crepuscularity_core::preprocess::slot_rotate_child_phrases;

use crate::include_expand;
use crate::ir::{PickerOption, StackAxis, ViewIr, ViewNode, IR_VERSION};
use crate::style;

/// Render from a virtual file map (`entry` may use `#Component` suffix).
pub fn render_from_files(
    files: &HashMap<String, String>,
    entry: &str,
    ctx: &TemplateContext,
) -> Result<ViewIr, String> {
    let mut ctx = ctx.clone();
    ctx.virtual_files = std::sync::Arc::new(files.clone());

    if let Some((file_part, comp_name)) = entry.split_once('#') {
        let content = files
            .get(file_part)
            .ok_or_else(|| format!("file not found in virtual fs: {file_part}"))?;
        return render_component_file_to_ir(content, comp_name, &ctx);
    }

    let content = files
        .get(entry)
        .ok_or_else(|| format!("file not found in virtual fs: {entry}"))?;
    render_template_to_ir(content, &ctx)
}

/// Parse and lower a template string to IR.
pub fn render_template_to_ir(template: &str, ctx: &TemplateContext) -> Result<ViewIr, String> {
    let nodes = parse_template(template)?;
    render_nodes_to_ir(&nodes, ctx)
}

/// Lower a named component from a multi-component `.crepus` file.
pub fn render_component_file_to_ir(
    content: &str,
    component_name: &str,
    ctx: &TemplateContext,
) -> Result<ViewIr, String> {
    let file = parse_component_file(content)?;
    let component = file
        .components
        .get(component_name)
        .ok_or_else(|| format!("component not found: {component_name}"))?;

    let mut child_ctx = ctx.clone();
    for (key, expr) in &component.meta.defaults {
        if !child_ctx.vars.contains_key(key) {
            let val = eval_expr(expr, &TemplateContext::new());
            child_ctx.vars.insert(key.clone(), val);
        }
    }

    render_nodes_to_ir(&component.nodes, &child_ctx)
}

/// Lower already-parsed nodes into a [`ViewIr`].
pub fn render_nodes_to_ir(nodes: &[Node], ctx: &TemplateContext) -> Result<ViewIr, String> {
    let root = render_nodes_list(nodes, ctx)?;
    Ok(ViewIr {
        version: IR_VERSION,
        root,
    })
}

pub(crate) fn render_nodes_list(
    nodes: &[Node],
    ctx: &TemplateContext,
) -> Result<Vec<ViewNode>, String> {
    let mut ctx = ctx.clone();
    let mut out = Vec::new();
    for node in nodes {
        if let Node::LetDecl(decl) = node {
            if decl.is_default && ctx.vars.contains_key(&decl.name) {
                continue;
            }
            let val = eval_expr(&decl.expr, &ctx);
            ctx.vars.insert(decl.name.clone(), val);
            continue;
        }
        if let Node::Include(inc) = node {
            let (inner_nodes, inner_ctx) = include_expand::expand_include(inc, &ctx)?;
            out.extend(render_nodes_list(&inner_nodes, &inner_ctx)?);
            continue;
        }
        out.push(render_node(node, &ctx)?);
    }
    Ok(out)
}

fn render_node(node: &Node, ctx: &TemplateContext) -> Result<ViewNode, String> {
    match node {
        Node::Element(el) => render_element(el, ctx),
        Node::Text(parts) => Ok(ViewNode::Text {
            content: render_text_inline(parts, ctx)?,
            style: None,
        }),
        Node::If(block) => render_if(block, ctx),
        Node::For(block) => render_for(block, ctx),
        Node::Match(block) => render_match(block, ctx),
        Node::LetDecl(_) => Ok(stack_column_raw(vec![])),
        Node::Include(_) => {
            Err("internal error: include should be expanded in render_nodes_list".into())
        }
        Node::Embed(_) => Ok(ViewNode::Text {
            content: String::new(),
            style: None,
        }),
        Node::RawText(expr) => Ok(ViewNode::Text {
            content: value_to_str(&eval_expr(expr, ctx)),
            style: None,
        }),
        Node::RawHtml(expr) => Ok(ViewNode::Text {
            content: value_to_str(&eval_expr(expr, ctx)),
            style: None,
        }),
    }
}

fn stack_column_raw(children: Vec<ViewNode>) -> ViewNode {
    ViewNode::Stack {
        axis: StackAxis::Column,
        spacing: None,
        align_items: None,
        justify_content: None,
        style: None,
        children,
    }
}

fn render_element(el: &Element, ctx: &TemplateContext) -> Result<ViewNode, String> {
    let tag = el.tag.to_ascii_lowercase();
    let tag = tag.as_str();

    if tag == "slot" {
        return if let Some((slot_nodes, slot_ctx)) = &ctx.slot {
            let children = render_nodes_list(slot_nodes, slot_ctx)?;
            Ok(stack_column_raw(children))
        } else {
            let children = render_nodes_list(&el.children, ctx)?;
            Ok(stack_column_raw(children))
        };
    }

    if tag == "slot-rotate" {
        let phrases = slot_rotate_child_phrases(&el.children)?;
        let mut interval_ms = 3200u64;
        for b in &el.bindings {
            if b.prop == "interval" {
                let v = value_to_str(&eval_expr(&b.value, ctx));
                let v = v.trim_matches('"').trim();
                interval_ms = v.parse().unwrap_or(3200);
            }
        }
        let classes = active_classes(el, ctx);
        let style = style::extract_stack_hints(&classes, Some(ctx)).style;
        return Ok(ViewNode::SlotRotate {
            phrases,
            interval_ms,
            style: style.opt(),
        });
    }

    let classes = active_classes(el, ctx);

    if tag == "button" {
        let label = collect_primary_text(&el.children, ctx)?;
        let on_click = el
            .event_handlers
            .iter()
            .find(|e| e.event == "click")
            .map(|e| e.handler.clone());
        let hints = style::extract_stack_hints(&classes, Some(ctx));
        return Ok(ViewNode::Button {
            label,
            on_click,
            style: hints.style.opt(),
        });
    }

    if tag == "toggle" || tag == "switch" {
        let label = component_label(el, ctx)?;
        let hints = style::extract_stack_hints(&classes, Some(ctx));
        return Ok(ViewNode::Toggle {
            label,
            bind: binding_raw(el, "bind"),
            checked: binding_bool(el, "checked", ctx).unwrap_or(false),
            on_change: event_handler(el, "change"),
            style: hints.style.opt(),
        });
    }

    if tag == "checkbox" {
        let label = component_label(el, ctx)?;
        let hints = style::extract_stack_hints(&classes, Some(ctx));
        return Ok(ViewNode::Checkbox {
            label,
            bind: binding_raw(el, "bind"),
            checked: binding_bool(el, "checked", ctx).unwrap_or(false),
            on_change: event_handler(el, "change"),
            style: hints.style.opt(),
        });
    }

    if tag == "slider" {
        let hints = style::extract_stack_hints(&classes, Some(ctx));
        return Ok(ViewNode::Slider {
            label: optional_component_label(el, ctx)?,
            bind: binding_raw(el, "bind"),
            value: binding_f32(el, "value", ctx).unwrap_or(0.0),
            min: binding_f32(el, "min", ctx).unwrap_or(0.0),
            max: binding_f32(el, "max", ctx).unwrap_or(100.0),
            step: binding_f32(el, "step", ctx),
            style: hints.style.opt(),
        });
    }

    if tag == "progress" {
        let hints = style::extract_stack_hints(&classes, Some(ctx));
        return Ok(ViewNode::Progress {
            label: optional_component_label(el, ctx)?,
            value: binding_f32(el, "value", ctx).unwrap_or(0.0),
            max: binding_f32(el, "max", ctx).unwrap_or(100.0),
            style: hints.style.opt(),
        });
    }

    if tag == "meter" {
        let hints = style::extract_stack_hints(&classes, Some(ctx));
        return Ok(ViewNode::Meter {
            label: optional_component_label(el, ctx)?,
            value: binding_f32(el, "value", ctx).unwrap_or(0.0),
            min: binding_f32(el, "min", ctx).unwrap_or(0.0),
            max: binding_f32(el, "max", ctx).unwrap_or(100.0),
            style: hints.style.opt(),
        });
    }

    if tag == "badge" {
        let hints = style::extract_stack_hints(&classes, Some(ctx));
        return Ok(ViewNode::Badge {
            label: component_label(el, ctx)?,
            tone: binding_string(el, "tone", ctx),
            style: hints.style.opt(),
        });
    }

    if tag == "divider" || tag == "hr" {
        let hints = style::extract_stack_hints(&classes, Some(ctx));
        return Ok(ViewNode::Divider {
            axis: stack_axis(&classes),
            style: hints.style.opt(),
        });
    }

    if tag == "spacer" {
        let hints = style::extract_stack_hints(&classes, Some(ctx));
        return Ok(ViewNode::Spacer {
            size: binding_f32(el, "size", ctx).or_else(|| parse_gap_spacing(&classes)),
            style: hints.style.opt(),
        });
    }

    if tag == "dropzone" {
        let label = collect_primary_text(&el.children, ctx).unwrap_or_default();
        let accept = el
            .bindings
            .iter()
            .find(|b| b.prop == "accept")
            .map(|b| value_to_str(&eval_expr(&b.value, ctx)));
        let on_drop = el
            .event_handlers
            .iter()
            .find(|e| e.event == "drop")
            .map(|e| e.handler.clone());
        let hints = style::extract_stack_hints(&classes, Some(ctx));
        let children = render_nodes_list(&el.children, ctx)?;
        return Ok(ViewNode::Dropzone {
            label,
            accept,
            on_drop,
            style: hints.style.opt(),
            children,
        });
    }

    if tag == "input" || tag == "textfield" || tag == "textinput" || tag == "textarea" {
        let bind = el
            .bindings
            .iter()
            .find(|b| b.prop == "bind")
            .map(|b| b.value.clone())
            .unwrap_or_default();
        let placeholder = el
            .bindings
            .iter()
            .find(|b| b.prop == "placeholder")
            .map(|b| {
                let v = b.value.trim();
                v.trim_matches(|c| c == '"' || c == '\'').to_string()
            })
            .unwrap_or_default();
        let multiline = classes.iter().any(|c| c == "multiline") || tag == "textarea";
        let hints = style::extract_stack_hints(&classes, Some(ctx));
        return Ok(ViewNode::Input {
            placeholder,
            bind,
            multiline,
            style: hints.style.opt(),
        });
    }

    if tag == "picker" || tag == "select" {
        let bind = el
            .bindings
            .iter()
            .find(|b| b.prop == "bind")
            .map(|b| b.value.clone())
            .unwrap_or_default();
        let mut options = Vec::new();
        for child in &el.children {
            if let Node::Element(inner) = child {
                let inner_tag = inner.tag.to_ascii_lowercase();
                if matches!(inner_tag.as_str(), "span" | "button" | "option") {
                    let label = collect_primary_text(&inner.children, ctx)?;
                    let value = inner
                        .bindings
                        .iter()
                        .find(|b| b.prop == "value")
                        .map(|b| {
                            let v = b.value.trim();
                            v.trim_matches(|c| c == '"' || c == '\'').to_string()
                        })
                        .filter(|s| !s.is_empty())
                        .unwrap_or_else(|| slug_option_value(&label));
                    options.push(PickerOption { value, label });
                }
            }
        }
        let hints = style::extract_stack_hints(&classes, Some(ctx));
        return Ok(ViewNode::Picker {
            bind,
            options,
            style: hints.style.opt(),
        });
    }

    if tag == "img" || tag == "image" {
        let src = el
            .bindings
            .iter()
            .find(|b| b.prop == "src")
            .map(|b| value_to_str(&eval_expr(&b.value, ctx)))
            .unwrap_or_default();
        let alt = el
            .bindings
            .iter()
            .find(|b| b.prop == "alt")
            .map(|b| value_to_str(&eval_expr(&b.value, ctx)));
        let placeholder = el
            .bindings
            .iter()
            .find(|b| b.prop == "placeholder")
            .map(|b| value_to_str(&eval_expr(&b.value, ctx)));
        let hints = style::extract_stack_hints(&classes, Some(ctx));
        return Ok(ViewNode::Image {
            src,
            alt,
            placeholder,
            style: hints.style.opt(),
        });
    }

    if tag == "ul" || tag == "ol" || tag == "list" || tag == "flatlist" {
        let hints = style::extract_stack_hints(&classes, Some(ctx));
        return Ok(ViewNode::List {
            ordered: tag == "ol",
            style: hints.style.opt(),
            children: render_nodes_list(&el.children, ctx)?,
        });
    }

    if tag == "li" || tag == "list-item" {
        let hints = style::extract_stack_hints(&classes, Some(ctx));
        return Ok(ViewNode::ListItem {
            style: hints.style.opt(),
            children: render_nodes_list(&el.children, ctx)?,
        });
    }

    if is_text_tag(tag) && el.children.len() == 1 {
        if let Node::Text(parts) = &el.children[0] {
            let txt = render_text_inline(parts, ctx)?;
            let st = style::extract_text_style(&classes, Some(ctx)).opt();
            return Ok(ViewNode::Text {
                content: txt,
                style: st,
            });
        }
    }

    let axis = if tag == "view" {
        StackAxis::Column
    } else {
        stack_axis(&classes)
    };
    let spacing = parse_gap_spacing(&classes);
    let scroll = style::is_scroll_container(&classes);
    let hints = style::extract_stack_hints(&classes, Some(ctx));
    let children = render_nodes_list(&el.children, ctx)?;

    if scroll {
        return Ok(ViewNode::Scroll {
            axis,
            style: hints.style.opt(),
            children,
        });
    }

    Ok(ViewNode::Stack {
        axis,
        spacing,
        align_items: hints.align_items,
        justify_content: hints.justify_content,
        style: hints.style.opt(),
        children,
    })
}

fn slug_option_value(label: &str) -> String {
    let mut s = label
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect::<String>();
    while s.contains("__") {
        s = s.replace("__", "_");
    }
    s.trim_matches('_').to_string()
}

fn component_label(el: &Element, ctx: &TemplateContext) -> Result<String, String> {
    optional_component_label(el, ctx).map(|label| label.unwrap_or_default())
}

fn optional_component_label(el: &Element, ctx: &TemplateContext) -> Result<Option<String>, String> {
    if let Some(label) = binding_string(el, "label", ctx) {
        return Ok(Some(label));
    }
    match collect_primary_text(el.children.as_slice(), ctx) {
        Ok(label) if !label.is_empty() => Ok(Some(label)),
        Ok(_) => Ok(None),
        Err(_) => Ok(None),
    }
}

fn binding_raw(el: &Element, prop: &str) -> Option<String> {
    el.bindings
        .iter()
        .find(|binding| binding.prop == prop)
        .map(|binding| {
            binding
                .value
                .trim()
                .trim_matches(|c| c == '"' || c == '\'')
                .to_string()
        })
        .filter(|value| !value.is_empty())
}

fn binding_string(el: &Element, prop: &str, ctx: &TemplateContext) -> Option<String> {
    el.bindings
        .iter()
        .find(|binding| binding.prop == prop)
        .map(|binding| value_to_str(&eval_expr(&binding.value, ctx)))
        .map(|value| {
            value
                .trim()
                .trim_matches(|c| c == '"' || c == '\'')
                .to_string()
        })
        .filter(|value| !value.is_empty())
}

fn binding_f32(el: &Element, prop: &str, ctx: &TemplateContext) -> Option<f32> {
    binding_string(el, prop, ctx).and_then(|value| value.parse().ok())
}

fn binding_bool(el: &Element, prop: &str, ctx: &TemplateContext) -> Option<bool> {
    binding_string(el, prop, ctx).and_then(|value| match value.as_str() {
        "true" | "1" | "yes" | "on" => Some(true),
        "false" | "0" | "no" | "off" => Some(false),
        _ => None,
    })
}

fn event_handler(el: &Element, event: &str) -> Option<String> {
    el.event_handlers
        .iter()
        .find(|handler| handler.event == event)
        .map(|handler| handler.handler.clone())
}

fn is_text_tag(tag: &str) -> bool {
    matches!(
        tag,
        "span" | "text" | "p" | "label" | "caption" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6"
    )
}

fn collect_primary_text(children: &[Node], ctx: &TemplateContext) -> Result<String, String> {
    for c in children {
        match c {
            Node::Text(parts) => return render_text_inline(parts, ctx),
            Node::Element(inner) => {
                let s = collect_primary_text(&inner.children, ctx)?;
                if !s.is_empty() {
                    return Ok(s);
                }
            }
            _ => {}
        }
    }
    Ok(String::new())
}

fn active_classes(el: &Element, ctx: &TemplateContext) -> Vec<String> {
    let mut expanded = Vec::new();
    for c in el.classes.iter() {
        expanded.push(ctx.interpolate(c));
    }
    for cc in &el.conditional_classes {
        if ctx.eval_condition(&cc.condition) {
            expanded.push(ctx.interpolate(&cc.class));
        }
    }
    expanded
}

fn stack_axis(classes: &[String]) -> StackAxis {
    let set: std::collections::HashSet<&str> = classes.iter().map(|s| s.as_str()).collect();
    if set.contains("flex-col") {
        StackAxis::Column
    } else if set.contains("flex-row") || set.contains("flex") {
        StackAxis::Row
    } else {
        StackAxis::Column
    }
}

fn parse_gap_spacing(classes: &[String]) -> Option<f32> {
    for c in classes {
        if let Some(rest) = c.strip_prefix("gap-") {
            if let Ok(n) = rest.parse::<u32>() {
                return Some((n as f32) * 4.0);
            }
        }
    }
    None
}

fn render_text_inline(parts: &[TextPart], ctx: &TemplateContext) -> Result<String, String> {
    let mut result = String::new();
    for part in parts {
        match part {
            TextPart::Literal(text) => result.push_str(text),
            TextPart::Expr(expr) => result.push_str(&value_to_str(&eval_expr(expr, ctx))),
        }
    }
    Ok(result)
}

fn render_if(block: &IfBlock, ctx: &TemplateContext) -> Result<ViewNode, String> {
    let body = if ctx.eval_condition(&block.condition) {
        &block.then_children
    } else if let Some(else_children) = &block.else_children {
        else_children
    } else {
        return Ok(stack_column_raw(vec![]));
    };
    let children = render_nodes_list(body, ctx)?;
    Ok(stack_column_raw(children))
}

fn render_for(block: &ForBlock, ctx: &TemplateContext) -> Result<ViewNode, String> {
    let items = ctx.get_list(&block.iterator);
    let mut children = Vec::new();
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
        children.push(render_nodes_list(&block.body, &child_ctx)?);
    }
    let flattened: Vec<ViewNode> = children
        .into_iter()
        .flat_map(|v| {
            if v.len() == 1 {
                v
            } else if v.is_empty() {
                vec![]
            } else {
                vec![stack_column_raw(v)]
            }
        })
        .collect();
    Ok(stack_column_raw(flattened))
}

fn render_match(block: &MatchBlock, ctx: &TemplateContext) -> Result<ViewNode, String> {
    let val = eval_expr(&block.expr, ctx);
    let value = value_to_str(&val);

    for arm in &block.arms {
        let pattern = arm.pattern.trim();
        if pattern == "_" {
            let children = render_nodes_list(&arm.body, ctx)?;
            return Ok(stack_column_raw(children));
        }
        if pattern.starts_with('"') && pattern.ends_with('"') {
            let lit = &pattern[1..pattern.len() - 1];
            if value == lit {
                let children = render_nodes_list(&arm.body, ctx)?;
                return Ok(stack_column_raw(children));
            }
        }
        if value == pattern {
            let children = render_nodes_list(&arm.body, ctx)?;
            return Ok(stack_column_raw(children));
        }
    }
    Ok(stack_column_raw(vec![]))
}
