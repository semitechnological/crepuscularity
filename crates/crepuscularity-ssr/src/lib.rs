//! SSR pipeline for Crepuscularity: Axum handlers that render .crepus templates server-side.
pub mod handler;
pub mod router;
pub mod stream;

pub use handler::{SsrHandler, SsrOptions};
pub use router::{RouteEntry, SsrRouter};
pub use stream::{stream_ssr_response, stream_static_template};
