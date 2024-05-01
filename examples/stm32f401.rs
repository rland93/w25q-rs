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

    let flashcfg = w25q::FlashConfig {
        total_size: 4096 * 4096, // 16MB
        page_size: 256,
    };

    let mut w25q_dev = w25q::W25Q::new_with_spi(spidev, flashcfg, delay);

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

        w25q_dev.read(0, &mut [0u8; 256]).unwrap();
        debug!("Read: {:02x}", &mut [0u8; 256] as &[u8; 256]);

        w25q_dev.write(0, &[0xAF; 256]).unwrap();
        debug!("Write: {:02x}", &[0xAF; 256]);

        w25q_dev.read(0, &mut [0u8; 256]).unwrap();
        debug!("Read: {:02x}", &mut [0u8; 256] as &[u8; 256]);

        w25q_dev.erase(0, 4096).unwrap();
        debug!("Erase: 4096");

        delay.delay_ms(1000);
    }
}
