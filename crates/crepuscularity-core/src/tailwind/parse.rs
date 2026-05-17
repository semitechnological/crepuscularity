//! Tailwind spacing / sizing / color token parsing (backend-agnostic).

use super::colors::lookup_named_color;

/// Tailwind spacing scale: `1` → 4 px unless arbitrary `[Npx]`.
pub fn parse_spacing_px(rest: &str) -> Option<u16> {
    if let Some(inner) = rest.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
        let stripped = inner.strip_suffix("px").unwrap_or(inner);
        return stripped.parse::<u16>().ok();
    }
    match rest {
        "px" => Some(1),
        "0" => Some(0),
        "0.5" => Some(2),
        "1" => Some(4),
        "1.5" => Some(6),
        "2" => Some(8),
        "2.5" => Some(10),
        "3" => Some(12),
        "3.5" => Some(14),
        "4" => Some(16),
        "5" => Some(20),
        "6" => Some(24),
        "7" => Some(28),
        "8" => Some(32),
        "9" => Some(36),
        "10" => Some(40),
        "11" => Some(44),
        "12" => Some(48),
        "14" => Some(56),
        "16" => Some(64),
        "20" => Some(80),
        "24" => Some(96),
        "32" => Some(128),
        "36" => Some(144),
        "40" => Some(160),
        "44" => Some(176),
        "48" => Some(192),
        "52" => Some(208),
        "56" => Some(224),
        "60" => Some(240),
        "64" => Some(256),
        "72" => Some(288),
        "80" => Some(320),
        "96" => Some(384),
        _ => rest.parse::<u16>().ok().map(|n| n.saturating_mul(4)),
    }
}

/// Width/height token after `w-` / `h-` prefix.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SizeToken {
    Full,
    Auto,
    Px(u16),
    Fraction { num: u16, den: u16 },
    Spacing(u16),
}

pub fn parse_size_width_height(rest: &str) -> Option<SizeToken> {
    match rest {
        "full" | "screen" => return Some(SizeToken::Full),
        "auto" | "fit" | "min" | "max" => return Some(SizeToken::Auto),
        "px" => return Some(SizeToken::Px(1)),
        _ => {}
    }
    if let Some((num, den)) = rest.split_once('/') {
        if let (Ok(n), Ok(d)) = (num.parse::<u16>(), den.parse::<u16>()) {
            if d > 0 {
                return Some(SizeToken::Fraction { num: n, den: d });
            }
        }
    }
    if let Some(inner) = rest.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
        let stripped = inner.strip_suffix("px").unwrap_or(inner);
        if let Ok(n) = stripped.parse::<u16>() {
            return Some(SizeToken::Px(n));
        }
    }
    parse_spacing_px(rest).map(SizeToken::Spacing)
}

/// Resolve `red-500`, `#fff`, `bg-[#0f0]`, or `red-500/50` to RGB bytes.
pub fn parse_color_rgb(name: &str) -> Option<[u8; 3]> {
    let name = name.trim();
    if let Some((color_part, _opacity)) = name.split_once('/') {
        return parse_color_rgb(color_part);
    }
    if let Some(inner) = name
        .strip_prefix('[')
        .and_then(|s| s.strip_suffix(']'))
    {
        return parse_color_rgb(inner);
    }
    if let Some(hex) = lookup_named_color(name) {
        return parse_hex_rgb(hex);
    }
    if name.starts_with('#') || name.starts_with("0x") {
        return parse_hex_rgb(name);
    }
    None
}

fn parse_hex_rgb(s: &str) -> Option<[u8; 3]> {
    let t = s.trim().trim_start_matches('#').trim_start_matches("0x");
    if t.len() < 6 {
        return None;
    }
    let t = &t[..6];
    Some([
        u8::from_str_radix(&t[0..2], 16).ok()?,
        u8::from_str_radix(&t[2..4], 16).ok()?,
        u8::from_str_radix(&t[4..6], 16).ok()?,
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spacing_scale() {
        assert_eq!(parse_spacing_px("4"), Some(16));
        assert_eq!(parse_spacing_px("[20px]"), Some(20));
    }

    #[test]
    fn tailwind_v4_color() {
        let rgb = parse_color_rgb("zinc-900").unwrap();
        assert_eq!(rgb, [0x18, 0x18, 0x1b]);
        assert_eq!(parse_color_rgb("red-500").unwrap(), [0xfb, 0x2c, 0x36]);
    }
}
