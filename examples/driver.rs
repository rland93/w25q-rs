#![no_main]
#![no_std]

use cortex_m_rt::entry;
use defmt::{debug, info};
use defmt_rtt as _;
use embedded_hal_bus::spi::ExclusiveDevice;
use panic_probe as _;
use stm32f4xx_hal::{prelude::*, spi};

#[entry]
fn main() -> ! {
    let dp = stm32f4xx_hal::pac::Peripherals::take().unwrap();
    let _cp = cortex_m::Peripherals::take().unwrap();

    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.freeze();

    let gpioa = dp.GPIOA.split();

    let mut delay = dp.TIM2.delay_ms(&clocks);

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

    let mut w25q_dev = w25q::W25Q::new_with_spi(spidev, delay);

    loop {
        // Read Unique ID
        let unique_id = w25q_dev.read_unique_id().unwrap();
        debug!("Unique ID: {:x} {:x}", unique_id.0, unique_id.1);
        // Read JEDEC ID
        let jedec_id = w25q_dev.read_jedec_id().unwrap();
        assert!(jedec_id.0 == 0xEF);
        debug!(
            "JEDEC ID: {:x} {:x} {:x}",
            jedec_id.0, jedec_id.1, jedec_id.2
        );
        let sfdp = w25q_dev.read_sfdp().unwrap();
        debug!("SFDP: {:?}", sfdp);

        // Read status registers
        // SR1
        let sr1 = w25q_dev.read_sr1().unwrap();
        #[cfg(feature = "defmt")]
        debug!("SR1: {:?}", sr1);

        // SR2
        let sr2 = w25q_dev.read_sr2().unwrap();
        #[cfg(feature = "defmt")]
        debug!("SR2: {:?}", sr2);

        // SR3
        let sr3 = w25q_dev.read_sr3().unwrap();
        #[cfg(feature = "defmt")]
        debug!("SR3: {:?}", sr3);

        // address range to test
        let start_address = 0x000000;
        let end_address = 0x000500;
        let step = w25q::PAGE_SIZE; // step by page size

        // buffer to hold reads
        let mut buf = [0x55; 256];

        // erase test sector
        w25q_dev
            .erase_sector(start_address, w25q::SECTOR_SIZE)
            .unwrap();

        info!(
            "Erased {} - {}",
            start_address,
            start_address + w25q::SECTOR_SIZE
        );

        // read and write test. the page number (address / 256) is written to
        // the first byte of each page.

        for address in (start_address..=end_address).step_by(step) {
            w25q_dev.read(address, &mut buf).unwrap();
            info!("Read: {:x}, first 8 bytes:  {:02x}", address, &buf[0..8]);
            assert!(buf[0] == 0xFF);

            let mut write = [0xFF; 256];
            write[0] = (address >> 8) as u8;
            for i in 1..5 {
                write[i] = i as u8;
            }
            w25q_dev.write(address, &write).unwrap();
            info!("Write: {:x} {:02x}", address, write[0]);

            // read back
            w25q_dev.read(address, &mut buf).unwrap();
            info!("Read: {:x}, first 8 bytes: {:02x}", address, &buf[0..8]);
            assert!(buf[0] == write[0]);
        }

        w25q_dev.delay.delay_ms(1000);
    }
}
