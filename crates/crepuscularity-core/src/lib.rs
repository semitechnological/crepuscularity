pub mod ast;
pub mod context;
pub mod eval;
pub mod parser;

pub use context::{TemplateContext, TemplateValue};
pub use parser::{
    parse_component_file, parse_template, ComponentDef, ComponentFile, ComponentMeta,
};
