/// Runtime GPUI renderer — walks the AST and builds GPUI elements dynamically.

use gpui::{AnyElement, IntoElement, ParentElement, Styled, div, rgb};

use crate::ast::*;
use crate::context::TemplateContext;
use crate::styler::apply_class;

/// Render a list of nodes into a single `AnyElement`.
pub fn render_nodes(nodes: &[Node], ctx: &TemplateContext) -> AnyElement {
    match nodes.len() {
        0 => div().into_any_element(),
        1 => render_node(&nodes[0], ctx),
        _ => {
            let mut d = div();
            for node in nodes {
                let child = render_node(node, ctx);
                d = d.child(child);
            }
            d.into_any_element()
        }
    }
}

pub fn render_node(node: &Node, ctx: &TemplateContext) -> AnyElement {
    match node {
        Node::Element(el) => render_element(el, ctx),
        Node::Text(parts) => render_text(parts, ctx).into_any_element(),
        Node::If(block) => render_if(block, ctx),
        Node::For(block) => render_for(block, ctx),
        Node::Match(block) => render_match(block, ctx),
        Node::LetDecl(_) => div().into_any_element(), // skip — no runtime eval
        Node::RawText(text) => {
            // Show the raw expression text as a placeholder
            div()
                .text_color(rgb(0x888888))
                .child(text.clone())
                .into_any_element()
        }
    }
}

fn render_element(el: &Element, ctx: &TemplateContext) -> AnyElement {
    let mut d = base_tag_element(&el.tag);

    // Apply Tailwind classes
    for class in &el.classes {
        d = apply_class(d, class);
    }

    // Apply conditional classes
    for cc in &el.conditional_classes {
        if ctx.eval_condition(&cc.condition) {
            d = apply_class(d, &cc.class);
        }
    }

    // Render children
    for child in &el.children {
        let child_el = render_node(child, ctx);
        d = d.child(child_el);
    }

    d.into_any_element()
}

fn base_tag_element(tag: &str) -> gpui::Div {
    match tag {
        "button" => div().cursor_pointer(),
        _ => div(),
    }
}

fn render_text(parts: &[TextPart], ctx: &TemplateContext) -> String {
    let mut result = String::new();
    for part in parts {
        match part {
            TextPart::Literal(text) => result.push_str(text),
            TextPart::Expr(expr) => result.push_str(&ctx.get_str(expr)),
        }
    }
    result
}

fn render_if(block: &IfBlock, ctx: &TemplateContext) -> AnyElement {
    if ctx.eval_condition(&block.condition) {
        render_nodes(&block.then_children, ctx)
    } else if let Some(else_children) = &block.else_children {
        render_nodes(else_children, ctx)
    } else {
        div().into_any_element()
    }
}

fn render_for(block: &ForBlock, ctx: &TemplateContext) -> AnyElement {
    // The iterator name maps to a list in the context
    let items = ctx.get_list(&block.iterator);

    let mut d = div();
    for item_ctx in items {
        // For each item, create a context that merges the item vars under the pattern name
        // For simple patterns like `item`, merge the item's vars plus `{pattern}` as a key
        let mut child_ctx = ctx.clone();
        // Merge all item vars into the child context
        for (k, v) in &item_ctx.vars {
            child_ctx.vars.insert(k.clone(), v.clone());
        }
        // Also expose the iteration variable directly using the pattern as prefix
        let pattern = block.pattern.trim();
        if !pattern.is_empty() {
            // Copy the item's string representation under the pattern name
            let item_str = item_ctx.get_str("value");
            if !item_str.is_empty() {
                child_ctx.vars.insert(pattern.to_string(), crate::context::TemplateValue::Str(item_str));
            }
        }

        let child = render_nodes(&block.body, &child_ctx);
        d = d.child(child);
    }
    d.into_any_element()
}

fn render_match(block: &MatchBlock, ctx: &TemplateContext) -> AnyElement {
    // Evaluate the match expression as a string and compare against arm patterns
    let value = ctx.get_str(&block.expr);

    for arm in &block.arms {
        let pattern = arm.pattern.trim();
        // Wildcard
        if pattern == "_" {
            return render_nodes(&arm.body, ctx);
        }
        // String literal match: "literal"
        if pattern.starts_with('"') && pattern.ends_with('"') {
            let lit = &pattern[1..pattern.len() - 1];
            if value == lit {
                return render_nodes(&arm.body, ctx);
            }
        }
        // Direct value match
        if value == pattern {
            return render_nodes(&arm.body, ctx);
        }
        // Enum variant (without payload): MyEnum::Variant
        // Just do string comparison
        if value.contains("::") && pattern.contains("::") && value == pattern {
            return render_nodes(&arm.body, ctx);
        }
    }

    div().into_any_element()
}
