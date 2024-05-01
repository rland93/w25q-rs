use crate::interface::{ReadData, WriteData};
use crate::registers::Register;
use crate::{Error, W25Q};
use embedded_hal::delay::DelayNs;

impl<DI, CommE, D> W25Q<DI, D>
where
    DI: ReadData<Error = Error<CommE>> + WriteData<Error = Error<CommE>>,
    D: DelayNs,
{
    pub fn read_unique_id(&mut self) -> Result<(u32, u32), Error<CommE>> {
        // register + 4 dummy bytes + 8 bytes of data
        let mut data = [0xFF; 12];

        self.iface
            .read_data(Register::READ_UNIQUE_ID as u8, &mut data)?;
        let u32_1 = (data[4] as u32) << 24
            | (data[5] as u32) << 16
            | (data[6] as u32) << 8
            | (data[7] as u32);
        let u32_2 = (data[8] as u32) << 24
            | (data[9] as u32) << 16
            | (data[10] as u32) << 8
            | (data[11] as u32);

        Ok((u32_1, u32_2))
    }

    pub fn read_jedec_id(&mut self) -> Result<(u8, u8, u8), Error<CommE>> {
        let mut data = [0xFF; 3];

        self.iface.read_data(Register::JEDEC_ID as u8, &mut data)?;

        Ok((data[0], data[1], data[2]))
    }

    pub fn read_sfdp(&mut self) -> Result<[u8; 256], Error<CommE>> {
        let mut data = [0xFF; 256];
        let addr = 0x00000000;

        self.iface
            .read_from_addr(Register::READ_SFDP_REGISTER as u8, addr, &mut data)?;

        Ok(data)
    }

    pub fn write_enable(&mut self) -> Result<(), Error<CommE>> {
        let reg = Register::WRITE_ENABLE as u8;
        self.iface.write_data(&[reg])?;
        Ok(())
    }

    pub fn read(&mut self, off: usize, buf: &mut [u8]) -> Result<usize, Error<CommE>> {
        if off >= self.total_size() {
            return Err(Error::AddressSize);
        }

        let max_possible = self.total_size() - off;
        let read_size = buf.len().min(max_possible);

        self.iface
            .read_from_addr(Register::READ_DATA as u8, off as u32, &mut buf[..read_size])
            .map_err(|e| e.into())?;

        Ok(read_size)
    }

    pub fn write(&mut self, off: usize, data: &[u8]) -> Result<usize, Error<CommE>> {
        // offset is within the flash size
        if off >= self.total_size() {
            return Err(Error::AddressSize);
        }

        let mut total_written = 0;
        let mut current_offset = off;
        let mut data_left = data;

        while !data_left.is_empty() {
            let page_offset = current_offset % self.page_size();
            let max_write_size = self.page_size() - page_offset;
            let write_size = min(max_write_size, data_left.len());

            self.iface.write_addr(
                Register::PAGE_PROGRAM as u8,
                current_offset as u32,
                &data_left[..write_size],
            )?;

            // update counters
            total_written += write_size;
            current_offset += write_size;
            data_left = &data_left[write_size..]; // Move the slice forward
        }

        Ok(total_written)
    }

    pub fn erase(&mut self, off: usize, len: usize) -> Result<usize, Error<CommE>> {
        if off >= self.total_size() {
            return Err(Error::AddressSize);
        }

        let end = (off + len).min(self.total_size());
        let start_sector = off / self.sector_size();
        let end_sector = (end + self.sector_size()) / self.sector_size();

        for sector_index in start_sector..end_sector {
            let sector_addr = sector_index * self.sector_size();
            self.iface
                .write_addr(Register::SECTOR_ERASE as u8, sector_addr as u32, &[])?;
        }
        loop {
            self.delay.delay_ms(45);
            // check register
        }

        Ok((end_sector - start_sector) * 4096)
    }

    pub fn sr1(&mut self) -> Result<u8, Error<CommE>> {
        let mut data = [0xFF];
        self.iface
            .read_data(Register::READ_STATUS_REGISTER_1 as u8, &mut data)?;
        Ok(data[0])
    }

    /// Attribute getters
    //////////////////////////////////////////

    fn total_size(&self) -> usize {
        return self.config.total_size;
    }

    fn page_size(&self) -> usize {
        return self.config.page_size;
    }

    fn sector_size(&self) -> usize {
        return 4096;
    }
}

fn min(a: usize, b: usize) -> usize {
    if a < b {
        a
    } else {
        b
    }
}
