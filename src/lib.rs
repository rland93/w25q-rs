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

/// BMI088 device object.
#[derive(Debug)]
pub struct W25Q<DI> {
    /// Digital interface (spi)
    iface: DI,
}

mod private {
    use super::interface;
    pub trait Sealed {}

    impl<SPI> Sealed for interface::SpiInterface<SPI> {}
}

impl<SPI> W25Q<interface::SpiInterface<SPI>> {
    /// Create new instance of the BMI088 device communicating through SPI.
    pub fn new_with_spi(spi: SPI) -> Self {
        W25Q {
            iface: interface::SpiInterface { spi },
        }
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
