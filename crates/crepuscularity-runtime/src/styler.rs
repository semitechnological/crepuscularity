/// Runtime Tailwind class → GPUI style applicator.
///
/// Supports:
/// - Static Tailwind classes: `flex`, `bg-red-500`, `p-4`
/// - Arbitrary values: `bg-[#ff5733]`, `w-[200px]`, `text-[14px]`
/// - Dynamic context expressions: `bg-{theme.surface}`, `text-{colors.muted}`
///   where the expression evaluates to a hex color string like "#1e1e2e" or "1e1e2e"

use gpui::{prelude::*, rgb, rgba, px, rems, relative, Div, Length, AbsoluteLength, DefiniteLength};

use crate::context::TemplateContext;

/// Apply a class to a div, optionally resolving `{expr}` placeholders against context.
pub fn apply_class(d: Div, class: &str) -> Div {
    apply_class_with_ctx(d, class, None)
}

/// Apply a class with optional template context for dynamic value resolution.
pub fn apply_class_with_ctx(d: Div, class: &str, ctx: Option<&TemplateContext>) -> Div {
    // State prefixes — skip silently in runtime renderer (no hover/focus state at runtime)
    if class.starts_with("hover:")
        || class.starts_with("focus:")
        || class.starts_with("active:")
    {
        return d;
    }

    // Check for context-expression classes: bg-{expr}, text-{expr}, border-{expr}
    if let Some(ctx) = ctx {
        if class.contains('{') {
            return apply_context_class(d, class, ctx);
        }
    }

    apply_base_class(d, class)
}

pub fn apply_base_class(d: Div, class: &str) -> Div {
    match apply_static(d, class) {
        Ok(d) => d,
        Err(d) => apply_dynamic(d, class),
    }
}

/// Try to resolve a class containing `{expr}` against the template context.
/// Falls through to `apply_base_class` if no context expression matched.
fn apply_context_class(d: Div, class: &str, ctx: &TemplateContext) -> Div {
    // Pattern: prefix-{expr} where prefix is bg, text, border, etc.
    for (prefix, apply_fn) in &[
        ("bg-", apply_bg as fn(Div, u32) -> Div),
        ("text-", apply_text_color as fn(Div, u32) -> Div),
        ("border-", apply_border_color as fn(Div, u32) -> Div),
    ] {
        if let Some(rest) = class.strip_prefix(prefix) {
            if rest.starts_with('{') && rest.ends_with('}') {
                let expr = &rest[1..rest.len() - 1];
                let val = crate::eval::eval_expr(expr, ctx);
                let color_str = crate::context::value_to_str(&val);
                if let Some(hex) = parse_color_str(&color_str) {
                    return apply_fn(d, hex);
                }
            }
        }
    }

    // bg-{expr}/alpha pattern for RGBA: bg-{expr}/20
    if let Some(rest) = class.strip_prefix("bg-") {
        if let Some((expr_part, alpha_str)) = rest.rsplit_once('/') {
            if expr_part.starts_with('{') && expr_part.ends_with('}') {
                let expr = &expr_part[1..expr_part.len() - 1];
                let val = crate::eval::eval_expr(expr, ctx);
                let color_str = crate::context::value_to_str(&val);
                if let Some(hex) = parse_color_str(&color_str) {
                    if let Ok(alpha) = alpha_str.parse::<u32>() {
                        let alpha_byte = (alpha * 255 / 100) as u32;
                        let rgba_val = (hex << 8) | alpha_byte;
                        return d.bg(rgba(rgba_val));
                    }
                }
            }
        }
    }

    // opacity-{expr}
    if let Some(rest) = class.strip_prefix("opacity-") {
        if rest.starts_with('{') && rest.ends_with('}') {
            let expr = &rest[1..rest.len() - 1];
            let val = crate::eval::eval_expr(expr, ctx);
            let s = crate::context::value_to_str(&val);
            if let Ok(n) = s.parse::<f32>() {
                return d.opacity(n / 100.0);
            }
        }
    }

    // No context expression matched — fall through to normal class processing
    apply_base_class(d, class)
}

/// Parse a color string like "#1e1e2e", "1e1e2e", or "0x1e1e2e" to a u32 hex value.
fn parse_color_str(s: &str) -> Option<u32> {
    let hex_str = s.strip_prefix('#')
        .or_else(|| s.strip_prefix("0x"))
        .unwrap_or(s);
    u32::from_str_radix(hex_str, 16).ok()
}

/// Returns Ok(div) if class matched, Err(div) to fall through to dynamic.
fn apply_static(d: Div, class: &str) -> Result<Div, Div> {
    Ok(match class {
        // Layout
        "flex" => d.flex(),
        "flex-col" => d.flex_col(),
        "flex-row" => d.flex_row(),
        "flex-wrap" => d.flex_wrap(),
        "flex-nowrap" => d.flex_nowrap(),
        "flex-1" => d.flex_1(),
        "flex-auto" => d.flex_auto(),
        "flex-initial" => d.flex_initial(),
        "flex-none" => d.flex_none(),
        "grow" => d.flex_grow(),
        "grow-0" => d.flex_none(),
        "shrink" => d.flex_shrink(),
        "shrink-0" => d.flex_shrink_0(),
        "block" => d.block(),
        "hidden" => d.hidden(),
        "invisible" => d.invisible(),
        "visible" => d.visible(),
        "absolute" => d.absolute(),
        "relative" => d.relative(),
        "overflow-hidden" => d.overflow_hidden(),
        "overflow-x-hidden" => d.overflow_x_hidden(),
        "overflow-y-hidden" => d.overflow_y_hidden(),
        // overflow-auto/scroll — scroll not available on Div (needs StatefulInteractiveElement)
        "overflow-scroll" | "overflow-auto"
        | "overflow-y-scroll" | "overflow-y-auto"
        | "overflow-x-scroll" | "overflow-x-auto" => d,

        // Sizing
        "w-full" => d.w_full(),
        "h-full" => d.h_full(),
        "w-auto" => d.w_auto(),
        "h-auto" => d.h_auto(),
        "w-px" => d.w_px(),
        "h-px" => d.h_px(),
        "min-w-0" => d.min_w(px(0.)),
        "min-h-0" => d.min_h(px(0.)),
        "w-0" => d.w(px(0.)),
        "h-0" => d.h(px(0.)),

        // Flexbox alignment
        "justify-center" => d.justify_center(),
        "justify-start" => d.justify_start(),
        "justify-end" => d.justify_end(),
        "justify-between" => d.justify_between(),
        "justify-around" => d.justify_around(),
        "items-center" => d.items_center(),
        "items-start" => d.items_start(),
        "items-end" => d.items_end(),
        "items-baseline" => d.items_baseline(),
        // self-* not supported in GPUI 0.2.x
        "self-stretch" | "self-center" | "self-start" | "self-end" | "self-auto" => d,

        // Colors — background
        "bg-black" => d.bg(gpui::black()),
        "bg-white" => d.bg(gpui::white()),
        "bg-transparent" => d.bg(gpui::transparent_black()),

        // Colors — text
        "text-white" => d.text_color(gpui::white()),
        "text-black" => d.text_color(gpui::black()),

        // Colors — border
        "border-white" => d.border_color(gpui::white()),
        "border-black" => d.border_color(gpui::black()),

        // Border width
        "border" => d.border_1(),
        "border-0" => d.border_0(),
        "border-2" => d.border_2(),
        "border-4" => d.border_4(),
        "border-8" => d.border_8(),
        "border-t" => d.border_t_1(),
        "border-b" => d.border_b_1(),
        "border-l" => d.border_l_1(),
        "border-r" => d.border_r_1(),

        // Border radius
        "rounded-none" => d.rounded_none(),
        "rounded-sm" => d.rounded_sm(),
        "rounded" | "rounded-md" => d.rounded_md(),
        "rounded-lg" => d.rounded_lg(),
        "rounded-xl" => d.rounded_xl(),
        "rounded-2xl" => d.rounded_2xl(),
        "rounded-3xl" => d.rounded_3xl(),
        "rounded-full" => d.rounded_full(),

        // Typography — weight
        "font-thin" => d.font_weight(gpui::FontWeight::THIN),
        "font-light" => d.font_weight(gpui::FontWeight::LIGHT),
        "font-normal" => d.font_weight(gpui::FontWeight::NORMAL),
        "font-medium" => d.font_weight(gpui::FontWeight::MEDIUM),
        "font-semibold" => d.font_weight(gpui::FontWeight::SEMIBOLD),
        "font-bold" => d.font_weight(gpui::FontWeight::BOLD),
        "font-extrabold" => d.font_weight(gpui::FontWeight::EXTRA_BOLD),
        "font-black" => d.font_weight(gpui::FontWeight::BLACK),
        "italic" | "font-italic" => d.italic(),

        // Typography — scale
        "text-xs" => d.text_size(rems(0.75)),
        "text-sm" => d.text_size(rems(0.875)),
        "text-base" => d.text_size(rems(1.)),
        "text-lg" => d.text_size(rems(1.125)),
        "text-xl" => d.text_size(rems(1.25)),
        "text-2xl" => d.text_size(rems(1.5)),
        "text-3xl" => d.text_size(rems(1.875)),
        "text-4xl" => d.text_size(rems(2.25)),
        "text-5xl" => d.text_size(rems(3.)),
        "text-6xl" => d.text_size(rems(3.75)),
        "text-7xl" => d.text_size(rems(4.5)),
        "text-8xl" => d.text_size(rems(6.)),
        "text-9xl" => d.text_size(rems(8.)),

        // Typography — alignment
        "text-left" => d.text_left(),
        "text-center" => d.text_center(),
        "text-right" => d.text_right(),

        // Typography — line height
        "leading-none" => d.line_height(relative(1.)),
        "leading-tight" => d.line_height(relative(1.25)),
        "leading-snug" => d.line_height(relative(1.375)),
        "leading-normal" => d.line_height(relative(1.5)),
        "leading-relaxed" => d.line_height(relative(1.625)),
        "leading-loose" => d.line_height(relative(2.)),

        // Typography — misc
        "truncate" => d.truncate(),
        "whitespace-nowrap" => d.whitespace_nowrap(),
        "whitespace-normal" => d.whitespace_normal(),
        "underline" => d.underline(),
        "line-through" => d.line_through(),

        // Cursor
        "cursor-pointer" => d.cursor_pointer(),
        "cursor-default" => d.cursor_default(),
        "cursor-text" => d.cursor_text(),
        "cursor-not-allowed" => d.cursor_not_allowed(),
        "cursor-crosshair" => d.cursor_crosshair(),
        "cursor-grab" => d.cursor_grab(),
        "cursor-grabbing" => d.cursor_grabbing(),
        "cursor-col-resize" => d.cursor_col_resize(),
        "cursor-row-resize" => d.cursor_row_resize(),

        // Shadow
        "shadow-sm" => d.shadow_sm(),
        "shadow" | "shadow-md" => d.shadow_md(),
        "shadow-lg" => d.shadow_lg(),
        "shadow-xl" => d.shadow_xl(),
        "shadow-2xl" => d.shadow_2xl(),
        "shadow-none" => d,

        // Transitions & animations — parsed but no-op at the class level
        // Actual animation is handled by animate: attributes in the renderer.
        "transition" | "transition-all" | "transition-colors" | "transition-opacity"
        | "transition-shadow" | "transition-transform"
        | "duration-75" | "duration-100" | "duration-150" | "duration-200"
        | "duration-300" | "duration-500" | "duration-700" | "duration-1000"
        | "ease-linear" | "ease-in" | "ease-out" | "ease-in-out"
        | "delay-75" | "delay-100" | "delay-150" | "delay-200"
        | "delay-300" | "delay-500" | "delay-700" | "delay-1000"
        | "animate-none" | "animate-spin" | "animate-ping" | "animate-pulse" | "animate-bounce" => d,

        // Ignored / unsupported (accepted silently to allow copy-pasting from Tailwind)
        "inline-flex" | "inline" | "grid" | "select-none" | "pointer-events-none"
        | "uppercase" | "lowercase" | "capitalize" | "whitespace-pre" | "sticky"
        | "fixed" | "overflow-scroll-x" | "content-center" | "content-start"
        | "content-end" | "items-stretch" => d,

        _ => return Err(d),
    })
}

fn apply_dynamic(d: Div, class: &str) -> Div {
    // Methods that accept Length (auto allowed: w, h, m*, inset, top, etc.)
    const LENGTH_PROPS: &[(&str, fn(Div, Length) -> Div)] = &[
        ("w-", |d, v| d.w(v)),
        ("h-", |d, v| d.h(v)),
        ("min-w-", |d, v| d.min_w(v)),
        ("min-h-", |d, v| d.min_h(v)),
        ("max-w-", |d, v| d.max_w(v)),
        ("max-h-", |d, v| d.max_h(v)),
        ("m-", |d, v| d.m(v)),
        ("mx-", |d, v| d.mx(v)),
        ("my-", |d, v| d.my(v)),
        ("mt-", |d, v| d.mt(v)),
        ("mb-", |d, v| d.mb(v)),
        ("ml-", |d, v| d.ml(v)),
        ("mr-", |d, v| d.mr(v)),
        ("top-", |d, v| d.top(v)),
        ("bottom-", |d, v| d.bottom(v)),
        ("left-", |d, v| d.left(v)),
        ("right-", |d, v| d.right(v)),
        ("inset-", |d, v| d.inset(v)),
        ("size-", |d, v| d.size(v)),
    ];

    for (prefix, apply) in LENGTH_PROPS {
        if let Some(rest) = class.strip_prefix(prefix) {
            if let Some(len) = parse_length(rest) {
                return apply(d, len);
            }
        }
    }

    // Methods that accept DefiniteLength (auto not allowed: p*, gap*)
    const DEFINITE_PROPS: &[(&str, fn(Div, DefiniteLength) -> Div)] = &[
        ("p-", |d, v| d.p(v)),
        ("px-", |d, v| d.px(v)),
        ("py-", |d, v| d.py(v)),
        ("pt-", |d, v| d.pt(v)),
        ("pb-", |d, v| d.pb(v)),
        ("pl-", |d, v| d.pl(v)),
        ("pr-", |d, v| d.pr(v)),
        ("gap-", |d, v| d.gap(v)),
        ("gap-x-", |d, v| d.gap_x(v)),
        ("gap-y-", |d, v| d.gap_y(v)),
    ];

    for (prefix, apply) in DEFINITE_PROPS {
        if let Some(rest) = class.strip_prefix(prefix) {
            if let Some(len) = parse_definite_length(rest) {
                return apply(d, len);
            }
        }
    }

    // Arbitrary border radius: rounded-[Npx]
    if let Some(rest) = class.strip_prefix("rounded-[") {
        if let Some(inner) = rest.strip_suffix(']') {
            if let Some(abs) = parse_absolute_length(inner) {
                return d.rounded(abs);
            }
        }
    }

    // Arbitrary border widths: border-t-[Npx], border-b-[Npx], etc.
    for (prefix, apply_fn) in &[
        ("border-t-", Div::border_t as fn(Div, AbsoluteLength) -> Div),
        ("border-b-", Div::border_b as fn(Div, AbsoluteLength) -> Div),
        ("border-l-", Div::border_l as fn(Div, AbsoluteLength) -> Div),
        ("border-r-", Div::border_r as fn(Div, AbsoluteLength) -> Div),
    ] {
        if let Some(rest) = class.strip_prefix(prefix) {
            if rest.starts_with('[') && rest.ends_with(']') {
                let inner = &rest[1..rest.len()-1];
                if let Some(abs) = parse_absolute_length(inner) {
                    return apply_fn(d, abs);
                }
            }
        }
    }

    // font-['Family'] or font-[Family]
    if let Some(rest) = class.strip_prefix("font-[") {
        if let Some(inner) = rest.strip_suffix(']') {
            let family = inner.trim_matches('\'').trim_matches('"').replace('_', " ");
            return d.font_family(family);
        }
    }

    // text-[size] — text_size takes AbsoluteLength
    if let Some(rest) = class.strip_prefix("text-[") {
        if let Some(inner) = rest.strip_suffix(']') {
            if let Some(len) = parse_absolute_length(inner) {
                return d.text_size(len);
            }
        }
    }

    // line-height-[value] or leading-[value]
    if let Some(rest) = class.strip_prefix("leading-[") {
        if let Some(inner) = rest.strip_suffix(']') {
            if let Some(abs) = parse_absolute_length(inner) {
                return d.line_height(abs);
            }
        }
    }

    // opacity-N
    if let Some(rest) = class.strip_prefix("opacity-") {
        if let Ok(n) = rest.parse::<f32>() {
            return d.opacity(n / 100.0);
        }
    }

    // Color families: bg-{family}-{shade}, text-{family}-{shade}, border-{family}-{shade}
    for (prefix, apply_fn) in &[
        ("bg-", apply_bg as fn(Div, u32) -> Div),
        ("text-", apply_text_color as fn(Div, u32) -> Div),
        ("border-", apply_border_color as fn(Div, u32) -> Div),
    ] {
        if let Some(rest) = class.strip_prefix(prefix) {
            // Arbitrary hex: bg-[#rrggbb] or bg-[#rrggbbaa]
            if rest.starts_with('[') && rest.ends_with(']') {
                let inner = &rest[1..rest.len() - 1];
                if let Some(hex_str) = inner.strip_prefix('#') {
                    if hex_str.len() == 8 {
                        // 8-digit hex: RRGGBBAA
                        if let Ok(hex) = u32::from_str_radix(hex_str, 16) {
                            return d.bg(rgba(hex));
                        }
                    }
                    if let Ok(hex) = u32::from_str_radix(hex_str, 16) {
                        return apply_fn(d, hex);
                    }
                }
            }
            // bg-family/alpha: bg-red-500/50
            if let Some(slash_pos) = rest.rfind('/') {
                let color_part = &rest[..slash_pos];
                let alpha_str = &rest[slash_pos + 1..];
                if let Ok(alpha_pct) = alpha_str.parse::<u32>() {
                    if let Some(dash_pos) = color_part.rfind('-') {
                        let family = &color_part[..dash_pos];
                        let shade = &color_part[dash_pos + 1..];
                        if let Some(hex) = tailwind_color(family, shade) {
                            let alpha_byte = (alpha_pct * 255 / 100) as u32;
                            let rgba_val = (hex << 8) | alpha_byte;
                            return d.bg(rgba(rgba_val));
                        }
                    }
                }
            }
            // Named: family-shade
            if let Some(dash_pos) = rest.rfind('-') {
                let family = &rest[..dash_pos];
                let shade = &rest[dash_pos + 1..];
                if let Some(hex) = tailwind_color(family, shade) {
                    return apply_fn(d, hex);
                }
            }
        }
    }

    d
}

fn apply_bg(d: Div, hex: u32) -> Div { d.bg(rgb(hex)) }
fn apply_text_color(d: Div, hex: u32) -> Div { d.text_color(rgb(hex)) }
fn apply_border_color(d: Div, hex: u32) -> Div { d.border_color(rgb(hex)) }

/// Parse a Tailwind length token into a `Length` (supports auto, full, %, px, rem, number)
fn parse_length(value: &str) -> Option<Length> {
    if value.starts_with('[') && value.ends_with(']') {
        let inner = &value[1..value.len() - 1];
        if let Some(abs) = parse_absolute_length(inner) {
            return Some(abs.into());
        }
        if let Some(rest) = inner.strip_suffix('%') {
            if let Ok(n) = rest.parse::<f32>() { return Some(relative(n / 100.0).into()); }
        }
        return None;
    }
    match value {
        "full" | "screen" => return Some(relative(1.).into()),
        "auto" => return Some(Length::Auto),
        "px" => return Some(px(1.).into()),
        "1/2" => return Some(relative(0.5).into()),
        "1/3" => return Some(relative(1.0/3.0).into()),
        "2/3" => return Some(relative(2.0/3.0).into()),
        "1/4" => return Some(relative(0.25).into()),
        "3/4" => return Some(relative(0.75).into()),
        _ => {}
    }
    if let Ok(n) = value.parse::<f32>() {
        return Some(rems(n * 0.25).into());
    }
    None
}

/// Parse a Tailwind length token into a `DefiniteLength` (no auto, no %)
fn parse_definite_length(value: &str) -> Option<DefiniteLength> {
    if value.starts_with('[') && value.ends_with(']') {
        return parse_absolute_length(&value[1..value.len() - 1]).map(Into::into);
    }
    match value {
        "px" => return Some(px(1.).into()),
        _ => {}
    }
    if let Ok(n) = value.parse::<f32>() {
        return Some(rems(n * 0.25).into());
    }
    None
}

/// Parse a CSS size string to `AbsoluteLength` (px or rem only)
pub fn parse_absolute_length(inner: &str) -> Option<AbsoluteLength> {
    if let Some(rest) = inner.strip_suffix("px") {
        if let Ok(n) = rest.parse::<f32>() { return Some(px(n).into()); }
    }
    if let Some(rest) = inner.strip_suffix("rem") {
        if let Ok(n) = rest.parse::<f32>() { return Some(rems(n).into()); }
    }
    if let Ok(n) = inner.parse::<f32>() { return Some(px(n).into()); }
    None
}

/// Parse a duration string like "300ms", "1s", "0.5s" into milliseconds.
pub fn parse_duration_ms(s: &str) -> Option<u64> {
    if let Some(rest) = s.strip_suffix("ms") {
        return rest.parse::<u64>().ok();
    }
    if let Some(rest) = s.strip_suffix('s') {
        return rest.parse::<f64>().ok().map(|v| (v * 1000.0) as u64);
    }
    s.parse::<u64>().ok()
}

// ============================================================
// Full Tailwind color palette
// ============================================================

pub fn tailwind_color(family: &str, shade: &str) -> Option<u32> {
    match (family, shade) {
        ("slate","50")=>Some(0xf8fafc),("slate","100")=>Some(0xf1f5f9),
        ("slate","200")=>Some(0xe2e8f0),("slate","300")=>Some(0xcbd5e1),
        ("slate","400")=>Some(0x94a3b8),("slate","500")=>Some(0x64748b),
        ("slate","600")=>Some(0x475569),("slate","700")=>Some(0x334155),
        ("slate","800")=>Some(0x1e293b),("slate","900")=>Some(0x0f172a),
        ("slate","950")=>Some(0x020617),
        ("gray","50")=>Some(0xf9fafb),("gray","100")=>Some(0xf3f4f6),
        ("gray","200")=>Some(0xe5e7eb),("gray","300")=>Some(0xd1d5db),
        ("gray","400")=>Some(0x9ca3af),("gray","500")=>Some(0x6b7280),
        ("gray","600")=>Some(0x4b5563),("gray","700")=>Some(0x374151),
        ("gray","800")=>Some(0x1f2937),("gray","900")=>Some(0x111827),
        ("gray","950")=>Some(0x030712),
        ("zinc","50")=>Some(0xfafafa),("zinc","100")=>Some(0xf4f4f5),
        ("zinc","200")=>Some(0xe4e4e7),("zinc","300")=>Some(0xd4d4d8),
        ("zinc","400")=>Some(0xa1a1aa),("zinc","500")=>Some(0x71717a),
        ("zinc","600")=>Some(0x52525b),("zinc","700")=>Some(0x3f3f46),
        ("zinc","800")=>Some(0x27272a),("zinc","900")=>Some(0x18181b),
        ("zinc","950")=>Some(0x09090b),
        ("neutral","50")=>Some(0xfafafa),("neutral","100")=>Some(0xf5f5f5),
        ("neutral","200")=>Some(0xe5e5e5),("neutral","300")=>Some(0xd4d4d4),
        ("neutral","400")=>Some(0xa3a3a3),("neutral","500")=>Some(0x737373),
        ("neutral","600")=>Some(0x525252),("neutral","700")=>Some(0x404040),
        ("neutral","800")=>Some(0x262626),("neutral","900")=>Some(0x171717),
        ("neutral","950")=>Some(0x0a0a0a),
        ("stone","50")=>Some(0xfafaf9),("stone","100")=>Some(0xf5f5f4),
        ("stone","200")=>Some(0xe7e5e4),("stone","300")=>Some(0xd6d3d1),
        ("stone","400")=>Some(0xa8a29e),("stone","500")=>Some(0x78716c),
        ("stone","600")=>Some(0x57534e),("stone","700")=>Some(0x44403c),
        ("stone","800")=>Some(0x292524),("stone","900")=>Some(0x1c1917),
        ("stone","950")=>Some(0x0c0a09),
        ("red","50")=>Some(0xfef2f2),("red","100")=>Some(0xfee2e2),
        ("red","200")=>Some(0xfecaca),("red","300")=>Some(0xfca5a5),
        ("red","400")=>Some(0xf87171),("red","500")=>Some(0xef4444),
        ("red","600")=>Some(0xdc2626),("red","700")=>Some(0xb91c1c),
        ("red","800")=>Some(0x991b1b),("red","900")=>Some(0x7f1d1d),
        ("red","950")=>Some(0x450a0a),
        ("orange","50")=>Some(0xfff7ed),("orange","100")=>Some(0xffedd5),
        ("orange","200")=>Some(0xfed7aa),("orange","300")=>Some(0xfdba74),
        ("orange","400")=>Some(0xfb923c),("orange","500")=>Some(0xf97316),
        ("orange","600")=>Some(0xea580c),("orange","700")=>Some(0xc2410c),
        ("orange","800")=>Some(0x9a3412),("orange","900")=>Some(0x7c2d12),
        ("orange","950")=>Some(0x431407),
        ("amber","50")=>Some(0xfffbeb),("amber","100")=>Some(0xfef3c7),
        ("amber","200")=>Some(0xfde68a),("amber","300")=>Some(0xfcd34d),
        ("amber","400")=>Some(0xfbbf24),("amber","500")=>Some(0xf59e0b),
        ("amber","600")=>Some(0xd97706),("amber","700")=>Some(0xb45309),
        ("amber","800")=>Some(0x92400e),("amber","900")=>Some(0x78350f),
        ("amber","950")=>Some(0x451a03),
        ("yellow","50")=>Some(0xfefce8),("yellow","100")=>Some(0xfef9c3),
        ("yellow","200")=>Some(0xfef08a),("yellow","300")=>Some(0xfde047),
        ("yellow","400")=>Some(0xfacc15),("yellow","500")=>Some(0xeab308),
        ("yellow","600")=>Some(0xca8a04),("yellow","700")=>Some(0xa16207),
        ("yellow","800")=>Some(0x854d0e),("yellow","900")=>Some(0x713f12),
        ("yellow","950")=>Some(0x422006),
        ("lime","50")=>Some(0xf7fee7),("lime","100")=>Some(0xecfccb),
        ("lime","200")=>Some(0xd9f99d),("lime","300")=>Some(0xbef264),
        ("lime","400")=>Some(0xa3e635),("lime","500")=>Some(0x84cc16),
        ("lime","600")=>Some(0x65a30d),("lime","700")=>Some(0x4d7c0f),
        ("lime","800")=>Some(0x3f6212),("lime","900")=>Some(0x365314),
        ("lime","950")=>Some(0x1a2e05),
        ("green","50")=>Some(0xf0fdf4),("green","100")=>Some(0xdcfce7),
        ("green","200")=>Some(0xbbf7d0),("green","300")=>Some(0x86efac),
        ("green","400")=>Some(0x4ade80),("green","500")=>Some(0x22c55e),
        ("green","600")=>Some(0x16a34a),("green","700")=>Some(0x15803d),
        ("green","800")=>Some(0x166534),("green","900")=>Some(0x14532d),
        ("green","950")=>Some(0x052e16),
        ("emerald","50")=>Some(0xecfdf5),("emerald","100")=>Some(0xd1fae5),
        ("emerald","200")=>Some(0xa7f3d0),("emerald","300")=>Some(0x6ee7b7),
        ("emerald","400")=>Some(0x34d399),("emerald","500")=>Some(0x10b981),
        ("emerald","600")=>Some(0x059669),("emerald","700")=>Some(0x047857),
        ("emerald","800")=>Some(0x065f46),("emerald","900")=>Some(0x064e3b),
        ("emerald","950")=>Some(0x022c22),
        ("teal","50")=>Some(0xf0fdfa),("teal","100")=>Some(0xccfbf1),
        ("teal","200")=>Some(0x99f6e4),("teal","300")=>Some(0x5eead4),
        ("teal","400")=>Some(0x2dd4bf),("teal","500")=>Some(0x14b8a6),
        ("teal","600")=>Some(0x0d9488),("teal","700")=>Some(0x0f766e),
        ("teal","800")=>Some(0x115e59),("teal","900")=>Some(0x134e4a),
        ("teal","950")=>Some(0x042f2e),
        ("cyan","50")=>Some(0xecfeff),("cyan","100")=>Some(0xcffafe),
        ("cyan","200")=>Some(0xa5f3fc),("cyan","300")=>Some(0x67e8f9),
        ("cyan","400")=>Some(0x22d3ee),("cyan","500")=>Some(0x06b6d4),
        ("cyan","600")=>Some(0x0891b2),("cyan","700")=>Some(0x0e7490),
        ("cyan","800")=>Some(0x155e75),("cyan","900")=>Some(0x164e63),
        ("cyan","950")=>Some(0x083344),
        ("sky","50")=>Some(0xf0f9ff),("sky","100")=>Some(0xe0f2fe),
        ("sky","200")=>Some(0xbae6fd),("sky","300")=>Some(0x7dd3fc),
        ("sky","400")=>Some(0x38bdf8),("sky","500")=>Some(0x0ea5e9),
        ("sky","600")=>Some(0x0284c7),("sky","700")=>Some(0x0369a1),
        ("sky","800")=>Some(0x075985),("sky","900")=>Some(0x0c4a6e),
        ("sky","950")=>Some(0x082f49),
        ("blue","50")=>Some(0xeff6ff),("blue","100")=>Some(0xdbeafe),
        ("blue","200")=>Some(0xbfdbfe),("blue","300")=>Some(0x93c5fd),
        ("blue","400")=>Some(0x60a5fa),("blue","500")=>Some(0x3b82f6),
        ("blue","600")=>Some(0x2563eb),("blue","700")=>Some(0x1d4ed8),
        ("blue","800")=>Some(0x1e40af),("blue","900")=>Some(0x1e3a8a),
        ("blue","950")=>Some(0x172554),
        ("indigo","50")=>Some(0xeef2ff),("indigo","100")=>Some(0xe0e7ff),
        ("indigo","200")=>Some(0xc7d2fe),("indigo","300")=>Some(0xa5b4fc),
        ("indigo","400")=>Some(0x818cf8),("indigo","500")=>Some(0x6366f1),
        ("indigo","600")=>Some(0x4f46e5),("indigo","700")=>Some(0x4338ca),
        ("indigo","800")=>Some(0x3730a3),("indigo","900")=>Some(0x312e81),
        ("indigo","950")=>Some(0x1e1b4b),
        ("violet","50")=>Some(0xf5f3ff),("violet","100")=>Some(0xede9fe),
        ("violet","200")=>Some(0xddd6fe),("violet","300")=>Some(0xc4b5fd),
        ("violet","400")=>Some(0xa78bfa),("violet","500")=>Some(0x8b5cf6),
        ("violet","600")=>Some(0x7c3aed),("violet","700")=>Some(0x6d28d9),
        ("violet","800")=>Some(0x5b21b6),("violet","900")=>Some(0x4c1d95),
        ("violet","950")=>Some(0x2e1065),
        ("purple","50")=>Some(0xfaf5ff),("purple","100")=>Some(0xf3e8ff),
        ("purple","200")=>Some(0xe9d5ff),("purple","300")=>Some(0xd8b4fe),
        ("purple","400")=>Some(0xc084fc),("purple","500")=>Some(0xa855f7),
        ("purple","600")=>Some(0x9333ea),("purple","700")=>Some(0x7e22ce),
        ("purple","800")=>Some(0x6b21a8),("purple","900")=>Some(0x581c87),
        ("purple","950")=>Some(0x3b0764),
        ("fuchsia","50")=>Some(0xfdf4ff),("fuchsia","100")=>Some(0xfae8ff),
        ("fuchsia","200")=>Some(0xf5d0fe),("fuchsia","300")=>Some(0xf0abfc),
        ("fuchsia","400")=>Some(0xe879f9),("fuchsia","500")=>Some(0xd946ef),
        ("fuchsia","600")=>Some(0xc026d3),("fuchsia","700")=>Some(0xa21caf),
        ("fuchsia","800")=>Some(0x86198f),("fuchsia","900")=>Some(0x701a75),
        ("fuchsia","950")=>Some(0x4a044e),
        ("pink","50")=>Some(0xfdf2f8),("pink","100")=>Some(0xfce7f3),
        ("pink","200")=>Some(0xfbcfe8),("pink","300")=>Some(0xf9a8d4),
        ("pink","400")=>Some(0xf472b6),("pink","500")=>Some(0xec4899),
        ("pink","600")=>Some(0xdb2777),("pink","700")=>Some(0xbe185d),
        ("pink","800")=>Some(0x9d174d),("pink","900")=>Some(0x831843),
        ("pink","950")=>Some(0x500724),
        ("rose","50")=>Some(0xfff1f2),("rose","100")=>Some(0xffe4e6),
        ("rose","200")=>Some(0xfecdd3),("rose","300")=>Some(0xfda4af),
        ("rose","400")=>Some(0xfb7185),("rose","500")=>Some(0xf43f5e),
        ("rose","600")=>Some(0xe11d48),("rose","700")=>Some(0xbe123c),
        ("rose","800")=>Some(0x9f1239),("rose","900")=>Some(0x881337),
        ("rose","950")=>Some(0x4c0519),
        _ => None,
    }
}
