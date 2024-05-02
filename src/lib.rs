//! Driver for W25Q. Uses embedded-hal traits.
//!
//! [`embedded-hal`]: https://github.com/rust-embedded/embedded-hal
//!
//!
#![allow(unused)]
#![deny(unsafe_code)]
#![no_std]

#[cfg(feature = "littlefs2")]
use littlefs2::io;

pub mod interface;
mod mem;
mod registers;
mod types;

use embedded_hal::delay::DelayNs;

pub use crate::types::*;

pub const SECTOR_SIZE: usize = 4096;
pub const SECTOR_COUNT: usize = 4096;
pub const BLOCK_SIZE_32: usize = 32768;
pub const BLOCK_SIZE_64: usize = 65536;
pub const PAGE_SIZE: usize = 256;
pub const PAGE_COUNT: usize = 65536;
pub const TOTAL_SIZE: usize = SECTOR_COUNT * SECTOR_SIZE;

///  device object.
#[derive(Debug)]
pub struct W25Q<DI, D> {
    /// Digital interface (spi)
    iface: DI,
    pub delay: D,
}

mod private {
    use super::interface;
    pub trait Sealed {}

    impl<SPI> Sealed for interface::SpiInterface<SPI> {}
}

impl<SPI, D> W25Q<interface::SpiInterface<SPI>, D> {
    /// Create new driver instance
    pub fn new_with_spi(spi: SPI, delay: D) -> Self {
        W25Q {
            iface: interface::SpiInterface { spi },
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

#[cfg(feature = "littlefs2")]
impl<CommE> Into<littlefs2::io::Error> for Error<CommE> {
    fn into(self) -> littlefs2::io::Error {
        match self {
            Error::Comm(_) => littlefs2::io::Error::Io,
            Error::BadData => littlefs2::io::Error::Io,
            Error::AddressSize => littlefs2::io::Error::Io,
        }
    }
}
