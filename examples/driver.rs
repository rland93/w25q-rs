#![no_main]
#![no_std]

use cortex_m_rt::entry;
use defmt::info;
use defmt_rtt as _;
use embedded_hal_bus::spi::ExclusiveDevice;
use panic_probe as _;
use stm32f4xx_hal::{prelude::*, spi};
use w25q::{SR, SR1, W25Q};

#[entry]
fn main() -> ! {
    let dp = stm32f4xx_hal::pac::Peripherals::take().unwrap();
    let _cp = cortex_m::Peripherals::take().unwrap();
    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.freeze();
    let gpioa = dp.GPIOA.split();
    let delay = dp.TIM2.delay_ms(&clocks);
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
    let mut w25q_dev = W25Q::new_with_spi(spidev, delay);

    loop {
        info!("Starting test cycle");

        // Test JEDEC ID
        match w25q_dev.read_jedec_id() {
            Ok(id) => info!("JEDEC ID: {:?}", id),
            Err(_e) => info!("Failed to read JEDEC ID"),
        }

        // Test Unique ID
        match w25q_dev.read_unique_id() {
            Ok(id) => info!("Unique ID: {:?}", id),
            Err(_e) => info!("Failed to read Unique ID"),
        }

        // Test Status Register operations
        let sr1 = SR1 {
            srp0: false,
            sec: false,
            tb: false,
            bp: 0,
            wel: false,
            busy: false,
        };
        if let Err(_e) = w25q_dev.write_status_register(SR::SR1(sr1)) {
            info!("Failed to write SR1");
        }
        match w25q_dev.read_status_register(SR::SR1(SR1::default())) {
            Ok(status) => info!("SR1: 0x{:02X}", status),
            Err(_e) => info!("Failed to read SR1"),
        }

        // Test Sector Erase and Page Program
        let test_address = 0x1000; // Choose an appropriate test address
        if let Err(_e) = w25q_dev.sector_erase(test_address) {
            info!("Failed to erase sector");
        }

        let test_data = [0x55; 256]; // Test data to write
        if let Err(_e) = w25q_dev.page_program(test_address, &test_data) {
            info!("Failed to program page");
        }

        // Test Read Data
        let mut read_buffer = [0u8; 256];
        if let Err(_e) = w25q_dev.read_data(test_address, &mut read_buffer) {
            info!("Failed to read data");
        } else {
            info!(
                "Read data matches written data: {}",
                read_buffer == test_data
            );
        }

        // Test Fast Read
        let mut fast_read_buffer = [0u8; 256];
        if let Err(_e) = w25q_dev.fast_read(test_address, &mut fast_read_buffer) {
            info!("Failed to fast read data");
        } else {
            info!(
                "Fast read data matches written data: {}",
                fast_read_buffer == test_data
            );
        }

        // Test Block Erase (32KB)
        if let Err(_e) = w25q_dev.block_erase_32kb(0x8000) {
            info!("Failed to erase 32KB block");
        }

        // Test Block Erase (64KB)
        if let Err(_e) = w25q_dev.block_erase_64kb(0x10000) {
            info!("Failed to erase 64KB block");
        }

        // Test Security Register operations
        let security_address = 0x1000; // Adjust based on your specific security register address
        let security_data = [0xAA; 256];
        if let Err(_e) = w25q_dev.erase_security_register(security_address) {
            info!("Failed to erase security register");
        }
        if let Err(_e) = w25q_dev.program_security_register(security_address, &security_data) {
            info!("Failed to program security register");
        }
        let mut security_read_buffer = [0u8; 256];
        if let Err(_e) =
            w25q_dev.read_security_register(security_address, &mut security_read_buffer)
        {
            info!("Failed to read security register");
        } else {
            info!(
                "Security register data matches written data: {}",
                security_read_buffer == security_data
            );
        }

        // Test Block Lock operations
        if let Err(_e) = w25q_dev.individual_block_lock(test_address) {
            info!("Failed to lock block");
        }
        match w25q_dev.read_block_lock(test_address) {
            Ok(locked) => info!("Block locked: {}", locked),
            Err(_e) => info!("Failed to read block lock status"),
        }
        if let Err(_e) = w25q_dev.individual_block_unlock(test_address) {
            info!("Failed to unlock block");
        }

        // Test Global Block Lock/Unlock
        if let Err(_e) = w25q_dev.global_block_lock() {
            info!("Failed to perform global block lock");
        }
        if let Err(_e) = w25q_dev.global_block_unlock() {
            info!("Failed to perform global block unlock");
        }

        // Test Power-down and Release from Power-down
        if let Err(_e) = w25q_dev.power_down() {
            info!("Failed to enter power-down mode");
        }
        if let Err(_e) = w25q_dev.release_power_down() {
            info!("Failed to release from power-down");
        }

        // Test Erase/Program Suspend and Resume
        if let Err(_e) = w25q_dev.erase_program_suspend() {
            info!("Failed to suspend erase/program");
        }
        if let Err(_e) = w25q_dev.erase_program_resume() {
            info!("Failed to resume erase/program");
        }

        // Test Reset Device
        if let Err(_e) = w25q_dev.reset_device() {
            info!("Failed to reset device");
        }

        info!("Test cycle completed");

        // Delay before next iteration
        w25q_dev.delay.delay_ms(5000);
    }
}
