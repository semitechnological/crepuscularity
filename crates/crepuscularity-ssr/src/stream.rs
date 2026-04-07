//! Streaming SSR — renders template in chunks, flushing head early.
//! Placeholder for Phase 6. Full implementation uses `axum::body::Body` + channel.

/// Stream an SSR response. Currently falls back to buffered render.
pub async fn stream_template(template: &str, ctx: crepuscularity_core::TemplateContext) -> String {
    crepuscularity_web::render_template_to_html(template, &ctx)
        .unwrap_or_else(|e| format!("<pre style='color:red'>{e}</pre>"))
}
