#![no_std]
#![no_main]

use embedded_alloc::TlsfHeap as Heap;
use crepuscularity_embedded::panel::embassy_stm32::{ili9341_240x320_static, Ili9341Spi240x320};

#[global_allocator]
static HEAP: Heap = Heap::empty();
use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

/// Generated on the host: `WRITE_FRAME=frame.bin cargo run -p embedded-stm32-host-sim`
static FRAME: &[u8] = include_bytes!("../frame.bin");

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    unsafe {
        embedded_alloc::init!(HEAP, 32 * 1024);
    }

    let p = embassy_stm32::init(Default::default());
    let _led = Output::new(p.PC13, Level::Low, Speed::Low);

    let panel = ili9341_240x320_static(p.SPI1, p.PA5, p.PA7, p.PA6, p.PA4, p.PA3, p.PA2);
    info!(
        "ili9341 ready {}x{}",
        Ili9341Spi240x320::WIDTH,
        Ili9341Spi240x320::HEIGHT
    );

    loop {
        panel.blit_full(FRAME).expect("blit");
        info!("frame blitted");
        Timer::after(Duration::from_secs(1)).await;
    }
}
