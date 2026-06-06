//! JSX / HTML tag syntax parser.
//!
//! Activated automatically when `parse_template()` detects JSX mode (first content
//! line starts with `<`). Produces the same `Node`/`Element` AST as the indentation
//! parser, so every backend — GPUI, web, webext — works unchanged.

mod attrs;
mod builders;
mod tags;
mod template;
mod text;

pub(crate) use template::parse_jsx_template;
