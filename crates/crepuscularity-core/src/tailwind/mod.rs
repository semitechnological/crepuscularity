//! Shared Tailwind CSS v4 utility parsing (colors, spacing, sizing).
//!
//! Used by embedded, native, and other backends for consistent class semantics.

pub mod colors;
pub mod parse;

pub use colors::{lookup_color_u32, lookup_named_color};
pub use parse::{
    parse_color_rgb, parse_radius_px, parse_size_width_height, parse_spacing_px,
    parse_text_size_px, SizeToken,
};
