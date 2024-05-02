# w25q-rs
Rust Driver for Winbond W25Q SPI Flash Memory chip.

Designed for usage with `embedded-hal-bus`.

## Features
use `defmt` to add defmt::Format to datatypes.
use `littlefs2` to add support for littleFS2. The Storage trait is implemented for the device in this case.