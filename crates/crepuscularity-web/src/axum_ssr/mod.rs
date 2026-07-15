//! Axum handlers/router/stream helpers for SSR.
//!
//! Gated behind the `axum` feature (implies `ssr`).

mod handler;
mod router;
mod stream;

pub use handler::{SsrHandler, SsrOptions};
pub use router::{RouteEntry, SsrRouter};
pub use stream::{stream_ssr_response, stream_ssr_response_with_nodes, stream_static_template};

/// Escape HTML special characters in error messages before injecting into HTML.
pub fn escape_html_error(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
