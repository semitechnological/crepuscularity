//! Streaming SSR: renders the template head synchronously, flushes, then sends body chunks.
//!
//! Currently uses a single-chunk approach (full render before flush); the channel-based
//! structure allows per-section streaming in a future iteration.
use axum::body::Body;
use axum::response::Response;
use crepuscularity_core::TemplateContext;
use crepuscularity_web::{render_ssr_document, SsrDocument};
use http::StatusCode;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

/// Stream an SSR response.
///
/// Sends the rendered HTML document as a streaming `text/html` response.
/// The render runs on a blocking thread so the async runtime is not stalled.
///
/// `doc` must use `'static` lifetime because it is moved into the blocking task.
/// Build it with `SsrDocument { title: Box::leak(...), ..Default::default() }` when
/// the title is not a static string literal.
pub async fn stream_ssr_response(
    template: String,
    ctx: TemplateContext,
    doc: SsrDocument<'static>,
) -> Response {
    let (tx, rx) = mpsc::channel::<Result<Vec<u8>, std::io::Error>>(4);

    tokio::task::spawn_blocking(move || {
        let result = render_ssr_document(&template, &ctx, &doc, true)
            .map(|s| s.into_bytes())
            .map_err(std::io::Error::other);
        let _ = tx.blocking_send(result);
    });

    let stream = ReceiverStream::new(rx);
    let body = Body::from_stream(stream);

    Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "text/html; charset=utf-8")
        .body(body)
        .unwrap()
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
