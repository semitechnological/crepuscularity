//! Display geometry.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ScreenSize {
    pub width: u16,
    pub height: u16,
}

impl ScreenSize {
    pub const fn new(width: u16, height: u16) -> Self {
        Self { width, height }
    }

    pub const fn pixel_count(self) -> usize {
        self.width as usize * self.height as usize
    }
}
