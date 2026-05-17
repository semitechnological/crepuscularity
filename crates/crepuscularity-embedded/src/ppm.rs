//! PPM (P6) export.

use std::path::Path;

use crate::framebuffer::Rgb888Buffer;

pub fn write_ppm(path: &Path, buffer: &Rgb888Buffer) -> Result<(), String> {
    buffer.write_ppm(path)
}
