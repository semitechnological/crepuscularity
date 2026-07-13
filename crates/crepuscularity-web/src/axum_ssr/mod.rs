//! Axum handlers/router/stream helpers for SSR.
//!
//! Gated behind the `axum` feature (implies `ssr`).

mod handler;
mod router;
mod stream;
mod util;

pub use handler::{SsrHandler, SsrOptions};
pub use router::{RouteEntry, SsrRouter};
pub use stream::{stream_ssr_response, stream_ssr_response_with_nodes, stream_static_template};
pub use util::escape_html_error;
