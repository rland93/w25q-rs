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

impl From<u8> for SR1 {
    fn from(byte: u8) -> Self {
        Self {
            srp0: (byte & 0b1000_0000) != 0,
            sec: (byte & 0b0100_0000) != 0,
            tb: (byte & 0b0010_0000) != 0,
            bp: (byte & 0b0001_1100) >> 2,
            wel: (byte & 0b0000_0010) != 0,
            busy: (byte & 0b0000_0001) != 0,
        }
    }
}

impl From<SR1> for u8 {
    fn from(sr1: SR1) -> Self {
        (sr1.srp0 as u8) << 7
            | (sr1.sec as u8) << 6
            | (sr1.tb as u8) << 5
            | (sr1.bp & 0b111) << 2
            | (sr1.wel as u8) << 1
            | sr1.busy as u8
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

impl From<u8> for SR2 {
    fn from(byte: u8) -> Self {
        Self {
            sus: byte & 0b1000_0000 != 0,
            cmp: byte & 0b0100_0000 != 0,
            lb: (byte >> 5) & 0b111,
            qe: byte & 0b0000_0010 != 0,
            srp1: byte & 0b0000_0001 != 0,
        }
    }
}

impl From<SR2> for u8 {
    fn from(sr2: SR2) -> Self {
        (sr2.sus as u8) << 7
            | (sr2.cmp as u8) << 6
            | (sr2.lb & 0b111) << 5
            | (sr2.qe as u8) << 1
            | sr2.srp1 as u8
    }
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, PartialEq, Default)]
pub struct SR3 {
    pub hold_or_reset: bool,
    pub driver_strength: u8,
    pub wps: bool,
}

impl From<u8> for SR3 {
    fn from(byte: u8) -> Self {
        Self {
            hold_or_reset: byte & 0b1000_0000 != 0,
            driver_strength: (byte >> 5) & 0b11, // Capture the two bits for driver_strength
            wps: byte & 0b0000_0100 != 0,
        }
    }
}

impl From<SR3> for u8 {
    fn from(sr3: SR3) -> Self {
        (sr3.hold_or_reset as u8) << 7 | (sr3.driver_strength & 0b11) << 5 | (sr3.wps as u8) << 2
    }
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, PartialEq)]
pub struct FlashConfig {
    pub total_size: usize,
    pub page_size: usize,
}
