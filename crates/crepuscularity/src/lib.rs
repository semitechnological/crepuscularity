/// Crepuscularity — general syntax/runtime crate.
///
/// Separate backends live in dedicated crates:
/// - `crepuscularity-web` for HTML rendering
/// - `crepuscularity-react` for JSX/TSX-style rendering
/// - `crepuscularity-gpui` for GPUI rendering
pub use crepuscularity_core as core;
pub use crepuscularity_react as react;
pub use crepuscularity_web as html;
pub use crepuscularity_web as web;

pub mod prelude {
    pub use crepuscularity_core::{
        parse_component_file, parse_template, ComponentDef, ComponentFile, ComponentMeta,
        TemplateContext, TemplateValue,
    };
    pub use crepuscularity_react::{
        render_component_file_to_jsx, render_nodes_to_jsx, render_template_to_jsx,
    };
    pub use crepuscularity_web::{
        render_component_file_to_html, render_nodes_to_html, render_template_to_html,
    };
}
