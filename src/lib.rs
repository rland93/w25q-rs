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
use embedded_hal::spi::SpiDevice;

pub use crate::types::*;

pub const SECTOR_SIZE: usize = 4096;
pub const SECTOR_COUNT: usize = 4096;
pub const BLOCK_SIZE_32: usize = 32768;
pub const BLOCK_SIZE_64: usize = 65536;
pub const PAGE_SIZE: usize = 256;
pub const PAGE_COUNT: usize = 65536;
pub const TOTAL_SIZE: usize = SECTOR_COUNT * SECTOR_SIZE;

///  device object.
pub struct W25Q<P, D>
where
    P: SpiDevice,
    D: DelayNs,
{
    pub periph: P,
    /// erase and write delays
    pub delay: D,
    /// address pointer for seek operations
    seek_ptr: usize,
}

impl<P, D> W25Q<P, D>
where
    P: SpiDevice,
    D: DelayNs,
{
    pub fn new_with_spi(spi_dev: P, delay: D) -> Self {
        Self {
            periph: spi_dev,
            delay: delay,
            seek_ptr: 0x000000,
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
