use crate::interface::{ReadData, WriteData};
use crate::registers::Register;
use crate::{Error, SR1, SR2, SR3, W25Q};
use embedded_hal::delay::DelayNs;
#[cfg(feature = "littlefs2")]
use generic_array::typenum::consts::{U1024, U256};
#[cfg(feature = "littlefs2")]
use generic_array::{ArrayLength, GenericArray};
#[cfg(feature = "littlefs2")]
use littlefs2::driver;
#[cfg(feature = "littlefs2")]
use littlefs2::io;

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
        // check first and early return if we can
        if self.can_write()? {
            return Ok(());
        }
        // otherwise enable write
        let reg = Register::WRITE_ENABLE as u8;
        self.iface.write_data(&[reg])?;
        Ok(())
    }

    pub fn can_write(&mut self) -> Result<bool, Error<CommE>> {
        let sr1 = self.read_sr1()?;
        Ok(sr1.wel)
    }

    pub fn read(&mut self, off: usize, buf: &mut [u8]) -> Result<usize, Error<CommE>> {
        if off >= crate::TOTAL_SIZE {
            return Err(Error::AddressSize);
        }

        let max_possible = crate::TOTAL_SIZE - off;
        let read_size = buf.len().min(max_possible);

        self.iface
            .read_from_addr(Register::READ_DATA as u8, off as u32, &mut buf[..read_size])
            .map_err(|e| e.into())?;

        Ok(read_size)
    }

    pub fn write(&mut self, off: usize, data: &[u8]) -> Result<usize, Error<CommE>> {
        self.write_enable()?;

        // offset is within the flash size
        if off >= crate::TOTAL_SIZE {
            return Err(Error::AddressSize);
        }

        let mut total_written = 0;
        let mut current_offset = off;
        let mut data_left = data;

        while !data_left.is_empty() {
            let page_offset = current_offset % crate::PAGE_SIZE;
            let max_write_size = crate::PAGE_SIZE - page_offset;
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

    pub fn erase_sector(&mut self, off: usize, len: usize) -> Result<usize, Error<CommE>> {
        self.write_enable()?;

        if off >= crate::TOTAL_SIZE || len >= crate::TOTAL_SIZE {
            return Err(Error::AddressSize);
        }

        let end = (off + len).min(crate::TOTAL_SIZE);
        let start_sector = off / crate::SECTOR_SIZE;
        let end_sector = (end + crate::SECTOR_SIZE) / crate::SECTOR_SIZE;

        for sector_index in start_sector..end_sector {
            let sector_addr = sector_index * crate::SECTOR_SIZE;
            self.iface
                .write_addr(Register::SECTOR_ERASE as u8, sector_addr as u32, &[])?;
        }
        // keep busy while device erases
        loop {
            // 45 ms typical. Up to 400ms.
            self.delay.delay_ms(45);
            let sr1 = self.read_sr1()?;
            if !sr1.busy {
                break;
            }
        }

        Ok((end_sector - start_sector) * 4096)
    }

    pub fn read_sr1(&mut self) -> Result<SR1, Error<CommE>> {
        let mut data = [0xFF];
        self.iface
            .read_data(Register::READ_STATUS_REGISTER_1 as u8, &mut data)?;
        Ok(SR1::from(data[0]))
    }

    pub fn read_sr2(&mut self) -> Result<SR2, Error<CommE>> {
        let mut data = [0xFF];
        self.iface
            .read_data(Register::READ_STATUS_REGISTER_2 as u8, &mut data)?;
        Ok(SR2::from(data[0]))
    }

    pub fn read_sr3(&mut self) -> Result<SR3, Error<CommE>> {
        let mut data = [0xFF];
        self.iface
            .read_data(Register::READ_STATUS_REGISTER_3 as u8, &mut data)?;
        Ok(SR3::from(data[0]))
    }
}

fn min(a: usize, b: usize) -> usize {
    if a < b {
        a
    } else {
        b
    }
}

#[cfg(feature = "littlefs2")]
impl<DI, CommE, D> driver::Storage for W25Q<DI, D>
where
    DI: ReadData<Error = Error<CommE>> + WriteData<Error = Error<CommE>>,
    D: DelayNs,
{
    type CACHE_SIZE = U256;
    type LOOKAHEAD_SIZE = U1024;

    fn read(&mut self, off: usize, buf: &mut [u8]) -> Result<usize, io::Error> {
        W25Q::read(self, off, buf).map_err(|e| e.into())
    }

    fn write(&mut self, off: usize, data: &[u8]) -> Result<usize, io::Error> {
        W25Q::write(self, off, data).map_err(|e| e.into())
    }

    fn erase(&mut self, off: usize, len: usize) -> Result<usize, io::Error> {
        W25Q::erase_sector(self, off, len).map_err(|e| e.into())
    }

    const READ_SIZE: usize = 256;
    const WRITE_SIZE: usize = 256;
    const BLOCK_SIZE: usize = 4096;
    const BLOCK_COUNT: usize = 4096;
    const BLOCK_CYCLES: isize = 500;
}
