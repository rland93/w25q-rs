#![no_main]
#![no_std]
#![cfg(feature = "littlefs2")]

use cortex_m_rt::entry;
use defmt::{debug, info};
use defmt_rtt as _;
use embedded_hal_bus::spi::ExclusiveDevice;
use littlefs2::{fs::Filesystem, io::SeekFrom, path::PathBuf};
use panic_probe as _;
use stm32f4xx_hal::{prelude::*, spi};

#[entry]
fn main() -> ! {
    let dp = stm32f4xx_hal::pac::Peripherals::take().unwrap();
    let _cp = cortex_m::Peripherals::take().unwrap();

    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.freeze();

    let gpioa = dp.GPIOA.split();

    let delay2 = dp.TIM2.delay_ms(&clocks);
    let mut delay3 = dp.TIM3.delay_ms(&clocks);

    info!("spi");
    let spi1 = spi::Spi1::new(
        dp.SPI1,
        (
            gpioa.pa5.into_alternate(),
            gpioa.pa6.into_alternate(),
            gpioa.pa7.into_alternate(),
        ),
        spi::Mode {
            polarity: spi::Polarity::IdleLow,
            phase: spi::Phase::CaptureOnFirstTransition,
        },
        1.MHz(),
        &clocks,
    );

    let cs = gpioa.pa4.into_push_pull_output();

    let spidev = ExclusiveDevice::new_no_delay(spi1, cs).unwrap();

    let mut w25q_dev = w25q::W25Q::new_with_spi(spidev, delay2);

    Filesystem::format(&mut w25q_dev).unwrap();
    let mut alloc = Filesystem::allocate();
    let mut fs = Filesystem::mount(&mut alloc, &mut w25q_dev).unwrap();

    loop {
        // Read Unique ID
        let unique_id = unsafe { fs.borrow_storage_mut().read_unique_id().unwrap() };
        debug!("Unique ID: {:x} {:x}", unique_id.0, unique_id.1);

        let mut buf = [0u8; 11];
        fs.open_file_with_options_and_then(
            |options| options.read(true).write(true).create(true),
            &PathBuf::from(b"example.txt"),
            |file| {
                file.write(b"Why is black smoke coming out?!")?;
                file.seek(SeekFrom::End(-24)).unwrap();
                assert_eq!(file.read(&mut buf)?, 11);
                Ok(())
            },
        )
        .unwrap();
        assert_eq!(&buf, b"black smoke");

        delay3.delay_ms(150000);
    }
}
