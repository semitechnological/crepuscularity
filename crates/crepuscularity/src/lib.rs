/// Crepuscularity — general syntax/runtime crate.
///
/// Separate backends live in dedicated crates:
/// - `crepuscularity-web` for HTML and JSX/TSX rendering
/// - `crepuscularity-gpui` for GPUI rendering
pub use crepuscularity_core as core;
pub use crepuscularity_web as html;
pub use crepuscularity_web as web;

pub mod prelude {
    pub use crepuscularity_core::{
        parse_component_file, parse_template, ComponentDef, ComponentFile, ComponentMeta,
        TemplateContext, TemplateValue,
    };
    pub use crepuscularity_web::{
        render_component_file_to_html, render_nodes_to_html, render_template_to_html,
    };
}
