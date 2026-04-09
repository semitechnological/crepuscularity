use axum::{Form, Router, response::Html, routing::get};
use base64::Engine;
use crepuscularity_core::{TemplateContext, TemplateValue};
use crepuscularity_web::{SsrDocument, render_ssr_document, render_template_to_html_with_ssr};
use serde::Deserialize;

const INDEX_CREPUS: &str = include_str!("../templates/index.crepus");
const ABOUT_CREPUS: &str = include_str!("../templates/about.crepus");
const TIME_CREPUS: &str = include_str!("../templates/time.crepus");
const ECHO_CREPUS: &str = include_str!("../templates/echo.crepus");
const MANIFEST_CREPUS: &str = include_str!("../templates/manifest.crepus");

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

fn render_page_with_ctx(template: &str, title: &str, ctx: TemplateContext) -> Result<String, String> {
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

async fn time() -> Html<String> {
    let now = chrono::Utc::now().format("%H:%M:%S UTC").to_string();
    let mut ctx = TemplateContext::new();
    ctx.vars.insert("time".to_string(), TemplateValue::Str(now));
    Html(render_page_with_ctx(TIME_CREPUS, "Server Time", ctx).unwrap_or_else(|e| format!("<pre>{e}</pre>")))
}

#[derive(Deserialize)]
struct EchoForm {
    message: String,
}

async fn echo_get() -> Html<String> {
    let mut ctx = TemplateContext::new();
    ctx.vars.insert("message".to_string(), TemplateValue::Str(String::new()));
    ctx.vars.insert("has_echo".to_string(), TemplateValue::Bool(false));
    Html(render_page_with_ctx(ECHO_CREPUS, "SSR Echo", ctx).unwrap_or_else(|e| format!("<pre>{e}</pre>")))
}

async fn echo_post(Form(form): Form<EchoForm>) -> Html<String> {
    let mut ctx = TemplateContext::new();
    ctx.vars.insert("message".to_string(), TemplateValue::Str(form.message.clone()));
    ctx.vars.insert("has_echo".to_string(), TemplateValue::Bool(true));
    Html(render_page_with_ctx(ECHO_CREPUS, "SSR Echo", ctx).unwrap_or_else(|e| format!("<pre>{e}</pre>")))
}

async fn manifest() -> Html<String> {
    let result = (|| -> Result<String, String> {
        let html = render_template_to_html_with_ssr(INDEX_CREPUS, &TemplateContext::new(), true)?;

        // Find the hydration script tag and extract base64 content
        let tag_start = r#"<script type="application/json" id="__crepus_hydration__" data-crepus-encoding="base64">"#;
        let tag_end = "</script>";

        let b64 = html
            .find(tag_start)
            .and_then(|start| {
                let content_start = start + tag_start.len();
                html[content_start..].find(tag_end).map(|end| &html[content_start..content_start + end])
            })
            .ok_or_else(|| "hydration script tag not found".to_string())?;

        let bytes = base64::engine::general_purpose::STANDARD
            .decode(b64)
            .map_err(|e| format!("base64 decode error: {e}"))?;

        let pretty = serde_json::to_string_pretty(
            &serde_json::from_slice::<serde_json::Value>(&bytes)
                .map_err(|e| format!("json parse error: {e}"))?,
        )
        .map_err(|e| format!("json pretty error: {e}"))?;

        let mut ctx = TemplateContext::new();
        ctx.vars.insert("manifest".to_string(), TemplateValue::Str(pretty));
        render_page_with_ctx(MANIFEST_CREPUS, "Hydration Manifest", ctx)
    })();

    Html(result.unwrap_or_else(|e| format!("<pre>{e}</pre>")))
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(index))
        .route("/about", get(about))
        .route("/time", get(time))
        .route("/echo", get(echo_get).post(echo_post))
        .route("/manifest", get(manifest));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3030").await.unwrap();
    println!("SSR demo running at http://localhost:3030");
    axum::serve(listener, app).await.unwrap();
}
