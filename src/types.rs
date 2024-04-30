///
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, PartialEq)]
pub enum SR {
    SR1(SR1),
    SR2(SR2),
    SR3(SR3),
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, PartialEq, Default)]
pub struct SR1 {
    pub srp0: bool,
    pub sec: bool,
    pub tb: bool,
    pub bp: u8,
    pub wel: bool,
    pub busy: bool,
}

impl SR1 {
    fn from_byte(byte: u8) -> Self {
        Self {
            srp0: (byte & 0b1000_0000) != 0,
            sec: (byte & 0b0100_0000) != 0,
            tb: (byte & 0b0010_0000) != 0,
            bp: (byte & 0b0001_1100) >> 2u8,
            wel: (byte & 0b0000_0010) != 0,
            busy: (byte & 0b0000_0001) != 0,
        }
    }
    fn to_byte(&self) -> u8 {
        (self.srp0 as u8) << 7u8
            | (self.sec as u8) << 6u8
            | (self.tb as u8) << 5u8
            | (self.bp & 0b111) << 2u8
            | (self.wel as u8) << 1u8
            | self.busy as u8
    }
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, PartialEq, Default)]
pub struct SR2 {
    pub sus: bool,
    pub cmp: bool,
    pub lb: u8,
    pub qe: bool,
    pub srp1: bool,
}
impl SR2 {
    fn from_byte(byte: u8) -> Self {
        Self {
            sus: byte & 0b1000_0000 != 0,
            cmp: byte & 0b0100_0000 != 0,
            lb: (byte >> 5u8) & 0b111,
            qe: byte & 0b0000_0010 != 0,
            srp1: byte & 0b0000_0001 != 0,
        }
    }

    fn to_byte(&self) -> u8 {
        (self.sus as u8) << 7u8
            | (self.cmp as u8) << 6u8
            | (self.lb & 0b111) << 5u8
            | (self.qe as u8) << 1u8
            | self.srp1 as u8
    }
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, PartialEq, Default)]
pub struct SR3 {
    pub hold_or_reset: bool,
    pub driver_strength: u8,
    pub wps: bool,
}
impl SR3 {
    fn from_byte(byte: u8) -> Self {
        Self {
            hold_or_reset: byte & 0b1000_0000 != 0,
            driver_strength: (byte >> 5) & 0b11, // Get S22 and S21
            wps: byte & 0b0000_0100 != 0,
        }
    }

    fn to_byte(&self) -> u8 {
        (self.hold_or_reset as u8) << 7 | (self.driver_strength & 0b11) << 5 | (self.wps as u8) << 2
    }
}
