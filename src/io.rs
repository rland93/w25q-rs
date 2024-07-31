use crate::W25Q;

use embedded_hal::delay;
use embedded_hal::spi;
use embedded_hal::spi::Error;
use embedded_io::{BufRead, ErrorType, Read, ReadReady, Seek, SeekFrom, Write, WriteReady};

use embedded_io::ErrorKind;

#[derive(Debug)]
pub enum W25QError {
    Io(ErrorKind),
    Spi(spi::ErrorKind),
}

impl embedded_io::Error for W25QError {
    fn kind(&self) -> ErrorKind {
        match self {
            W25QError::Io(kind) => *kind,
            W25QError::Spi(e) => match e {
                spi::ErrorKind::Overrun => ErrorKind::OutOfMemory,
                spi::ErrorKind::ModeFault => ErrorKind::PermissionDenied,
                spi::ErrorKind::FrameFormat => ErrorKind::InvalidData,
                spi::ErrorKind::ChipSelectFault => ErrorKind::ConnectionReset,
                spi::ErrorKind::Other => ErrorKind::Other,
                _ => ErrorKind::Other,
            },
        }
    }
}

impl From<spi::ErrorKind> for W25QError {
    fn from(e: spi::ErrorKind) -> Self {
        match e {
            spi::ErrorKind::Overrun => W25QError::Spi(spi::ErrorKind::Overrun),
            spi::ErrorKind::ModeFault => W25QError::Spi(spi::ErrorKind::ModeFault),
            spi::ErrorKind::FrameFormat => W25QError::Spi(spi::ErrorKind::FrameFormat),
            spi::ErrorKind::ChipSelectFault => W25QError::Spi(spi::ErrorKind::ChipSelectFault),
            spi::ErrorKind::Other => W25QError::Spi(spi::ErrorKind::Other),
            _ => W25QError::Spi(spi::ErrorKind::Other),
        }
    }
}

impl<SPI, DELAY> ErrorType for W25Q<SPI, DELAY>
where
    SPI: spi::SpiDevice,
    DELAY: delay::DelayNs,
{
    type Error = W25QError;
}

impl<SPI, DELAY> Read for W25Q<SPI, DELAY>
where
    SPI: spi::SpiDevice,
    DELAY: delay::DelayNs,
{
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, W25QError> {
        let address = self.seek_ptr as u32;
        self.fast_read(address, buf)
            .map_err(|e| W25QError::Spi(e.kind()))?;
        self.seek_ptr += buf.len();
        Ok(buf.len())
    }
}

impl<SPI, DELAY> Write for W25Q<SPI, DELAY>
where
    SPI: spi::SpiDevice,
    DELAY: delay::DelayNs,
{
    fn write(&mut self, buf: &[u8]) -> Result<usize, W25QError> {
        let address = self.seek_ptr as u32;
        self.page_program(address, buf)
            .map_err(|e| W25QError::Spi(e.kind()))?;
        self.seek_ptr += buf.len();
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<(), W25QError> {
        Ok(())
    }
}

impl<SPI, DELAY> Seek for W25Q<SPI, DELAY>
where
    SPI: spi::SpiDevice,
    DELAY: delay::DelayNs,
{
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, W25QError> {
        let seeked: u64;
        match pos {
            SeekFrom::Start(pos) => {
                if pos > self.capacity() {
                    return Err(W25QError::Io(ErrorKind::InvalidInput));
                }
                seeked = pos;
                self.seek_ptr = seeked as usize;
            }
            SeekFrom::End(pos) => {
                if pos > 0 {
                    return Err(W25QError::Io(ErrorKind::InvalidInput));
                }
                seeked = self.capacity() - pos.abs() as u64;
                self.seek_ptr = seeked as usize;
            }
            SeekFrom::Current(pos) => {
                if (self.seek_ptr as i64 + pos) < 0
                    || (self.seek_ptr as i64 + pos) as u64 > self.capacity()
                {
                    return Err(W25QError::Io(ErrorKind::InvalidInput));
                }
                seeked = (self.seek_ptr as i64 + pos) as u64;
                self.seek_ptr = seeked as usize;
            }
        }
        Ok(seeked)
    }
}

impl<SPI, DELAY> ReadReady for W25Q<SPI, DELAY>
where
    SPI: spi::SpiDevice,
    DELAY: delay::DelayNs,
{
    fn read_ready(&mut self) -> Result<bool, W25QError> {
        Ok(true)
    }
}

impl<SPI, DELAY> WriteReady for W25Q<SPI, DELAY>
where
    SPI: spi::SpiDevice,
    DELAY: delay::DelayNs,
{
    fn write_ready(&mut self) -> Result<bool, W25QError> {
        Ok(true)
    }
}

impl<SPI, DELAY> BufRead for W25Q<SPI, DELAY>
where
    SPI: spi::SpiDevice,
    DELAY: delay::DelayNs,
{
    fn fill_buf(&mut self) -> Result<&[u8], Self::Error> {
        if self.buffer_start >= self.buffer_end {
            let page_start = (self.seek_ptr / crate::PAGE_SIZE) * crate::PAGE_SIZE;
            self.fast_read_into_internal_buffer(page_start as u32)
                .map_err(|e| W25QError::Spi(e.kind()))?;
            self.buffer_start = self.seek_ptr - page_start;
            self.buffer_end = crate::PAGE_SIZE;
        }
        Ok(&self.buffer[self.buffer_start..self.buffer_end])
    }

    fn consume(&mut self, amt: usize) {
        let new_start = self.buffer_start + amt;
        self.buffer_start = new_start.min(self.buffer_end);
        self.seek_ptr += amt;
    }
}
