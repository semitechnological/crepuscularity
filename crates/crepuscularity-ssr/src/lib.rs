//! SSR pipeline for Crepuscularity: Axum handlers that render .crepus templates server-side.
mod util;
pub mod handler;
pub mod router;
pub mod stream;

pub use handler::{SsrHandler, SsrOptions};
pub use router::{RouteEntry, SsrRouter};
pub use stream::{stream_ssr_response, stream_ssr_response_with_nodes, stream_static_template};
pub use util::escape_html_error;
