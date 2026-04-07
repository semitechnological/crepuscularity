//! SSR pipeline for Crepuscularity: Axum handlers that render .crepus templates server-side.
pub mod handler;
pub mod stream;

pub use handler::{SsrHandler, SsrOptions};
