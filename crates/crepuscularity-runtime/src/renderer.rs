/// Runtime GPUI renderer — walks the AST and builds GPUI elements dynamically.

use gpui::{AnyElement, IntoElement, ParentElement, Styled, div, rgb};

use crate::ast::*;
use crate::context::{TemplateContext, TemplateValue, value_to_str};
use crate::styler::apply_class;

/// Render a list of nodes into a single `AnyElement`, threading `LetDecl`s into
/// a running context clone so later siblings see the declared variables.
pub fn render_nodes(nodes: &[Node], ctx: &TemplateContext) -> AnyElement {
    render_nodes_with_ctx(nodes, ctx.clone())
}

fn render_nodes_with_ctx(nodes: &[Node], mut ctx: TemplateContext) -> AnyElement {
    let mut rendered: Vec<AnyElement> = Vec::new();

    for node in nodes {
        if let Node::LetDecl(decl) = node {
            if decl.is_default && ctx.vars.contains_key(&decl.name) {
                // Default: skip if already set by parent props
            } else {
                let val = crate::eval::eval_expr(&decl.expr, &ctx);
                ctx.vars.insert(decl.name.clone(), val);
            }
        } else {
            rendered.push(render_node(node, &ctx));
        }
    }

    match rendered.len() {
        0 => div().into_any_element(),
        1 => rendered.remove(0),
        _ => {
            let mut d = div();
            for child in rendered {
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
        Node::LetDecl(_) => div().into_any_element(), // handled in render_nodes_with_ctx
        Node::RawText(expr) => {
            let val = crate::eval::eval_expr(expr, ctx);
            div().child(value_to_str(&val)).into_any_element()
        }
        Node::Include(inc) => render_include(inc, ctx),
    }
}

fn render_element(el: &Element, ctx: &TemplateContext) -> AnyElement {
    // Intercept the `slot` pseudo-tag: render slot content from parent, or fallback children.
    if el.tag == "slot" {
        return if let Some((slot_nodes, slot_ctx)) = &ctx.slot {
            render_nodes(slot_nodes, slot_ctx)
        } else {
            render_nodes(&el.children, ctx)
        };
    }

    let mut d = base_tag_element(&el.tag);

    for class in &el.classes {
        d = apply_class(d, class);
    }

    for cc in &el.conditional_classes {
        if ctx.eval_condition(&cc.condition) {
            d = apply_class(d, &cc.class);
        }
    }

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
            TextPart::Expr(expr) => {
                let val = crate::eval::eval_expr(expr, ctx);
                result.push_str(&value_to_str(&val));
            }
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
    let items = ctx.get_list(&block.iterator);

    let mut d = div();
    for item_ctx in items {
        let mut child_ctx = ctx.clone();
        for (k, v) in &item_ctx.vars {
            child_ctx.vars.insert(k.clone(), v.clone());
        }
        let pattern = block.pattern.trim();
        if !pattern.is_empty() {
            let item_str = item_ctx.get_str("value");
            if !item_str.is_empty() {
                child_ctx.vars.insert(pattern.to_string(), TemplateValue::Str(item_str));
            }
        }

        let child = render_nodes(&block.body, &child_ctx);
        d = d.child(child);
    }
    d.into_any_element()
}

fn render_match(block: &MatchBlock, ctx: &TemplateContext) -> AnyElement {
    let val = crate::eval::eval_expr(&block.expr, ctx);
    let value = value_to_str(&val);

    for arm in &block.arms {
        let pattern = arm.pattern.trim();
        if pattern == "_" {
            return render_nodes(&arm.body, ctx);
        }
        if pattern.starts_with('"') && pattern.ends_with('"') {
            let lit = &pattern[1..pattern.len() - 1];
            if value == lit {
                return render_nodes(&arm.body, ctx);
            }
        }
        if value == pattern {
            return render_nodes(&arm.body, ctx);
        }
    }

    div().into_any_element()
}

fn render_include(inc: &IncludeNode, ctx: &TemplateContext) -> AnyElement {
    // Resolve the component path relative to the current file's directory.
    let file_path = if let Some(base) = &ctx.base_dir {
        base.join(&inc.path)
    } else {
        std::path::PathBuf::from(&inc.path)
    };

    let content = match std::fs::read_to_string(&file_path) {
        Ok(c) => c,
        Err(e) => {
            let msg = format!("include error: {:?}: {}", file_path, e);
            return div().text_color(rgb(0xff4444)).child(msg).into_any_element();
        }
    };

    let nodes = match crate::parser::parse_template(&content) {
        Ok(n) => n,
        Err(e) => {
            let msg = format!("include parse error: {}", e);
            return div().text_color(rgb(0xff4444)).child(msg).into_any_element();
        }
    };

    // Build child context: fresh vars from evaluated props, correct base_dir, and slot.
    let mut child_ctx = TemplateContext::new();
    child_ctx.base_dir = file_path.parent().map(|p| p.to_path_buf());

    for (key, expr) in &inc.props {
        let val = crate::eval::eval_expr(expr, ctx);
        child_ctx.vars.insert(key.clone(), val);
    }

    if !inc.slot.is_empty() {
        child_ctx.slot = Some((inc.slot.clone(), Box::new(ctx.clone())));
    }

    render_nodes(&nodes, &child_ctx)
}
