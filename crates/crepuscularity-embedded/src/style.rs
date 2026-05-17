//! Tailwind utility classes → [`crate::document::EmbeddedStyle`].

#[cfg(not(feature = "std"))]
use alloc::string::String;
#[cfg(feature = "std")]
use std::string::String;

use crate::document::EmbeddedStyle;
use crate::tailwind_apply;

#[cfg(feature = "std")]
use crepuscularity_core::context::TemplateContext;

pub fn parse_classes(classes: &[String]) -> EmbeddedStyle {
    style_from_classes_with_context(classes, None)
}

pub fn style_from_classes(classes: &[String]) -> EmbeddedStyle {
    parse_classes(classes)
}

#[cfg(feature = "std")]
pub fn style_from_classes_with_context(
    classes: &[String],
    ctx: Option<&TemplateContext>,
) -> EmbeddedStyle {
    let mut s = EmbeddedStyle::default();
    tailwind_apply::apply_classes(classes, &mut s, ctx);
    s
}

#[cfg(not(feature = "std"))]
pub fn style_from_classes_with_context(classes: &[String], _ctx: Option<&()>) -> EmbeddedStyle {
    let mut s = EmbeddedStyle::default();
    tailwind_apply::apply_classes(classes, &mut s, None);
    s
}
