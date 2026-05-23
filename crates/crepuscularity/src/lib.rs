//! Convenience facade for Crepuscularity applications.
//!
//! The backend crates remain independently usable. This crate re-exports the shared core syntax
//! APIs plus the HTML/WASM renderer for applications that want one dependency.
pub use crepuscularity_core as core;
pub use crepuscularity_core::build;
pub use crepuscularity_lvgl as lvgl;
pub use crepuscularity_native as native;
pub use crepuscularity_web as html;
pub use crepuscularity_web as web;
pub use crepuscularity_web::crepus_refs;

pub mod target;

/// Pragma line for indent `.crepus` source: load one family from Google Fonts when the site shell is built.
pub fn google_font(family: &str) -> String {
    let q = family.trim().replace('\\', r"\\").replace('"', "\\\"");
    format!("google-font \"{q}\"\n")
}

/// One line, multiple quoted families (same as `google-fonts "A" "B"` in `.crepus`).
pub fn google_fonts(families: &[&str]) -> String {
    let mut s = String::from("google-fonts");
    for f in families {
        let q = f.trim().replace('\\', r"\\").replace('"', "\\\"");
        s.push(' ');
        s.push('"');
        s.push_str(&q);
        s.push('"');
    }
    s.push('\n');
    s
}

pub mod prelude {
    pub use crepuscularity_core::{
        parse_component_file, parse_template, ComponentDef, ComponentFile, ComponentMeta,
        TemplateContext, TemplateValue,
    };
    pub use crepuscularity_web::{
        render_component_file_to_html, render_nodes_to_html, render_template_to_html,
    };
}
