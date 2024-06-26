#![allow(non_camel_case_types)]
#![allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy, PartialEq)]

pub enum Register {
    WRITE_ENABLE = 0x06,
    VOLATILE_SR_WRITE_ENABLE = 0x50,
    WRITE_DISABLE = 0x04,
    RELEASE_POWER_DOWN = 0xAB,
    MANUFACTURER_DEVICE_ID = 0x90,
    JEDEC_ID = 0x9F,
    READ_UNIQUE_ID = 0x4B,
    READ_DATA = 0x03,
    FAST_READ = 0x0B,
    PAGE_PROGRAM = 0x02,
    SECTOR_ERASE = 0x20,
    BLOCK_ERASE_32 = 0x52,
    BLOCK_ERASE_64 = 0xD8,
    CHIP_ERASE = 0xC7,
    CHIP_ERASE_2 = 0x60,
    READ_STATUS_REGISTER_1 = 0x05,
    WRITE_STATUS_REGISTER_1 = 0x01,
    READ_STATUS_REGISTER_2 = 0x35,
    WRITE_STATUS_REGISTER_2 = 0x31,
    READ_STATUS_REGISTER_3 = 0x15,
    WRITE_STATUS_REGISTER_3 = 0x11,
    READ_SFDP_REGISTER = 0x5A,
    ERASE_SECURITY_REGISTER = 0x44,
    PROGRAM_SECURITY_REGISTER = 0x42,
    READ_SECURITY_REGISTER = 0x48,
    GLOBAL_BLOCK_LOCK = 0x7E,
    GLOBAL_BLOCK_UNLOCK = 0x98,
    READ_BLOCK_LOCK = 0x3D,
    INDIVIDUAL_BLOCK_LOCK = 0x36,
    INDIVIDUAL_BLOCK_UNLOCK = 0x39,
    ERASE_PROGRAM_SUSPEND = 0x75,
    ERASE_PROGRAM_RESUME = 0x7A,
    POWER_DOWN = 0xB9,
    ENABLE_RESET = 0x66,
    RESET = 0x99,
}
