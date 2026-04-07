use axum::{response::Html, routing::get, Router};
use crepuscularity_core::TemplateContext;
use crepuscularity_web::render_template_to_html;

const TEMPLATE: &str = include_str!("../../weather.crepus");

async fn index() -> Html<String> {
    let mut ctx = TemplateContext::new();
    ctx.set("city", "London");
    ctx.set("temp", "14");
    ctx.set("description", "Partly cloudy");

    let body = render_template_to_html(TEMPLATE, &ctx)
        .unwrap_or_else(|e| format!("<pre style='color:red'>{e}</pre>"));

    Html(format!(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Weather</title>
  <script src="https://cdn.tailwindcss.com"></script>
</head>
<body class="bg-zinc-950 min-h-screen flex items-center justify-center">
{body}
</body>
</html>"#
    ))
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(index));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3002").await.unwrap();
    println!("weather-web-server listening on http://localhost:3002");
    axum::serve(listener, app).await.unwrap();
}
