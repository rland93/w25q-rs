use crate::registers::Register;
use crate::{Error, SR1, SR2, SR3, W25Q};
use embedded_hal::{delay, spi};

impl<P, D> W25Q<P, D>
where
    P: spi::SpiDevice,
    D: delay::DelayNs,
{
    /// read the (u32, u32) unique identifier for the chip. identifier is
    /// different for each hardware
    pub fn read_unique_id(&mut self) -> Result<(u32, u32), P::Error> {
        // register + 4 dummy bytes + 8 bytes of data
        let mut data = [0xFF; 12];

        self.read_data(Register::READ_UNIQUE_ID as u8, &mut data)?;
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

    /// read the JEDEC ID which is the same for every chip
    pub fn read_jedec_id(&mut self) -> Result<(u8, u8, u8), P::Error> {
        let mut data = [0xFF; 3];

        self.read_data(Register::JEDEC_ID as u8, &mut data)?;

        Ok((data[0], data[1], data[2]))
    }

    /// read the sfdp as bytes
    pub fn read_sfdp(&mut self) -> Result<[u8; 256], P::Error> {
        let mut data = [0xFF; 256];
        let addr = 0x00000000;

        self.read_from_address(Register::READ_SFDP_REGISTER as u8, addr, &mut data)?;

        Ok(data)
    }

    /// set the non-volatile write enable bit
    pub fn write_enable(&mut self) -> Result<(), P::Error> {
        // check first and early return if we can
        if self.can_write()? {
            return Ok(());
        }
        // otherwise enable write
        let reg = Register::WRITE_ENABLE as u8;
        self.write_data(reg, &[])?;
        Ok(())
    }

    /// check if we can write
    pub fn can_write(&mut self) -> Result<bool, P::Error> {
        let sr1 = self.read_sr1()?;
        Ok(sr1.wel)
    }

    /// read status register 1
    pub fn read_sr1(&mut self) -> Result<SR1, P::Error> {
        let mut data = [0xFF];
        self.read_data(Register::READ_STATUS_REGISTER_1 as u8, &mut data)?;
        Ok(SR1::from(data[0]))
    }

    /// read status register 2
    pub fn read_sr2(&mut self) -> Result<SR2, P::Error> {
        let mut data = [0xFF];
        self.read_data(Register::READ_STATUS_REGISTER_2 as u8, &mut data)?;
        Ok(SR2::from(data[0]))
    }

    /// read status register 3
    pub fn read_sr3(&mut self) -> Result<SR3, P::Error> {
        let mut data = [0xFF];
        self.read_data(Register::READ_STATUS_REGISTER_3 as u8, &mut data)?;
        Ok(SR3::from(data[0]))
    }
}

impl<P, D> embedded_io::ErrorType for W25Q<P, D>
where
    P: spi::SpiDevice,
    D: delay::DelayNs,
{
    type Error = embedded_io::ErrorKind;
}

impl<P, D> embedded_io::Read for W25Q<P, D>
where
    P: spi::SpiDevice,
    D: delay::DelayNs,
{
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        let max_possible = crate::TOTAL_SIZE - self.seek_ptr;
        let read_size = buf.len().min(max_possible);
        let mut total_read = 0;
        // chunked read -- page size
        while total_read < read_size {
            let page = self.seek_ptr / crate::PAGE_SIZE;
            let page_offset = self.seek_ptr % crate::PAGE_SIZE;
            let page_size = min(crate::PAGE_SIZE - page_offset, read_size - total_read);
            let mut data = [0; crate::PAGE_SIZE];
            self.read_from_address(Register::READ_DATA as u8, page as u32, &mut data)
                .map_err(|e| embedded_io::ErrorKind::BrokenPipe);
            buf[total_read..total_read + page_size]
                .copy_from_slice(&data[page_offset..page_offset + page_size]);
            total_read += page_size;
            self.seek_ptr += page_size;
        }
        Ok(total_read)
    }
}

impl<P, D> embedded_io::BufRead for W25Q<P, D>
where
    P: spi::SpiDevice,
    D: delay::DelayNs,
{
    fn fill_buf(&mut self) -> Result<&[u8], Self::Error> {
        if self.buffer_start == self.buffer_end {
            let page = self.seek_ptr / crate::PAGE_SIZE;
            let page_address = page * crate::PAGE_SIZE;
            self.buffer_start = 0;
            self.buffer_end = 0;

            // read complete page into buffer
            let mut temp = core::mem::replace(&mut self.buffer, [0; crate::PAGE_SIZE]);
            self.read_from_address(Register::READ_DATA as u8, page_address as u32, &mut temp)
                .map_err(|e| embedded_io::ErrorKind::BrokenPipe)?;
            self.buffer = temp;

            self.buffer_end = crate::PAGE_SIZE;
        }

        Ok(&self.buffer[self.buffer_start..self.buffer_end])
    }

    fn consume(&mut self, amt: usize) {
        self.buffer_start += amt;
        if self.buffer_start >= self.buffer_end {
            self.buffer_start = 0;
            self.buffer_end = 0;
        }
        self.seek_ptr += amt;
    }
}

impl<P, D> embedded_io::ReadReady for W25Q<P, D>
where
    P: spi::SpiDevice,
    D: delay::DelayNs,
{
    fn read_ready(&mut self) -> Result<bool, <Self as embedded_io::ErrorType>::Error> {
        self.read_sr1()
            .map(|sr1| sr1.busy == false)
            .map_err(|e| embedded_io::ErrorKind::BrokenPipe)
    }
}

impl<P, D> embedded_io::Seek for W25Q<P, D>
where
    P: spi::SpiDevice,
    D: delay::DelayNs,
{
    fn seek(&mut self, from: embedded_io::SeekFrom) -> Result<u64, Self::Error> {
        match from {
            embedded_io::SeekFrom::Start(offset) => {
                self.seek_ptr = offset as usize;
            }
            embedded_io::SeekFrom::End(offset) => {
                self.seek_ptr = crate::TOTAL_SIZE - offset as usize;
            }
            embedded_io::SeekFrom::Current(offset) => {
                self.seek_ptr = self.seek_ptr + offset as usize;
            }
        }
        Ok(self.seek_ptr as u64)
    }
}

impl<P, D> embedded_io::Write for W25Q<P, D>
where
    P: spi::SpiDevice,
    D: delay::DelayNs,
{
    fn write(&mut self, _: &[u8]) -> Result<usize, Self::Error> {
        todo!()
    }
    fn flush(&mut self) -> Result<(), Self::Error> {
        todo!()
    }
}

impl<P, D> embedded_io::WriteReady for W25Q<P, D>
where
    P: spi::SpiDevice,
    D: delay::DelayNs,
{
    fn write_ready(&mut self) -> Result<bool, Self::Error> {
        todo!()
    }
}

fn min(a: usize, b: usize) -> usize {
    if a < b {
        a
    } else {
        b
    }
}
