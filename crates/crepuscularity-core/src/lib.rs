//! Shared parser, evaluator, preprocessing, and analysis primitives for Crepuscularity.
//!
//! Backend crates use this crate as the source of truth for `.crepus` syntax. Keep changes here
//! mirrored into macro-facing code when the same syntax should work at compile time and runtime.

pub mod analysis;
pub mod ast;
pub mod build;
pub mod cache;
pub mod context;
pub mod eval;
pub mod parser;
pub mod preprocess;
mod util;

pub use analysis::{analyze_template, classify_node, BindingMap, Fingerprint, Region};
#[cfg(not(target_arch = "wasm32"))]
pub use cache::DriverCache;
pub use context::{TemplateContext, TemplateValue};
pub use parser::{
    parse_component_file, parse_template, unescape_crepus_text_literal, ComponentDef,
    ComponentFile, ComponentMeta,
};
pub use preprocess::{
    google_fonts_head_markup, merge_unique_font_families, strip_indent_decorators, IndentDecorators,
};
