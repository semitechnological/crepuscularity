pub mod analysis;
pub mod ast;
pub mod cache;
pub mod context;
pub mod eval;
pub mod parser;
pub mod preprocess;

pub use analysis::{analyze_template, classify_node, BindingMap, Fingerprint, Region};
#[cfg(not(target_arch = "wasm32"))]
pub use cache::DriverCache;
pub use context::{TemplateContext, TemplateValue};
pub use parser::{
    parse_component_file, parse_template, ComponentDef, ComponentFile, ComponentMeta,
};
pub use preprocess::{
    google_fonts_head_markup, merge_unique_font_families, strip_indent_decorators, IndentDecorators,
};
