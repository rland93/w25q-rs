// littleFS2
#[cfg(feature = "littlefs2")]
use generic_array::typenum::consts::{U1024, U256};
#[cfg(feature = "littlefs2")]
use generic_array::{ArrayLength, GenericArray};
#[cfg(feature = "littlefs2")]
use littlefs2::driver;
#[cfg(feature = "littlefs2")]
use littlefs2::io;

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
