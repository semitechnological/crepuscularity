//! Lower `.crepus` templates to a JSON view tree for SwiftUI, Jetpack Compose, and future
//! `android-activity` / `objc2` shells.
//!
//! ## Coverage vs GPUI / web
//! This is **not** 100% parity with `crepuscularity-runtime`/`crepuscularity-gpui` `styler.rs`.
//! [`style`] implements a **growing** Tailwind subset (spacing, typography, common colors, radii,
//! flex alignment, dynamic `bg-{expr}` / `text-{expr}`). Extend `style.rs` for missing classes.
//!
//! Supported control flow: `if`, `for`, `match`, `include` (virtual FS + disk), `slot`,
//! `slot-rotate` (all phrases + interval in IR). Widgets: `button`, `img`, scroll containers
//! (`overflow-y-scroll`, etc.), styled `span`→`text`, generic `div` stacks.

mod include_expand;
mod style;

use std::collections::HashMap;

use crepuscularity_core::ast::*;
use crepuscularity_core::context::{value_to_str, TemplateContext, TemplateValue};
use crepuscularity_core::eval::eval_expr;
use crepuscularity_core::parser::{parse_component_file, parse_template};
use crepuscularity_core::preprocess::slot_rotate_child_phrases;
use serde::{Deserialize, Serialize};

/// Bumped when the JSON schema gains incompatible fields; shells should check `version`.
pub const IR_VERSION: u32 = 2;

/// Root document produced by [`render_template_to_ir`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ViewIr {
    pub version: u32,
    pub root: Vec<ViewNode>,
}

/// Portable layout/theming hints mapped from Tailwind-like classes (see `style.rs`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ViewStyle {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub padding: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub padding_horizontal: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub padding_vertical: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub padding_top: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub padding_bottom: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub padding_left: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub padding_right: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin_horizontal: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin_vertical: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin_top: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin_bottom: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin_left: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin_right: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_size: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_weight: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_align: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub foreground_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub corner_radius: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub italic: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub underline: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strikethrough: Option<bool>,
}

impl ViewStyle {
    fn is_effectively_empty(&self) -> bool {
        self.padding.is_none()
            && self.padding_horizontal.is_none()
            && self.padding_vertical.is_none()
            && self.padding_top.is_none()
            && self.padding_bottom.is_none()
            && self.padding_left.is_none()
            && self.padding_right.is_none()
            && self.margin.is_none()
            && self.margin_horizontal.is_none()
            && self.margin_vertical.is_none()
            && self.margin_top.is_none()
            && self.margin_bottom.is_none()
            && self.margin_left.is_none()
            && self.margin_right.is_none()
            && self.font_size.is_none()
            && self.font_weight.is_none()
            && self.text_align.is_none()
            && self.foreground_color.is_none()
            && self.background_color.is_none()
            && self.corner_radius.is_none()
            && self.italic.is_none()
            && self.underline.is_none()
            && self.strikethrough.is_none()
    }

    fn opt(self) -> Option<ViewStyle> {
        if self.is_effectively_empty() {
            None
        } else {
            Some(self)
        }
    }
}

/// A node in the platform-neutral tree. Serialized with `kind` for Swift/Kotlin.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ViewNode {
    #[serde(rename = "text")]
    Text {
        content: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        style: Option<ViewStyle>,
    },
    #[serde(rename = "stack")]
    Stack {
        axis: StackAxis,
        #[serde(skip_serializing_if = "Option::is_none")]
        spacing: Option<f32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        align_items: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        justify_content: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        style: Option<ViewStyle>,
        children: Vec<ViewNode>,
    },
    #[serde(rename = "button")]
    Button {
        label: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        on_click: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        style: Option<ViewStyle>,
    },
    #[serde(rename = "image")]
    Image {
        src: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        alt: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        style: Option<ViewStyle>,
    },
    #[serde(rename = "scroll")]
    Scroll {
        axis: StackAxis,
        #[serde(skip_serializing_if = "Option::is_none")]
        style: Option<ViewStyle>,
        children: Vec<ViewNode>,
    },
    #[serde(rename = "slotRotate")]
    SlotRotate {
        phrases: Vec<String>,
        interval_ms: u64,
        #[serde(skip_serializing_if = "Option::is_none")]
        style: Option<ViewStyle>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StackAxis {
    Row,
    Column,
}

/// Render from a virtual file map (`entry` may use `#Component` suffix).
pub fn render_from_files(
    files: &HashMap<String, String>,
    entry: &str,
    ctx: &TemplateContext,
) -> Result<ViewIr, String> {
    let mut ctx = ctx.clone();
    ctx.virtual_files = files.clone();

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
        child_ctx
            .vars
            .entry(key.clone())
            .or_insert_with(|| eval_expr(expr, &TemplateContext::new()));
    }

    render_nodes_to_ir(&component.nodes, &child_ctx)
}

/// Serialize IR to JSON (compact).
pub fn to_json(ir: &ViewIr) -> Result<String, serde_json::Error> {
    serde_json::to_string(ir)
}

/// Serialize IR to pretty-printed JSON (fixtures / debugging).
pub fn to_json_pretty(ir: &ViewIr) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(ir)
}

/// Lower already-parsed nodes into a [`ViewIr`].
pub fn render_nodes_to_ir(nodes: &[Node], ctx: &TemplateContext) -> Result<ViewIr, String> {
    let root = render_nodes_list(nodes, ctx)?;
    Ok(ViewIr {
        version: IR_VERSION,
        root,
    })
}

fn render_nodes_list(nodes: &[Node], ctx: &TemplateContext) -> Result<Vec<ViewNode>, String> {
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
            content: render_text(parts, ctx)?,
            style: None,
        }),
        Node::If(block) => render_if(block, ctx),
        Node::For(block) => render_for(block, ctx),
        Node::Match(block) => render_match(block, ctx),
        Node::LetDecl(_) => Ok(stack_column_raw(vec![])),
        Node::Include(_) => {
            Err("internal error: include should be expanded in render_nodes_list".into())
        }
        Node::RawText(expr) => Ok(ViewNode::Text {
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
    if el.tag == "slot" {
        return if let Some((slot_nodes, slot_ctx)) = &ctx.slot {
            let children = render_nodes_list(slot_nodes, slot_ctx)?;
            Ok(stack_column_raw(children))
        } else {
            let children = render_nodes_list(&el.children, ctx)?;
            Ok(stack_column_raw(children))
        };
    }

    if el.tag == "slot-rotate" {
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

    if el.tag == "button" {
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

    if el.tag == "img" {
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
        let hints = style::extract_stack_hints(&classes, Some(ctx));
        return Ok(ViewNode::Image {
            src,
            alt,
            style: hints.style.opt(),
        });
    }

    // Single text-ish span: collapse to styled Text
    if el.tag == "span" && el.children.len() == 1 {
        if let Node::Text(parts) = &el.children[0] {
            let txt = render_text(parts, ctx)?;
            let st = style::extract_text_style(&classes, Some(ctx)).opt();
            return Ok(ViewNode::Text {
                content: txt,
                style: st,
            });
        }
    }

    let axis = stack_axis(&classes);
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

fn collect_primary_text(children: &[Node], ctx: &TemplateContext) -> Result<String, String> {
    for c in children {
        match c {
            Node::Text(parts) => return render_text(parts, ctx),
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
    // Interpolate `{expr}` in class strings using template context (Tailwind dynamic palette).
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

fn render_text(parts: &[TextPart], ctx: &TemplateContext) -> Result<String, String> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::HashMap;

    #[test]
    fn plain_text_stack() {
        let mut ctx = TemplateContext::new();
        ctx.set("name", "Ada");
        let tpl = "div flex flex-col gap-4\n  span\n    \"Hello {name}\"";
        let ir = render_template_to_ir(tpl, &ctx).unwrap();
        assert_eq!(ir.version, IR_VERSION);

        let expected = json!({
            "version": IR_VERSION,
            "root": [{
                "kind": "stack",
                "axis": "column",
                "spacing": 16.0,
                "children": [{
                    "kind": "text",
                    "content": "Hello Ada"
                }]
            }]
        });
        let v: serde_json::Value = serde_json::to_value(&ir).unwrap();
        assert_eq!(v, expected);
    }

    fn round_trip(ir: &ViewIr) {
        let s = to_json(ir).unwrap();
        let back: ViewIr = serde_json::from_str(&s).unwrap();
        assert_eq!(*ir, back);
    }

    #[test]
    fn serde_round_trip() {
        let mut ctx = TemplateContext::new();
        ctx.set("show", true);
        let ir = render_template_to_ir(
            "div flex flex-row\n if {show}\n  \"yes\"\n else\n  \"no\"",
            &ctx,
        )
        .unwrap();
        round_trip(&ir);
    }

    #[test]
    fn for_loop() {
        let mut ctx = TemplateContext::new();
        let mut a = TemplateContext::new();
        a.set("value", "a");
        let mut b = TemplateContext::new();
        b.set("value", "b");
        ctx.set(
            "items",
            crepuscularity_core::TemplateValue::List(vec![a, b]),
        );
        let tpl = "div\n for item in {items}\n  span\n    \"{item}\"";
        let ir = render_template_to_ir(tpl, &ctx).unwrap();
        let v = serde_json::to_value(&ir).unwrap();
        assert_eq!(
            v["root"][0]["children"][0]["children"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
        round_trip(&ir);
    }

    #[test]
    fn include_virtual_file() {
        let mut ctx = TemplateContext::new();
        let mut files = HashMap::new();
        files.insert(
            "child.crepus".into(),
            "span text-green-400\n  \"In child\"".into(),
        );
        ctx.virtual_files = files;
        let tpl = "include child.crepus";
        let ir = render_template_to_ir(tpl, &ctx).unwrap();
        let s = serde_json::to_string(&ir).unwrap();
        assert!(s.contains("In child"));
        assert!(s.contains("#4ade80") || s.contains("green"));
    }

    #[test]
    fn match_arm() {
        let mut ctx = TemplateContext::new();
        ctx.set("status", "on");
        let tpl = "div\n match {status}\n \"on\" =>\n  \"OK\"\n _ =>\n  \"?\"";
        let ir = render_template_to_ir(tpl, &ctx).unwrap();
        let v = serde_json::to_value(&ir).unwrap();
        assert!(v.to_string().contains("OK"), "expected OK in {}", v);
        round_trip(&ir);
    }

    #[test]
    fn button_and_dynamic_color() {
        let mut ctx = TemplateContext::new();
        ctx.set("surface", "18181b");
        let tpl = "button @click=\"go\" bg-{surface}\n  \"Tap\"";
        let ir = render_template_to_ir(tpl, &ctx).unwrap();
        let v = serde_json::to_value(&ir).unwrap();
        assert_eq!(v["root"][0]["kind"], "button");
        assert_eq!(v["root"][0]["label"], "Tap");
        round_trip(&ir);
    }

    #[test]
    fn render_from_files_entry() {
        let mut files = HashMap::new();
        files.insert("main.crepus".into(), "div\n  \"ok\"".into());
        let ir = render_from_files(&files, "main.crepus", &TemplateContext::new()).unwrap();
        assert_eq!(ir.root.len(), 1);
    }
}
