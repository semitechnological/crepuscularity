//! Rasterize [`EmbeddedDocument`] trees into a [`Framebuffer`].

use crate::color::Color;
use crate::document::{EmbeddedDocument, EmbeddedNode, Rect, DEFAULT_BG, DEFAULT_TEXT};
use crate::font::{draw_text, FontMetrics};
use crate::framebuffer::Framebuffer;

pub fn paint_document(fb: &mut impl Framebuffer, doc: &EmbeddedDocument) {
    let bg = doc
        .root
        .first()
        .and_then(|n| n.style.bg)
        .unwrap_or(DEFAULT_BG);
    fb.fill(bg);
    for node in &doc.root {
        paint_node(fb, node);
    }
}

fn paint_node(fb: &mut impl Framebuffer, node: &EmbeddedNode) {
    if let Some(bg) = node.style.bg {
        fill_rect(fb, node.bounds, bg);
    }
    for child in &node.children {
        paint_node(fb, child);
    }
    if node.style.border_width > 0 {
        let border_color = node.style.border_color.unwrap_or(DEFAULT_TEXT);
        stroke_rect(fb, node.bounds, node.style.border_width, border_color);
    }
    if let Some(text) = &node.text {
        draw_text(
            fb,
            node.bounds,
            text,
            node.style.text.unwrap_or(DEFAULT_TEXT),
            &FontMetrics::default(),
        );
    }
}

fn fill_rect(fb: &mut impl Framebuffer, r: Rect, color: Color) {
    for y in r.y..r.y.saturating_add(r.h).min(fb.height()) {
        for x in r.x..r.x.saturating_add(r.w).min(fb.width()) {
            fb.set_pixel(x, y, color);
        }
    }
}

fn stroke_rect(fb: &mut impl Framebuffer, r: Rect, width: u16, color: Color) {
    let w = width.max(1);
    for i in 0..w {
        let inset = i;
        let x0 = r.x.saturating_add(inset);
        let y0 = r.y.saturating_add(inset);
        let x1 = r.x.saturating_add(r.w).saturating_sub(1 + inset);
        let y1 = r.y.saturating_add(r.h).saturating_sub(1 + inset);
        if x0 >= fb.width() || y0 >= fb.height() {
            continue;
        }
        for x in x0..=x1.min(fb.width().saturating_sub(1)) {
            fb.set_pixel(x, y0, color);
            if y1 != y0 {
                fb.set_pixel(x, y1.min(fb.height().saturating_sub(1)), color);
            }
        }
        for y in y0..=y1.min(fb.height().saturating_sub(1)) {
            fb.set_pixel(x0, y, color);
            if x1 != x0 {
                fb.set_pixel(x1.min(fb.width().saturating_sub(1)), y, color);
            }
        }
    }
}
