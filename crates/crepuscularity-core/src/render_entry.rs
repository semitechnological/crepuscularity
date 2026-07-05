use crate::ast::Node;
use crate::context::TemplateContext;
use crate::eval::eval_expr;
use crate::parser::{parse_component_file, parse_template};
use crate::CrepusError;

pub fn render_template_with<F, T>(
    template: &str,
    ctx: &TemplateContext,
    parse: impl FnOnce(&str) -> Result<Vec<Node>, CrepusError>,
    f: F,
) -> Result<T, CrepusError>
where
    F: FnOnce(&[Node], &TemplateContext) -> Result<T, CrepusError>,
{
    let nodes = parse(template)?;
    f(&nodes, ctx)
}

pub fn render_template<F, T>(template: &str, ctx: &TemplateContext, f: F) -> Result<T, CrepusError>
where
    F: FnOnce(&[Node], &TemplateContext) -> Result<T, CrepusError>,
{
    render_template_with(template, ctx, parse_template, f)
}

pub fn render_component_file<F, T>(
    content: &str,
    name: &str,
    ctx: &TemplateContext,
    f: F,
) -> Result<T, CrepusError>
where
    F: FnOnce(&[Node], &TemplateContext) -> Result<T, CrepusError>,
{
    let file = parse_component_file(content)?;
    let component = file
        .components
        .get(name)
        .ok_or_else(|| CrepusError::render(format!("component not found: {name}")))?;

    let mut child_ctx = ctx.clone();
    for (key, expr) in &component.meta.defaults {
        if !child_ctx.vars.contains_key(key) {
            child_ctx
                .vars
                .insert(key.clone(), eval_expr(expr, &TemplateContext::new())?);
        }
    }

    f(&component.nodes, &child_ctx)
}
