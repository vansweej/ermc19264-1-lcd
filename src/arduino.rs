use self::ChipSelect::*;
use arduino_hal::delay_ms;
use arduino_hal::port::{mode::Output, Pin};
use avr_hal_generic::port::PinOps;

pub struct Arduino<RW, RS, EN, CS1, CS2, CS3> {
    rw: Pin<Output, RW>,
    rs: Pin<Output, RS>,
    en: Pin<Output, EN>,
    cs1: Pin<Output, CS1>,
    cs2: Pin<Output, CS2>,
    cs3: Pin<Output, CS3>,
}

#[derive(Copy, Clone, PartialEq)]
pub enum ChipSelect {
    CHIP1,
    CHIP2,
    CHIP3,
}

impl ChipSelect {
    pub fn iter() -> impl Iterator<Item = ChipSelect> {
        [CHIP1, CHIP2, CHIP3].iter().copied()
    }
}

impl TryFrom<u8> for ChipSelect {
    type Error = ();

    fn try_from(i: u8) -> Result<Self, Self::Error> {
        match i {
            0 => Ok(ChipSelect::CHIP1),
            1 => Ok(ChipSelect::CHIP2),
            2 => Ok(ChipSelect::CHIP3),
            _ => Err(()),
        }
    }
}

impl<RW, RS, EN, CS1, CS2, CS3> Arduino<RW, RS, EN, CS1, CS2, CS3>
where
    RW: PinOps,
    RS: PinOps,
    EN: PinOps,
    CS1: PinOps,
    CS2: PinOps,
    CS3: PinOps,
{
    pub fn new(
        mut w: Pin<Output, RW>,
        mut s: Pin<Output, RS>,
        mut e: Pin<Output, EN>,
        mut c1: Pin<Output, CS1>,
        mut c2: Pin<Output, CS2>,
        mut c3: Pin<Output, CS3>,
    ) -> Self {
        w.set_low();
        s.set_low();
        e.set_low();
        c1.set_high();
        c2.set_high();
        c3.set_high();
        Arduino {
            rw: w,
            rs: s,
            en: e,
            cs1: c1,
            cs2: c2,
            cs3: c3,
        }
    }

    pub fn set_rwrs(&mut self, w: u8, s: u8) {
        if w == 1 {
            self.rw.set_high();
        } else {
            self.rw.set_low();
        }
        if s == 1 {
            self.rs.set_high();
        } else {
            self.rs.set_low();
        }
    }

    pub fn en_strobe_high(&mut self) {
        self.en.set_high();
    }

    pub fn en_strobe_low(&mut self) {
        self.en.set_low();
    }

    pub fn unselect_all_chips(&mut self) {
        self.cs1.set_high();
        self.cs2.set_high();
        self.cs3.set_high();
    }

    pub fn select_all_chips(&mut self) {
        self.cs1.set_low();
        self.cs2.set_low();
        self.cs3.set_low();
    }

    pub fn select_chip(&mut self, chip: ChipSelect) {
        match chip {
            ChipSelect::CHIP1 => {
                self.cs1.set_low();
            }
            ChipSelect::CHIP2 => {
                self.cs2.set_low();
            }
            ChipSelect::CHIP3 => {
                self.cs3.set_low();
            }
        }
    }
}
