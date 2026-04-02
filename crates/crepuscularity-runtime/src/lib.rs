pub mod hot_reload;
pub mod renderer;
pub mod styler;
pub mod watcher;

pub use crepuscularity_core as core;
pub use crepuscularity_core::ast;
pub use crepuscularity_core::context;
pub use crepuscularity_core::eval;
pub use crepuscularity_core::parser;
pub use crepuscularity_core::{TemplateContext, TemplateValue};
pub use hot_reload::{HotReloadState, HotReloadView};
pub use crepuscularity_core::parser::{
    parse_component_file, parse_template, ComponentDef, ComponentFile, ComponentMeta,
};
pub use renderer::render_nodes;
