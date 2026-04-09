//! Lower `.crepus` templates to a small JSON view tree for SwiftUI and Jetpack Compose samples.
//!
//! v1 supports `Text`, `Stack` (`row` / `column`), control flow (`if`, `for`, `match`), `slot`, and
//! a static preview for `slot-rotate` (first phrase only). `include` is rejected until the IR
//! includes virtual file resolution.

use crepuscularity_core::ast::*;
use crepuscularity_core::context::{value_to_str, TemplateContext, TemplateValue};
use crepuscularity_core::eval::eval_expr;
use crepuscularity_core::parser::{parse_component_file, parse_template};
use crepuscularity_core::preprocess::slot_rotate_child_phrases;
use serde::{Deserialize, Serialize};

const IR_VERSION: u32 = 1;

/// Root document produced by [`render_template_to_ir`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ViewIr {
    pub version: u32,
    pub root: Vec<ViewNode>,
}

/// A node in the platform-neutral tree. Serialized with `kind` tagging for Swift/Kotlin decoders.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ViewNode {
    #[serde(rename = "text")]
    Text { content: String },
    #[serde(rename = "stack")]
    Stack {
        axis: StackAxis,
        #[serde(skip_serializing_if = "Option::is_none")]
        spacing: Option<f32>,
        children: Vec<ViewNode>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StackAxis {
    Row,
    Column,
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
        out.push(render_node(node, &ctx)?);
    }
    Ok(out)
}

fn render_node(node: &Node, ctx: &TemplateContext) -> Result<ViewNode, String> {
    match node {
        Node::Element(el) => render_element(el, ctx),
        Node::Text(parts) => Ok(ViewNode::Text {
            content: render_text(parts, ctx)?,
        }),
        Node::If(block) => render_if(block, ctx),
        Node::For(block) => render_for(block, ctx),
        Node::Match(block) => render_match(block, ctx),
        Node::LetDecl(_) => Ok(stack_column(vec![])),
        Node::Include(_) => Err(
            "native IR v1 does not support `include`; use `render_template_to_ir` on expanded templates or the web/renderer path".into(),
        ),
        Node::RawText(expr) => Ok(ViewNode::Text {
            content: value_to_str(&eval_expr(expr, ctx)),
        }),
    }
}

fn stack_column(children: Vec<ViewNode>) -> ViewNode {
    ViewNode::Stack {
        axis: StackAxis::Column,
        spacing: None,
        children,
    }
}

fn render_element(el: &Element, ctx: &TemplateContext) -> Result<ViewNode, String> {
    if el.tag == "slot" {
        return if let Some((slot_nodes, slot_ctx)) = &ctx.slot {
            let children = render_nodes_list(slot_nodes, slot_ctx)?;
            Ok(stack_column(children))
        } else {
            let children = render_nodes_list(&el.children, ctx)?;
            Ok(stack_column(children))
        };
    }

    if el.tag == "slot-rotate" {
        let phrases = slot_rotate_child_phrases(&el.children)?;
        let label = phrases
            .into_iter()
            .next()
            .filter(|s| !s.is_empty())
            .unwrap_or_default();
        return Ok(stack_column(vec![ViewNode::Text { content: label }]));
    }

    let classes = active_classes(el, ctx);
    let axis = stack_axis(&classes);
    let spacing = parse_gap_spacing(&classes);
    let children = render_nodes_list(&el.children, ctx)?;

    Ok(ViewNode::Stack {
        axis,
        spacing,
        children,
    })
}

fn active_classes(el: &Element, ctx: &TemplateContext) -> Vec<String> {
    let mut class_names = el.classes.clone();
    for cc in &el.conditional_classes {
        if ctx.eval_condition(&cc.condition) {
            class_names.push(cc.class.clone());
        }
    }
    class_names
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

/// Best-effort Tailwind `gap-*` → logical spacing (value × 4, matching common dp scale).
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
        return Ok(stack_column(vec![]));
    };
    let children = render_nodes_list(body, ctx)?;
    Ok(stack_column(children))
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
    // Flatten each iteration to a single stack (column of body nodes merged)
    let flattened: Vec<ViewNode> = children
        .into_iter()
        .flat_map(|v| {
            if v.len() == 1 {
                v
            } else if v.is_empty() {
                vec![]
            } else {
                vec![stack_column(v)]
            }
        })
        .collect();
    Ok(stack_column(flattened))
}

fn render_match(block: &MatchBlock, ctx: &TemplateContext) -> Result<ViewNode, String> {
    let val = eval_expr(&block.expr, ctx);
    let value = value_to_str(&val);

    for arm in &block.arms {
        let pattern = arm.pattern.trim();
        if pattern == "_" {
            let children = render_nodes_list(&arm.body, ctx)?;
            return Ok(stack_column(children));
        }
        if pattern.starts_with('"') && pattern.ends_with('"') {
            let lit = &pattern[1..pattern.len() - 1];
            if value == lit {
                let children = render_nodes_list(&arm.body, ctx)?;
                return Ok(stack_column(children));
            }
        }
        if value == pattern {
            let children = render_nodes_list(&arm.body, ctx)?;
            return Ok(stack_column(children));
        }
    }
    Ok(stack_column(vec![]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn plain_text_stack() {
        let mut ctx = TemplateContext::new();
        ctx.set("name", "Ada");
        let tpl = "div flex flex-col gap-4\n  span\n    \"Hello {name}\"";
        let ir = render_template_to_ir(tpl, &ctx).unwrap();
        assert_eq!(ir.version, 1);
        assert_eq!(ir.root.len(), 1);

        let expected = json!({
            "version": 1,
            "root": [{
                "kind": "stack",
                "axis": "column",
                "spacing": 16.0,
                "children": [{
                    "kind": "stack",
                    "axis": "column",
                    "children": [{
                        "kind": "text",
                        "content": "Hello Ada"
                    }]
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
        // div wraps the for-loop stack; iterations are nested under that stack
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
    fn include_rejected() {
        let ctx = TemplateContext::new();
        let err = render_template_to_ir("include other.crepus", &ctx).unwrap_err();
        assert!(err.contains("include"));
    }

    #[test]
    fn match_arm() {
        let mut ctx = TemplateContext::new();
        ctx.set("status", "on");
        // Match arms must share the same indent as `match` (parser contract).
        let tpl = "div\n match {status}\n \"on\" =>\n  \"OK\"\n _ =>\n  \"?\"";
        let ir = render_template_to_ir(tpl, &ctx).unwrap();
        let v = serde_json::to_value(&ir).unwrap();
        assert!(v.to_string().contains("OK"), "expected OK in {}", v);
        round_trip(&ir);
    }
}
