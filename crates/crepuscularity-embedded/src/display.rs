//! Panel flush helpers for common RGB565 display drivers (SPI ILI9341, ST7789, LTDC, …).

use crate::framebuffer::Framebuffer;
use crate::screen::ScreenSize;

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};
#[cfg(feature = "std")]
use std::{string::String, vec::Vec};

/// How each 16-bit pixel is encoded on the wire (many controllers differ).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Rgb565ByteOrder {
    /// Standard RGB565: high byte = RRRRRGGG, low byte = GGGGBBBB.
    #[default]
    Rgb,
    /// Swap the two bytes of each pixel (required on some ST7789/ILI9341 modules).
    Bgr,
}

/// Window offset and byte order for a physical panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PanelConfig {
    pub byte_order: Rgb565ByteOrder,
    /// Column offset applied by some 240×320 modules (visible area starts at (x_offset, y_offset)).
    pub x_offset: u16,
    pub y_offset: u16,
}

impl PanelConfig {
    pub const fn ili9341_240x320() -> Self {
        Self {
            byte_order: Rgb565ByteOrder::Rgb,
            x_offset: 0,
            y_offset: 0,
        }
    }

    pub const fn st7789_240x320() -> Self {
        Self {
            byte_order: Rgb565ByteOrder::Bgr,
            x_offset: 0,
            y_offset: 0,
        }
    }
}

/// Flush a row-major RGB565 buffer to a display (SPI burst, `esp_lcd`, LTDC helper, etc.).
pub trait Rgb565Display {
    fn screen_size(&self) -> ScreenSize;

    /// Draw `pixels` into the rectangle `(x, y) .. (x + w, y + h)`.
    fn flush_rgb565_rect(
        &mut self,
        x: u16,
        y: u16,
        w: u16,
        h: u16,
        pixels: &[u8],
    ) -> Result<(), DisplayError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DisplayError {
    Message(String),
}

impl core::fmt::Display for DisplayError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            DisplayError::Message(m) => write!(f, "{m}"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for DisplayError {}

/// Swap each RGB565 pixel's two bytes into `scratch` (ST7789 / some ILI9341 modules).
pub fn swap_rgb565_bytes_bgr<'a>(src: &[u8], scratch: &'a mut Vec<u8>) -> &'a [u8] {
    scratch.clear();
    scratch.reserve(src.len());
    let mut i = 0;
    while i + 1 < src.len() {
        scratch.push(src[i + 1]);
        scratch.push(src[i]);
        i += 2;
    }
    scratch.as_slice()
}

/// Write every pixel from `fb` into `display` using `panel` offsets and byte order.
pub fn flush_framebuffer<D: Rgb565Display + ?Sized>(
    display: &mut D,
    fb: &impl Framebuffer,
    panel: PanelConfig,
    scratch: &mut Vec<u8>,
) -> Result<(), DisplayError> {
    let size = fb.screen_size();
    let raw = fb
        .as_rgb565_bytes()
        .ok_or_else(|| DisplayError::Message("framebuffer is not RGB565".into()))?;
    let bytes = match panel.byte_order {
        Rgb565ByteOrder::Rgb => raw,
        Rgb565ByteOrder::Bgr => swap_rgb565_bytes_bgr(raw, scratch),
    };
    display.flush_rgb565_rect(
        panel.x_offset,
        panel.y_offset,
        size.width,
        size.height,
        bytes,
    )
}
