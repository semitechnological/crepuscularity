//! Convenience facade for Crepuscularity applications.
//!
//! The backend crates remain independently usable. This crate re-exports the shared core syntax
//! APIs plus the HTML/WASM renderer for applications that want one dependency.
pub use crepuscularity_core as core;
pub use crepuscularity_core::build;
pub use crepuscularity_web as html;
pub use crepuscularity_web as web;
pub use crepuscularity_web::crepus_refs;

pub mod target;

pub mod prelude {
    pub use crepuscularity_core::{TemplateContext, TemplateValue};
    pub use crepuscularity_web::{render_component_file_to_html, render_template_to_html};
}
