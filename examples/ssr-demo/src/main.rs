use axum::{Router, response::Html, routing::get};
use crepuscularity_core::TemplateContext;
use crepuscularity_web::{SsrDocument, render_ssr_document};

const INDEX_CREPUS: &str = include_str!("../templates/index.crepus");
const ABOUT_CREPUS: &str = include_str!("../templates/about.crepus");

const TAILWIND: &str = r#"<script src="https://cdn.tailwindcss.com"></script>"#;

fn render_page(template: &str, title: &str) -> Result<String, String> {
    let ctx = TemplateContext::new();
    render_ssr_document(
        template,
        &ctx,
        &SsrDocument {
            title,
            head_extra: TAILWIND,
            ..Default::default()
        },
        true,
    )
}

async fn index() -> Html<String> {
    Html(render_page(INDEX_CREPUS, "Home").unwrap_or_else(|e| format!("<pre>{e}</pre>")))
}

async fn about() -> Html<String> {
    Html(render_page(ABOUT_CREPUS, "About").unwrap_or_else(|e| format!("<pre>{e}</pre>")))
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(index))
        .route("/about", get(about));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3030").await.unwrap();
    println!("SSR demo running at http://localhost:3030");
    axum::serve(listener, app).await.unwrap();
}
