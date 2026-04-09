//! Maps a subset of Tailwind-style classes onto [`ViewStyle`] and stack alignment.
//! Goal: grow toward GPUI `styler.rs` coverage; not every class is implemented yet—extend the
//! `apply_class` match as needed.

use crepuscularity_core::context::TemplateContext;

use crate::ViewStyle;

#[derive(Debug, Clone, Default)]
pub(crate) struct StackLayoutHints {
    pub style: ViewStyle,
    /// `items-*` → cross-axis alignment for flex stacks.
    pub align_items: Option<String>,
    /// `justify-*` → main-axis distribution.
    pub justify_content: Option<String>,
}

/// `spacing` is flex `gap-*` (already handled elsewhere); merge stack layout + element style.
pub(crate) fn extract_stack_hints(
    classes: &[String],
    ctx: Option<&TemplateContext>,
) -> StackLayoutHints {
    let mut hints = StackLayoutHints::default();
    for c in classes {
        apply_layout_class(&mut hints, c, ctx);
    }
    hints
}

pub(crate) fn extract_text_style(classes: &[String], ctx: Option<&TemplateContext>) -> ViewStyle {
    let mut s = ViewStyle::default();
    for c in classes {
        apply_text_class(&mut s, c, ctx);
    }
    s
}

fn spacing_u(n: u32) -> f32 {
    (n as f32) * 4.0
}

fn parse_spacing_suffix(class: &str, prefix: &str) -> Option<f32> {
    let rest = class.strip_prefix(prefix)?;
    let n: u32 = rest.parse().ok()?;
    Some(spacing_u(n))
}

fn apply_layout_class(hints: &mut StackLayoutHints, class: &str, ctx: Option<&TemplateContext>) {
    if class.starts_with("hover:") || class.starts_with("focus:") || class.starts_with("active:") {
        return;
    }

    let s = &mut hints.style;

    // Padding
    if let Some(v) = parse_spacing_suffix(class, "p-") {
        s.padding = Some(v);
        return;
    }
    if let Some(v) = parse_spacing_suffix(class, "px-") {
        s.padding_horizontal = Some(v);
        return;
    }
    if let Some(v) = parse_spacing_suffix(class, "py-") {
        s.padding_vertical = Some(v);
        return;
    }
    if let Some(v) = parse_spacing_suffix(class, "pt-") {
        s.padding_top = Some(v);
        return;
    }
    if let Some(v) = parse_spacing_suffix(class, "pb-") {
        s.padding_bottom = Some(v);
        return;
    }
    if let Some(v) = parse_spacing_suffix(class, "pl-") {
        s.padding_left = Some(v);
        return;
    }
    if let Some(v) = parse_spacing_suffix(class, "pr-") {
        s.padding_right = Some(v);
        return;
    }

    // Margin
    if let Some(v) = parse_spacing_suffix(class, "m-") {
        s.margin = Some(v);
        return;
    }
    if let Some(v) = parse_spacing_suffix(class, "mx-") {
        s.margin_horizontal = Some(v);
        return;
    }
    if let Some(v) = parse_spacing_suffix(class, "my-") {
        s.margin_vertical = Some(v);
        return;
    }
    if let Some(v) = parse_spacing_suffix(class, "mt-") {
        s.margin_top = Some(v);
        return;
    }
    if let Some(v) = parse_spacing_suffix(class, "mb-") {
        s.margin_bottom = Some(v);
        return;
    }
    if let Some(v) = parse_spacing_suffix(class, "ml-") {
        s.margin_left = Some(v);
        return;
    }
    if let Some(v) = parse_spacing_suffix(class, "mr-") {
        s.margin_right = Some(v);
        return;
    }

    // Background / border / radius on containers
    apply_color_and_radius(s, class, ctx);

    match class {
        "items-start" => hints.align_items = Some("start".into()),
        "items-end" => hints.align_items = Some("end".into()),
        "items-center" => hints.align_items = Some("center".into()),
        "items-stretch" => hints.align_items = Some("stretch".into()),
        "items-baseline" => hints.align_items = Some("baseline".into()),
        "justify-start" => hints.justify_content = Some("start".into()),
        "justify-end" => hints.justify_content = Some("end".into()),
        "justify-center" => hints.justify_content = Some("center".into()),
        "justify-between" => hints.justify_content = Some("between".into()),
        "justify-around" => hints.justify_content = Some("around".into()),
        "justify-evenly" => hints.justify_content = Some("evenly".into()),
        _ => apply_text_class(s, class, ctx),
    }
}

fn apply_text_class(s: &mut ViewStyle, class: &str, ctx: Option<&TemplateContext>) {
    if class.starts_with("hover:") || class.starts_with("focus:") || class.starts_with("active:") {
        return;
    }

    apply_color_and_radius(s, class, ctx);

    match class {
        "text-xs" => s.font_size = Some(12.0),
        "text-sm" => s.font_size = Some(14.0),
        "text-base" => s.font_size = Some(16.0),
        "text-lg" => s.font_size = Some(18.0),
        "text-xl" => s.font_size = Some(20.0),
        "text-2xl" => s.font_size = Some(24.0),
        "text-3xl" => s.font_size = Some(30.0),
        "text-4xl" => s.font_size = Some(36.0),
        "text-5xl" => s.font_size = Some(48.0),
        "text-6xl" => s.font_size = Some(60.0),
        "text-7xl" => s.font_size = Some(72.0),
        "text-8xl" => s.font_size = Some(96.0),
        "text-9xl" => s.font_size = Some(128.0),

        "font-thin" => s.font_weight = Some(100),
        "font-extralight" => s.font_weight = Some(200),
        "font-light" => s.font_weight = Some(300),
        "font-normal" => s.font_weight = Some(400),
        "font-medium" => s.font_weight = Some(500),
        "font-semibold" => s.font_weight = Some(600),
        "font-bold" => s.font_weight = Some(700),
        "font-extrabold" => s.font_weight = Some(800),
        "font-black" => s.font_weight = Some(900),

        "text-left" => s.text_align = Some("leading".into()),
        "text-center" => s.text_align = Some("center".into()),
        "text-right" => s.text_align = Some("trailing".into()),

        "italic" | "font-italic" => s.italic = Some(true),
        "not-italic" => s.italic = Some(false),

        "underline" => s.underline = Some(true),
        "line-through" => s.strikethrough = Some(true),
        _ => {}
    }
}

fn apply_color_and_radius(s: &mut ViewStyle, class: &str, ctx: Option<&TemplateContext>) {
    if let Some(ctx) = ctx {
        if class.contains('{') {
            if let Some(rest) = class.strip_prefix("bg-") {
                if let (true, expr) = parse_braced_expr(rest) {
                    let v = crepuscularity_core::eval::eval_expr(expr, ctx);
                    let color_str = crepuscularity_core::context::value_to_str(&v);
                    if let Some(hex) = parse_hex_color(&color_str) {
                        s.background_color = Some(hex);
                    }
                    return;
                }
            }
            if let Some(rest) = class.strip_prefix("text-") {
                if let (true, expr) = parse_braced_expr(rest) {
                    let v = crepuscularity_core::eval::eval_expr(expr, ctx);
                    let color_str = crepuscularity_core::context::value_to_str(&v);
                    if let Some(hex) = parse_hex_color(&color_str) {
                        s.foreground_color = Some(hex);
                    }
                    return;
                }
            }
        }
    }

    // Static named colors (aligned with common demo templates).
    match class {
        "bg-black" => s.background_color = Some("#000000".into()),
        "bg-white" => s.background_color = Some("#ffffff".into()),
        "text-white" => s.foreground_color = Some("#ffffff".into()),
        "text-black" => s.foreground_color = Some("#000000".into()),
        "bg-zinc-950" => s.background_color = Some("#09090b".into()),
        "bg-zinc-900" => s.background_color = Some("#18181b".into()),
        "bg-zinc-800" => s.background_color = Some("#27272a".into()),
        "text-zinc-100" => s.foreground_color = Some("#f4f4f5".into()),
        "text-zinc-200" => s.foreground_color = Some("#e4e4e7".into()),
        "text-zinc-400" => s.foreground_color = Some("#a1a1aa".into()),
        "text-zinc-500" => s.foreground_color = Some("#71717a".into()),
        "text-green-400" => s.foreground_color = Some("#4ade80".into()),
        "text-red-400" => s.foreground_color = Some("#f87171".into()),
        "text-yellow-400" => s.foreground_color = Some("#facc15".into()),
        "bg-red-500" => s.background_color = Some("#ef4444".into()),
        "bg-green-500" => s.background_color = Some("#22c55e".into()),
        "bg-blue-500" => s.background_color = Some("#3b82f6".into()),
        _ => {}
    }

    // Arbitrary hex: bg-[#0f0f0f]
    if let Some(rest) = class.strip_prefix("bg-[") {
        if let Some(hex) = rest.strip_suffix(']') {
            if let Some(h) = parse_hex_color(hex) {
                s.background_color = Some(h);
            }
        }
    }
    if let Some(rest) = class.strip_prefix("text-[") {
        if let Some(hex) = rest.strip_suffix(']') {
            if let Some(h) = parse_hex_color(hex) {
                s.foreground_color = Some(h);
            }
        }
    }

    // Border radius (logical points, not necessarily matching GPUI px exactly).
    match class {
        "rounded-none" => s.corner_radius = Some(0.0),
        "rounded-sm" => s.corner_radius = Some(2.0),
        "rounded" | "rounded-md" => s.corner_radius = Some(6.0),
        "rounded-lg" => s.corner_radius = Some(8.0),
        "rounded-xl" => s.corner_radius = Some(12.0),
        "rounded-2xl" => s.corner_radius = Some(16.0),
        "rounded-3xl" => s.corner_radius = Some(24.0),
        "rounded-full" => s.corner_radius = Some(9999.0),
        _ => {}
    }
}

fn parse_braced_expr(s: &str) -> (bool, &str) {
    if s.starts_with('{') && s.ends_with('}') {
        return (true, s[1..s.len() - 1].trim());
    }
    (false, "")
}

fn parse_hex_color(s: &str) -> Option<String> {
    let t = s.trim();
    let hex = t
        .strip_prefix('#')
        .or_else(|| t.strip_prefix("0x"))
        .unwrap_or(t);
    if hex.len() == 6 && hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return Some(format!("#{hex}"));
    }
    if hex.len() == 8 && hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return Some(format!("#{hex}"));
    }
    None
}

pub(crate) fn is_scroll_container(classes: &[String]) -> bool {
    classes.iter().any(|c| {
        matches!(
            c.as_str(),
            "overflow-scroll"
                | "overflow-y-scroll"
                | "overflow-y-auto"
                | "overflow-x-scroll"
                | "overflow-x-auto"
        )
    })
}
