//! Driver for W25Q. Uses embedded-hal traits.
//!
//! [`embedded-hal`]: https://github.com/rust-embedded/embedded-hal
//!
//!
#![allow(unused)]
#![deny(unsafe_code)]
#![no_std]

pub mod interface;
mod mem;
mod registers;
mod types;

use embedded_hal::delay::DelayNs;

pub use crate::types::*;

/// BMI088 device object.
#[derive(Debug)]
pub struct W25Q<DI, D> {
    /// Digital interface (spi)
    iface: DI,
    config: FlashConfig,
    delay: D,
}

mod private {
    use super::interface;
    pub trait Sealed {}

    impl<SPI> Sealed for interface::SpiInterface<SPI> {}
}

impl<SPI, D> W25Q<interface::SpiInterface<SPI>, D> {
    /// Create new driver instance
    pub fn new_with_spi(spi: SPI, config: FlashConfig, delay: D) -> Self {
        W25Q {
            iface: interface::SpiInterface { spi },
            config: config,
            delay: delay,
        }
    }

    pub fn destroy(self) -> SPI {
        return self.iface.destroy();
    }
}

/// Crate Errors
#[derive(Debug)]
pub enum Error<CommE> {
    /// Interface communication error
    Comm(CommE),
    /// Bad data
    BadData,
    /// Address size
    AddressSize,
}
