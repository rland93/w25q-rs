use crate::{registers::Register, Error, W25Q};
use embedded_hal::{delay, spi};

impl<P, D> W25Q<P, D>
where
    P: spi::SpiDevice,
    D: delay::DelayNs,
{
    pub(crate) fn read_from_addr(
        &mut self,
        register: u8,
        addr: u32,
        payload: &mut [u8],
    ) -> Result<u8, P::Error> {
        // [register, addr, addr, addr] MSB
        let mut data = [0; 4];
        data[0] = register;
        data[1] = ((addr >> 16) & 0xFF) as u8;
        data[2] = ((addr >> 8) & 0xFF) as u8;
        data[3] = (addr & 0xFF) as u8;

        self.periph
            .transaction(&mut [spi::Operation::Write(&data), spi::Operation::Read(payload)])?;
        Ok(data[0])
    }

    pub(crate) fn read_data(&mut self, register: u8, payload: &mut [u8]) -> Result<(), P::Error> {
        self.periph.transaction(&mut [
            spi::Operation::Write(&[register]),
            spi::Operation::Read(payload),
        ])?;
        Ok(())
    }

    pub(crate) fn write_addr(
        &mut self,
        register: u8,
        addr: u32,
        payload: &[u8],
    ) -> Result<(), P::Error> {
        // [register, addr, addr, addr] MSB
        let mut data = [0; 4];
        data[0] = register;
        data[1] = ((addr >> 16) & 0xFF) as u8;
        data[2] = ((addr >> 8) & 0xFF) as u8;
        data[3] = (addr & 0xFF) as u8;

        // write enable first
        self.periph.transaction(&mut [spi::Operation::Write(&[
            Register::VOLATILE_SR_WRITE_ENABLE as u8,
        ])])?;

        self.periph
            .transaction(&mut [spi::Operation::Write(&data), spi::Operation::Write(payload)])?;
        Ok(())
    }

    pub(crate) fn write_data(&mut self, payload: &[u8]) -> Result<(), P::Error> {
        // write enable first
        self.periph.transaction(&mut [spi::Operation::Write(&[
            Register::VOLATILE_SR_WRITE_ENABLE as u8,
        ])])?;

        self.periph.write(payload)?;

        Ok(())
    }
}
