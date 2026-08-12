use axum::{response::Html, routing::{get, post}, Form, Router};
use crepuscularity_core::TemplateContext;
use crepuscularity_web::{render_template_to_html, render_template_to_html_with_hydration};
use serde::Deserialize;

const TEMPLATE: &str = include_str!("../../counter.crepus");

async fn index() -> Html<String> {
    let mut ctx = TemplateContext::new();
    ctx.set("count", 0i64);

    let body = render_template_to_html_with_hydration(TEMPLATE, &ctx)
        .unwrap_or_else(|e| format!("<pre style='color:red'>{e}</pre>"));

    Html(format!(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Counter Demo</title>
  <script src="https://cdn.tailwindcss.com"></script>
  <script src="https://unpkg.com/htmx.org@1.9.12"></script>
</head>
<body class="bg-zinc-950 text-white min-h-screen" hx-boost="true">
{body}
</body>
</html>"#
    ))
}

#[derive(Deserialize)]
struct IncrementForm {
    count: i64,
}

async fn increment(Form(form): Form<IncrementForm>) -> Html<String> {
    let new_count = form.count + 1;
    let mut ctx = TemplateContext::new();
    ctx.set("count", new_count);
    let fragment = render_template_to_html(
        r#"p #count-display text-lg
  "Count: {count}""#,
        &ctx,
    )
    .unwrap_or_else(|e| format!("<pre>{e}</pre>"));
    Html(fragment)
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(index))
        .route("/increment", post(increment));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("counter-server listening on http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}
