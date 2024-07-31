#![no_std]
#![deny(unsafe_code)]

#[cfg(feature = "defmt")]
use defmt::Format;

pub mod io;

use embedded_hal::{
    delay::{self, DelayNs},
    spi::{self, SpiDevice},
};

pub const SECTOR_SIZE: usize = 4096;
pub const SECTOR_COUNT: usize = 4096;
pub const BLOCK_SIZE_32: usize = 32768;
pub const BLOCK_SIZE_64: usize = 65536;
pub const PAGE_SIZE: usize = 256;
pub const PAGE_COUNT: usize = 65536;
pub const TOTAL_SIZE: usize = SECTOR_COUNT * SECTOR_SIZE;

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, PartialEq)]
pub enum SR {
    SR1(SR1),
    SR2(SR2),
    SR3(SR3),
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, PartialEq, Default)]
pub struct SR1 {
    pub srp0: bool,
    pub sec: bool,
    pub tb: bool,
    pub bp: u8,
    pub wel: bool,
    pub busy: bool,
}

impl From<u8> for SR1 {
    fn from(byte: u8) -> Self {
        Self {
            srp0: (byte & 0b1000_0000) != 0,
            sec: (byte & 0b0100_0000) != 0,
            tb: (byte & 0b0010_0000) != 0,
            bp: (byte & 0b0001_1100) >> 2,
            wel: (byte & 0b0000_0010) != 0,
            busy: (byte & 0b0000_0001) != 0,
        }
    }
}

impl From<SR1> for u8 {
    fn from(sr1: SR1) -> Self {
        (sr1.srp0 as u8) << 7
            | (sr1.sec as u8) << 6
            | (sr1.tb as u8) << 5
            | (sr1.bp & 0b111) << 2
            | (sr1.wel as u8) << 1
            | sr1.busy as u8
    }
}

impl SR1 {
    pub fn to_writable_u8(&self) -> u8 {
        let mut result = 0;
        result |= (self.sec as u8) << 6;
        result |= (self.tb as u8) << 5;
        result |= (self.bp & 0b111) << 2;
        result
    }
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, PartialEq, Default)]
pub struct SR2 {
    pub sus: bool,
    pub cmp: bool,
    pub lb: u8,
    pub qe: bool,
    pub srp1: bool,
}

impl From<u8> for SR2 {
    fn from(byte: u8) -> Self {
        Self {
            sus: byte & 0b1000_0000 != 0,
            cmp: byte & 0b0100_0000 != 0,
            lb: (byte >> 5) & 0b111,
            qe: byte & 0b0000_0010 != 0,
            srp1: byte & 0b0000_0001 != 0,
        }
    }
}

impl From<SR2> for u8 {
    fn from(sr2: SR2) -> Self {
        (sr2.sus as u8) << 7
            | (sr2.cmp as u8) << 6
            | (sr2.lb & 0b111) << 5
            | (sr2.qe as u8) << 1
            | sr2.srp1 as u8
    }
}

impl SR2 {
    pub fn to_writable_u8(&self) -> u8 {
        let mut result = 0;
        result |= (self.cmp as u8) << 6;
        result |= (self.qe as u8) << 1;
        result |= (self.lb & 0b111) << 3;
        result
    }
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, PartialEq, Default)]
pub struct SR3 {
    pub hold_or_reset: bool,
    pub driver_strength: u8,
    pub wps: bool,
}

impl From<u8> for SR3 {
    fn from(byte: u8) -> Self {
        Self {
            hold_or_reset: byte & 0b1000_0000 != 0,
            driver_strength: (byte >> 5) & 0b11,
            wps: byte & 0b0000_0100 != 0,
        }
    }
}

impl From<SR3> for u8 {
    fn from(sr3: SR3) -> Self {
        (sr3.hold_or_reset as u8) << 7 | (sr3.driver_strength & 0b11) << 5 | (sr3.wps as u8) << 2
    }
}

impl SR3 {
    pub fn to_writable_u8(&self) -> u8 {
        let mut result = 0;
        result |= (self.wps as u8) << 2;
        result |= (self.driver_strength & 0b11) << 5;
        result
    }
}

#[allow(non_camel_case_types)]
#[allow(clippy::upper_case_acronyms)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Register {
    WRITE_ENABLE = 0x06,
    VOLATILE_SR_WRITE_ENABLE = 0x50,
    WRITE_DISABLE = 0x04,
    RELEASE_POWER_DOWN = 0xAB,
    MANUFACTURER_DEVICE_ID = 0x90,
    JEDEC_ID = 0x9F,
    READ_UNIQUE_ID = 0x4B,
    READ_DATA = 0x03,
    FAST_READ = 0x0B,
    FAST_READ_DUAL_OUTPUT = 0x3B,
    FAST_READ_QUAD_OUTPUT = 0x6B,
    PAGE_PROGRAM = 0x02,
    SECTOR_ERASE = 0x20,
    BLOCK_ERASE_32KB = 0x52,
    BLOCK_ERASE_64KB = 0xD8,
    CHIP_ERASE = 0xC7,
    CHIP_ERASE_2 = 0x60,
    READ_STATUS_REGISTER_1 = 0x05,
    WRITE_STATUS_REGISTER_1 = 0x01,
    READ_STATUS_REGISTER_2 = 0x35,
    WRITE_STATUS_REGISTER_2 = 0x31,
    READ_STATUS_REGISTER_3 = 0x15,
    WRITE_STATUS_REGISTER_3 = 0x11,
    READ_SFDP_REGISTER = 0x5A,
    ERASE_SECURITY_REGISTER = 0x44,
    PROGRAM_SECURITY_REGISTER = 0x42,
    READ_SECURITY_REGISTER = 0x48,
    GLOBAL_BLOCK_LOCK = 0x7E,
    GLOBAL_BLOCK_UNLOCK = 0x98,
    READ_BLOCK_LOCK = 0x3D,
    INDIVIDUAL_BLOCK_LOCK = 0x36,
    INDIVIDUAL_BLOCK_UNLOCK = 0x39,
    ERASE_PROGRAM_SUSPEND = 0x75,
    ERASE_PROGRAM_RESUME = 0x7A,
    POWER_DOWN = 0xB9,
    ENABLE_RESET = 0x66,
    RESET_DEVICE = 0x99,
}

///  device object.
pub struct W25Q<SPI, DELAY>
where
    SPI: SpiDevice,
    DELAY: DelayNs,
{
    pub periph: SPI,
    /// erase and write delays
    pub delay: DELAY,
    /// address pointer for seek operations
    seek_ptr: usize,
    /// buffer for read/write operations
    buffer: [u8; crate::PAGE_SIZE],
    /// buffer start index
    buffer_start: usize,
    /// buffer end index
    buffer_end: usize,
}

#[cfg(feature = "defmt")]
impl<SPI, DELAY> Format for W25Q<SPI, DELAY>
where
    SPI: SpiDevice,
    DELAY: DelayNs,
{
    fn format(&self, f: defmt::Formatter) {
        let addr = self as *const _ as usize;

        defmt::write!(
            f,
            "W25Q@{:#x}: seek ptr: {:#x}, internal buf={:#x}..{:#x}",
            addr,
            self.seek_ptr,
            self.buffer_start,
            self.buffer_end,
        );
    }
}

impl<SPI, DELAY> W25Q<SPI, DELAY>
where
    SPI: SpiDevice,
    DELAY: DelayNs,
{
    pub fn new_with_spi(spi_dev: SPI, delay: DELAY) -> Self {
        Self {
            periph: spi_dev,
            delay: delay,
            seek_ptr: 0x000000,
            buffer: [0x00; crate::PAGE_SIZE],
            buffer_start: 0x000000,
            buffer_end: 0x000100,
        }
    }
}

impl<SPI, DELAY> W25Q<SPI, DELAY>
where
    SPI: spi::SpiDevice,
    DELAY: delay::DelayNs,
{
    /// with `command` at `address`, read bytes into payload.
    pub(crate) fn read_from_address(
        &mut self,
        command: u8,
        address: u32,
        payload: &mut [u8],
    ) -> Result<u8, SPI::Error> {
        // [command, address, address, address] MSB
        let mut data = [0; 4];
        data[0] = command;
        data[1] = ((address >> 16) & 0xFF) as u8;
        data[2] = ((address >> 8) & 0xFF) as u8;
        data[3] = (address & 0xFF) as u8;

        self.periph
            .transaction(&mut [spi::Operation::Write(&data), spi::Operation::Read(payload)])?;
        Ok(data[0])
    }

    /// with `command` (no address), read bytes into payload.
    pub(crate) fn read_register(
        &mut self,
        command: u8,
        payload: &mut [u8],
    ) -> Result<(), SPI::Error> {
        self.periph.transaction(&mut [
            spi::Operation::Write(&[command]),
            spi::Operation::Read(payload),
        ])?;
        Ok(())
    }

    /// write bytes from `payload` with command `command` at `address`
    pub(crate) fn write_address(
        &mut self,
        command: u8,
        address: u32,
        payload: &[u8],
    ) -> Result<(), SPI::Error> {
        // [command, address, address, address] MSB
        let mut data = [0; 4];
        data[0] = command;
        data[1] = ((address >> 16) & 0xFF) as u8;
        data[2] = ((address >> 8) & 0xFF) as u8;
        data[3] = (address & 0xFF) as u8;

        // write enable first
        self.periph.transaction(&mut [spi::Operation::Write(&[
            Register::VOLATILE_SR_WRITE_ENABLE as u8,
        ])])?;

        self.periph
            .transaction(&mut [spi::Operation::Write(&data), spi::Operation::Write(payload)])?;
        Ok(())
    }

    /// with `command`, write data from `payload`
    pub(crate) fn write_data(&mut self, command: u8, payload: &[u8]) -> Result<(), SPI::Error> {
        self.periph.transaction(&mut [
            spi::Operation::Write(&[Register::VOLATILE_SR_WRITE_ENABLE as u8]),
            spi::Operation::Write(&[command]),
            spi::Operation::Write(payload),
        ])?;

        Ok(())
    }

    pub fn read_jedec_id(&mut self) -> Result<[u8; 3], SPI::Error> {
        let mut id = [0u8; 3];
        self.read_register(Register::JEDEC_ID as u8, &mut id)?;
        Ok(id)
    }

    pub fn read_unique_id(&mut self) -> Result<[u8; 8], SPI::Error> {
        let mut id = [0u8; 8];
        let cmd = [Register::READ_UNIQUE_ID as u8, 0, 0, 0, 0];
        self.periph
            .transaction(&mut [spi::Operation::Write(&cmd), spi::Operation::Read(&mut id)])?;
        Ok(id)
    }

    pub fn read_status_register(&mut self, register: SR) -> Result<u8, SPI::Error> {
        let cmd = match register {
            SR::SR1(_) => Register::READ_STATUS_REGISTER_1,
            SR::SR2(_) => Register::READ_STATUS_REGISTER_2,
            SR::SR3(_) => Register::READ_STATUS_REGISTER_3,
        };
        let mut status = [0u8; 1];
        self.read_register(cmd as u8, &mut status)?;
        Ok(status[0])
    }

    pub fn write_status_register(&mut self, register: SR) -> Result<(), SPI::Error> {
        let (cmd, value) = match register {
            SR::SR1(sr1) => (Register::WRITE_STATUS_REGISTER_1, sr1.to_writable_u8()),
            SR::SR2(sr2) => (Register::WRITE_STATUS_REGISTER_2, sr2.to_writable_u8()),
            SR::SR3(sr3) => (Register::WRITE_STATUS_REGISTER_3, sr3.to_writable_u8()),
        };
        self.write_enable()?;
        self.write_data(cmd as u8, &[value])?;
        self.wait_until_ready()
    }

    pub fn write_enable(&mut self) -> Result<(), SPI::Error> {
        self.periph
            .transaction(&mut [spi::Operation::Write(&[Register::WRITE_ENABLE as u8])])
    }

    pub fn write_disable(&mut self) -> Result<(), SPI::Error> {
        self.periph
            .transaction(&mut [spi::Operation::Write(&[Register::WRITE_DISABLE as u8])])
    }

    pub fn chip_erase(&mut self) -> Result<(), SPI::Error> {
        self.write_enable()?;
        self.periph
            .transaction(&mut [spi::Operation::Write(&[Register::CHIP_ERASE as u8])])?;
        self.wait_until_ready()
    }

    pub fn sector_erase(&mut self, address: u32) -> Result<(), SPI::Error> {
        self.write_enable()?;
        self.write_address(Register::SECTOR_ERASE as u8, address, &[])?;
        self.wait_until_ready()
    }

    pub fn block_erase_32kb(&mut self, address: u32) -> Result<(), SPI::Error> {
        self.write_enable()?;
        self.write_address(Register::BLOCK_ERASE_32KB as u8, address, &[])?;
        self.wait_until_ready()
    }

    pub fn block_erase_64kb(&mut self, address: u32) -> Result<(), SPI::Error> {
        self.write_enable()?;
        self.write_address(Register::BLOCK_ERASE_64KB as u8, address, &[])?;
        self.wait_until_ready()
    }

    pub fn page_program(&mut self, address: u32, data: &[u8]) -> Result<(), SPI::Error> {
        self.write_enable()?;
        self.write_address(Register::PAGE_PROGRAM as u8, address, data)?;
        self.wait_until_ready()
    }

    pub fn read_data(&mut self, address: u32, data: &mut [u8]) -> Result<(), SPI::Error> {
        self.read_from_address(Register::READ_DATA as u8, address, data)?;
        Ok(())
    }

    pub fn fast_read(&mut self, address: u32, data: &mut [u8]) -> Result<(), SPI::Error> {
        let mut cmd = [Register::FAST_READ as u8, 0, 0, 0, 0];
        cmd[1..4].copy_from_slice(&address.to_be_bytes()[1..]);
        self.periph
            .transaction(&mut [spi::Operation::Write(&cmd), spi::Operation::Read(data)])?;
        Ok(())
    }

    pub(crate) fn fast_read_into_internal_buffer(
        &mut self,
        address: u32,
    ) -> Result<(), SPI::Error> {
        let mut cmd = [Register::FAST_READ as u8, 0, 0, 0, 0];
        cmd[1..4].copy_from_slice(&address.to_be_bytes()[1..]);
        self.periph.transaction(&mut [
            spi::Operation::Write(&cmd),
            spi::Operation::Read(&mut self.buffer),
        ])?;
        self.buffer_start = 0;
        self.buffer_end = self.buffer.len();
        Ok(())
    }

    pub fn power_down(&mut self) -> Result<(), SPI::Error> {
        self.periph
            .transaction(&mut [spi::Operation::Write(&[Register::POWER_DOWN as u8])])
    }

    pub fn release_power_down(&mut self) -> Result<(), SPI::Error> {
        self.periph
            .transaction(&mut [spi::Operation::Write(&[Register::RELEASE_POWER_DOWN as u8])])
    }

    pub fn erase_security_register(&mut self, address: u32) -> Result<(), SPI::Error> {
        self.write_enable()?;
        self.write_address(Register::ERASE_SECURITY_REGISTER as u8, address, &[])?;
        self.wait_until_ready()
    }

    pub fn program_security_register(
        &mut self,
        address: u32,
        data: &[u8],
    ) -> Result<(), SPI::Error> {
        self.write_enable()?;
        self.write_address(Register::PROGRAM_SECURITY_REGISTER as u8, address, data)?;
        self.wait_until_ready()
    }

    pub fn read_security_register(
        &mut self,
        address: u32,
        data: &mut [u8],
    ) -> Result<(), SPI::Error> {
        self.read_from_address(Register::READ_SECURITY_REGISTER as u8, address, data)?;
        Ok(())
    }

    pub fn global_block_lock(&mut self) -> Result<(), SPI::Error> {
        self.write_enable()?;
        self.periph
            .transaction(&mut [spi::Operation::Write(&[Register::GLOBAL_BLOCK_LOCK as u8])])
    }

    pub fn global_block_unlock(&mut self) -> Result<(), SPI::Error> {
        self.write_enable()?;
        self.periph.transaction(&mut [spi::Operation::Write(
            &[Register::GLOBAL_BLOCK_UNLOCK as u8],
        )])
    }

    pub fn read_block_lock(&mut self, address: u32) -> Result<bool, SPI::Error> {
        let mut status = [0u8; 1];
        self.read_from_address(Register::READ_BLOCK_LOCK as u8, address, &mut status)?;
        Ok(status[0] & 0x01 != 0)
    }

    pub fn individual_block_lock(&mut self, address: u32) -> Result<(), SPI::Error> {
        self.write_enable()?;
        self.write_address(Register::INDIVIDUAL_BLOCK_LOCK as u8, address, &[])?;
        self.wait_until_ready()
    }

    pub fn individual_block_unlock(&mut self, address: u32) -> Result<(), SPI::Error> {
        self.write_enable()?;
        self.write_address(Register::INDIVIDUAL_BLOCK_UNLOCK as u8, address, &[])?;
        self.wait_until_ready()
    }

    pub fn erase_program_suspend(&mut self) -> Result<(), SPI::Error> {
        self.periph.transaction(&mut [spi::Operation::Write(&[
            Register::ERASE_PROGRAM_SUSPEND as u8
        ])])
    }

    pub fn erase_program_resume(&mut self) -> Result<(), SPI::Error> {
        self.periph.transaction(&mut [spi::Operation::Write(&[
            Register::ERASE_PROGRAM_RESUME as u8
        ])])
    }

    pub fn reset_device(&mut self) -> Result<(), SPI::Error> {
        self.periph.transaction(&mut [
            spi::Operation::Write(&[Register::ENABLE_RESET as u8]),
            spi::Operation::Write(&[Register::RESET_DEVICE as u8]),
        ])?;
        self.delay.delay_ms(30); // Wait for reset to complete
        Ok(())
    }

    pub fn capacity(&self) -> u64 {
        TOTAL_SIZE as u64
    }

    fn wait_until_ready(&mut self) -> Result<(), SPI::Error> {
        loop {
            let status = self.read_status_register(SR::SR1(SR1::default()))?;
            if status & 0x01 == 0 {
                break;
            }
            self.delay.delay_ms(1);
        }
        Ok(())
    }
}
