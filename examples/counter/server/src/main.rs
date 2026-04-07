use axum::{response::Html, routing::get, Router};
use crepuscularity_core::TemplateContext;
use crepuscularity_web::render_template_to_html_with_hydration;

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
</head>
<body class="bg-zinc-950 text-white min-h-screen">
{body}
</body>
</html>"#
    ))
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(index));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("counter-server listening on http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}
