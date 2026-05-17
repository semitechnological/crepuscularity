//! Optional display drivers behind Cargo features (`mipidsi`, `embassy-stm32`, `esp-idf`).

pub mod preset;

#[cfg(feature = "mipidsi")]
pub mod mipidsi_blit;

#[cfg(feature = "embassy-stm32")]
pub mod embassy_stm32;

#[cfg(feature = "esp-idf")]
pub mod esp_idf;
