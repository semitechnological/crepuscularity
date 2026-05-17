//! Full Tailwind CSS v4 palette (shared with `crepuscularity-core`).

#[path = "../../crepuscularity-core/src/tailwind/colors.rs"]
#[allow(dead_code)]
mod colors_impl;

pub use colors_impl::lookup_named_color;
