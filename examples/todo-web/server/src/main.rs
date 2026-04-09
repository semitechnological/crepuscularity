use axum::{response::Html, routing::get, Router};
use crepuscularity_core::{TemplateContext, TemplateValue};
use crepuscularity_web::render_template_to_html;

const TEMPLATE: &str = include_str!("../../todo.crepus");

async fn index() -> Html<String> {
    // Build a list of todo items as TemplateContext rows.
    let items: Vec<TemplateContext> = ["Buy groceries", "Write Rust code", "Deploy to production"]
        .iter()
        .map(|&label| {
            let mut row = TemplateContext::new();
            row.set("value", label);
            row
        })
        .collect();

    let count = items.len();
    let mut ctx = TemplateContext::new();
    ctx.vars
        .insert("items".to_string(), TemplateValue::List(items));
    ctx.set("count", count as i64);

    let body = render_template_to_html(TEMPLATE, &ctx)
        .unwrap_or_else(|e| format!("<pre style='color:red'>{e}</pre>"));

    Html(format!(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Todo List</title>
  <script src="https://cdn.tailwindcss.com"></script>
</head>
<body class="bg-white min-h-screen">
{body}
</body>
</html>"#
    ))
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(index));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
    println!("todo-web-server listening on http://localhost:3001");
    axum::serve(listener, app).await.unwrap();
}
