///
use crate::{private, Error};
use embedded_hal::spi;
use embedded_hal::spi::Operation;

/// SPI interface
#[derive(Debug)]
pub struct SpiInterface<SPI> {
    pub(crate) spi: SPI,
}

/// Read data
pub trait ReadData: private::Sealed {
    /// Error type
    type Error;
    /// Read a register
    fn read_from_addr(
        &mut self,
        register: u8,
        addr: u32,
        payload: &mut [u8],
    ) -> Result<u8, Self::Error>;
    /// Read data
    fn read_data(&mut self, register: u8, payload: &mut [u8]) -> Result<(), Self::Error>;
}

impl<SPI> ReadData for SpiInterface<SPI>
where
    SPI: spi::SpiDevice<u8>,
{
    type Error = Error<SPI::Error>;

    fn read_from_addr(
        &mut self,
        register: u8,
        addr: u32,
        payload: &mut [u8],
    ) -> Result<u8, Self::Error> {
        // 24 bit address size check
        if addr & 0xFF000000 != 0 {
            return Err(Error::AddressSize);
        }
        // [register, addr, addr, addr] MSB
        let mut data = [0; 4];
        data[0] = register;
        data[1] = ((addr >> 16) & 0xFF) as u8;
        data[2] = ((addr >> 8) & 0xFF) as u8;
        data[3] = (addr & 0xFF) as u8;

        self.spi
            .transaction(&mut [Operation::Write(&data), Operation::Read(payload)])
            .map_err(Error::Comm)?;
        Ok(data[0])
    }

    fn read_data(&mut self, register: u8, payload: &mut [u8]) -> Result<(), Self::Error> {
        self.spi
            .transaction(&mut [Operation::Write(&[register]), Operation::Read(payload)])
            .map_err(Error::Comm)?;
        Ok(())
    }
}

/// Write Data
pub trait WriteData: private::Sealed {
    /// Error type
    type Error;
    /// Write to register
    fn write_addr(&mut self, register: u8, addr: u32, payload: &[u8]) -> Result<(), Self::Error>;
    /// Write data
    fn write_data(&mut self, payload: &[u8]) -> Result<(), Self::Error>;
}

impl<SPI> WriteData for SpiInterface<SPI>
where
    SPI: spi::SpiDevice<u8>,
{
    type Error = Error<SPI::Error>;

    fn write_addr(&mut self, register: u8, addr: u32, payload: &[u8]) -> Result<(), Self::Error> {
        // 24 bit address size check
        if addr & 0xFF000000 != 0 {
            return Err(Error::AddressSize);
        }
        // [register, addr, addr, addr] MSB
        let mut data = [0; 4];
        data[0] = register;
        data[1] = ((addr >> 16) & 0xFF) as u8;
        data[2] = ((addr >> 8) & 0xFF) as u8;
        data[3] = (addr & 0xFF) as u8;

        self.spi
            .transaction(&mut [Operation::Write(&data), Operation::Write(payload)])
            .map_err(Error::Comm)?;
        Ok(())
    }

    fn write_data(&mut self, payload: &[u8]) -> Result<(), Self::Error> {
        self.spi.write(payload).map_err(Error::Comm)
    }
}
