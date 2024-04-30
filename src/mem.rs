use crate::interface::{ReadData, WriteData};
use crate::registers::Register;
use crate::{Error, W25Q};

impl<DI, CommE> W25Q<DI>
where
    DI: ReadData<Error = Error<CommE>> + WriteData<Error = Error<CommE>>,
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
        self.iface
            .read_data(Register::WRITE_ENABLE as u8, &mut [])?;
        Ok(())
    }
}
