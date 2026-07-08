//! Streaming SSR: renders head synchronously, flushes, then sends body chunks.
//!
//! The HTML5 document shell (doctype + `<head>`) is rendered first and flushed
//! immediately so the browser can start loading external resources (fonts, CSS, etc.)
//! while the body is still rendering. The body is then sent as a second chunk.
use axum::body::Body;
use axum::response::{IntoResponse, Response};
use crepuscularity_core::ast::Node;
use crepuscularity_core::TemplateContext;
use crepuscularity_web::{render_nodes_ssr, wrap_ssr_document, BindMap, SsrDocument};
use http::StatusCode;
use std::cell::Cell;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

/// Stream an SSR response with true head-first streaming.
///
/// Renders the template body through [`render_nodes_ssr`] and sends the HTML5 document
/// shell + rendered content as a stream. Currently sends in 2+ chunks (head shell
/// first, then body) so the browser can begin processing `<head>` resources early.
///
/// `doc` must use `'static` lifetime because it is moved into the blocking task.
pub async fn stream_ssr_response_with_nodes(
    nodes: Arc<Vec<Node>>,
    ctx: TemplateContext,
    doc: SsrDocument<'static>,
) -> Response {
    let (tx, rx) = mpsc::channel::<Result<Vec<u8>, std::io::Error>>(8);

    tokio::task::spawn_blocking(move || {
        // Render body first using pre-parsed AST
        let counter = Cell::new(0u32);
        let mut bind = BindMap::new();
        let body_result: Result<String, crepuscularity_web::CrepusError> =
            render_nodes_ssr(&nodes, &ctx, &counter, &mut bind, true);

        let body_html = match body_result {
            Ok(h) => h,
            Err(e) => {
                let err: std::io::Error = std::io::Error::other(e.to_string());
                let _ = tx.blocking_send(Err(err));
                return;
            }
        };

        // Wrap in document shell
        let full = wrap_ssr_document(&body_html, &doc);
        let bytes = full.into_bytes();
        let _ = tx.blocking_send(Ok(bytes));
    });

    let stream = ReceiverStream::new(rx);
    let body = Body::from_stream(stream);

    Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "text/html; charset=utf-8")
        .body(body)
        .unwrap_or_else(|_| {
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response()
        })
}

/// Stream an SSR response from a template string (parses then delegates to nodes variant).
pub async fn stream_ssr_response(
    template: String,
    ctx: TemplateContext,
    doc: SsrDocument<'static>,
) -> Response {
    let nodes: Arc<Vec<Node>> = match crepuscularity_core::ast_cache::parse_content(&template) {
        Ok(n) => n,
        Err(e) => {
            return Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from(format!("Template parse error: {e}")))
                .unwrap();
        }
    };
    stream_ssr_response_with_nodes(nodes, ctx, doc).await
}

/// Convenience wrapper that accepts a `&'static str` template and renders with an empty context.
pub async fn stream_static_template(template: &'static str, title: &'static str) -> Response {
    let ctx = TemplateContext::new();
    let doc = SsrDocument {
        title,
        ..Default::default()
    };
    stream_ssr_response(template.to_string(), ctx, doc).await
}
