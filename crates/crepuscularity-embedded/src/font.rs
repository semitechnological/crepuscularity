//! Bitmap font metrics, glyph lookup, and framebuffer text drawing.

use crate::color::Color;
use crate::document::Rect;
use crate::framebuffer::Framebuffer;

/// Monospace 8×8 font metrics used by the software renderer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FontMetrics {
    pub cell_width: u8,
    pub cell_height: u8,
    pub baseline: u8,
}

impl Default for FontMetrics {
    fn default() -> Self {
        Self::builtin_8x8()
    }
}

impl FontMetrics {
    pub const fn builtin_8x8() -> Self {
        Self {
            cell_width: 8,
            cell_height: 8,
            baseline: 7,
        }
    }

    pub fn line_height(&self) -> u16 {
        self.cell_height as u16
    }

    pub fn measure_text(&self, text: &str) -> (u16, u16) {
        let lines = if text.is_empty() {
            1
        } else {
            text.lines().count()
        };
        let max_cols = text
            .lines()
            .map(|line| line.chars().count())
            .max()
            .unwrap_or(0);
        (
            (max_cols as u16).saturating_mul(self.cell_width as u16),
            (lines as u16).saturating_mul(self.line_height()),
        )
    }

    pub fn glyph_rows(&self, ch: char) -> [u8; 8] {
        let _ = self;
        glyph_bitmap(ch)
    }

    pub fn is_supported(&self, ch: char) -> bool {
        let _ = self;
        (32..127).contains(&(ch as u32 as u8))
    }
}

pub fn measure_text(text: &str) -> (u16, u16) {
    FontMetrics::default().measure_text(text)
}

pub fn char_width() -> u16 {
    FontMetrics::default().cell_width as u16
}

pub fn char_height() -> u16 {
    FontMetrics::default().cell_height as u16
}

pub fn draw_text(
    fb: &mut impl Framebuffer,
    bounds: Rect,
    text: &str,
    color: Color,
    metrics: &FontMetrics,
) {
    let pad_x = 2u16;
    let pad_y = 2u16;
    let mut y = bounds.y.saturating_add(pad_y);
    for line in text.lines() {
        let mut x = bounds.x.saturating_add(pad_x);
        for ch in line.chars() {
            if x.saturating_add(metrics.cell_width as u16) > bounds.x.saturating_add(bounds.w) {
                break;
            }
            if metrics.is_supported(ch) {
                draw_glyph(fb, x, y, metrics.glyph_rows(ch), color);
            } else {
                draw_replacement_box(
                    fb,
                    x,
                    y,
                    metrics.cell_width as u16,
                    metrics.cell_height as u16,
                    color,
                );
            }
            x = x.saturating_add(metrics.cell_width as u16);
        }
        y = y.saturating_add(metrics.line_height());
        if y >= bounds.y.saturating_add(bounds.h) {
            break;
        }
    }
}

fn draw_glyph(fb: &mut impl Framebuffer, x: u16, y: u16, rows: [u8; 8], color: Color) {
    for (ri, row) in rows.iter().enumerate() {
        for col in 0..8u16 {
            if row & (1 << (7 - col)) != 0 {
                fb.set_pixel(x + col, y + ri as u16, color);
            }
        }
    }
}

fn draw_replacement_box(fb: &mut impl Framebuffer, x: u16, y: u16, w: u16, h: u16, color: Color) {
    for dy in 0..h {
        for dx in 0..w {
            fb.set_pixel(x + dx, y + dy, color);
        }
    }
}

fn glyph_bitmap(ch: char) -> [u8; 8] {
    let idx = ch as u8;
    if (32..127).contains(&idx) {
        GLYPHS[(idx - 32) as usize]
    } else {
        [0; 8]
    }
}

use crate::font_data::GLYPHS;
