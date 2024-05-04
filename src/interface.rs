/// standard operations that are common between registers/commands.
use crate::{registers::Register, Error, W25Q};
use embedded_hal::{delay, spi};

impl<P, D> W25Q<P, D>
where
    P: spi::SpiDevice,
    D: delay::DelayNs,
{
    /// with `command` at `address`, read bytes into payload.
    pub(crate) fn read_from_address(
        &mut self,
        command: u8,
        address: u32,
        payload: &mut [u8],
    ) -> Result<u8, P::Error> {
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
    pub(crate) fn read_data(&mut self, command: u8, payload: &mut [u8]) -> Result<(), P::Error> {
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
    ) -> Result<(), P::Error> {
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
    pub(crate) fn write_data(&mut self, command: u8, payload: &[u8]) -> Result<(), P::Error> {
        self.periph.transaction(&mut [
            spi::Operation::Write(&[Register::VOLATILE_SR_WRITE_ENABLE as u8]),
            spi::Operation::Write(payload),
        ])?;

        Ok(())
    }
}
