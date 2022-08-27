use arduino_hal::port::{
    mode::{Input, Io, Output, PullUp},
    Pin,
};
use avr_hal_generic::port::PinOps;
use core::convert::From;
use embedded_hal::digital::v2::{InputPin, OutputPin, PinState};

type DBOutputPin<T> = Pin<Output, T>;
type DBInputPin<T> = Pin<Input<PullUp>, T>;

pub enum DBPort<T> {
    DBOutput(DBOutputPin<T>),
    DBInput(DBInputPin<T>),
}

impl<T> DBPort<T>
where
    T: PinOps,
{
    pub fn data_dir_in_helper(self) -> Self {
        if let DBPort::DBOutput(o) = self {
            DBPort::from(o)
        } else {
            self
        }
    }

    pub fn data_dir_out_helper(self) -> Self {
        if let DBPort::DBInput(i) = self {
            DBPort::from(i)
        } else {
            self
        }
    }
}

impl<T> OutputPin for DBPort<T>
where
    T: PinOps,
{
    type Error = ();

    fn set_low(&mut self) -> Result<(), Self::Error> {
        if let DBPort::DBOutput(o) = self {
            o.set_low();
            Ok(())
        } else {
            Err(())
        }
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        if let DBPort::DBOutput(o) = self {
            o.set_high();
            Ok(())
        } else {
            Err(())
        }
    }
}

impl<T> InputPin for DBPort<T>
where
    T: PinOps,
{
    type Error = ();

    fn is_high(&self) -> Result<bool, Self::Error> {
        if let DBPort::DBInput(i) = self {
            Ok(i.is_high())
        } else {
            Err(())
        }
    }

    fn is_low(&self) -> Result<bool, Self::Error> {
        if let DBPort::DBInput(i) = self {
            Ok(i.is_low())
        } else {
            Err(())
        }
    }
}

impl<T> From<DBOutputPin<T>> for DBPort<T>
where
    T: PinOps,
{
    fn from(op: DBOutputPin<T>) -> Self {
        DBPort::DBInput(op.into_pull_up_input())
    }
}

impl<T> From<DBInputPin<T>> for DBPort<T>
where
    T: PinOps,
{
    fn from(op: DBInputPin<T>) -> Self {
        DBPort::DBOutput(op.into_output())
    }
}

pub struct DBArduino<D0, D1, D2, D3, D4, D5, D6, D7> {
    db0: DBPort<D0>,
    db1: DBPort<D1>,
    db2: DBPort<D2>,
    db3: DBPort<D3>,
    db4: DBPort<D4>,
    db5: DBPort<D5>,
    db6: DBPort<D6>,
    db7: DBPort<D7>,
}

impl<D0, D1, D2, D3, D4, D5, D6, D7> DBArduino<D0, D1, D2, D3, D4, D5, D6, D7>
where
    D0: PinOps,
    D1: PinOps,
    D2: PinOps,
    D3: PinOps,
    D4: PinOps,
    D5: PinOps,
    D6: PinOps,
    D7: PinOps,
{
    pub fn new(
        d0: Pin<Output, D0>,
        d1: Pin<Output, D1>,
        d2: Pin<Output, D2>,
        d3: Pin<Output, D3>,
        d4: Pin<Output, D4>,
        d5: Pin<Output, D5>,
        d6: Pin<Output, D6>,
        d7: Pin<Output, D7>,
    ) -> Self {
        DBArduino {
            db0: DBPort::DBOutput(d0),
            db1: DBPort::DBOutput(d1),
            db2: DBPort::DBOutput(d2),
            db3: DBPort::DBOutput(d3),
            db4: DBPort::DBOutput(d4),
            db5: DBPort::DBOutput(d5),
            db6: DBPort::DBOutput(d6),
            db7: DBPort::DBOutput(d7),
        }
    }

    pub fn data_dir_in(self) -> DBArduino<D0, D1, D2, D3, D4, D5, D6, D7> {
        DBArduino {
            db0: self.db0.data_dir_in_helper(),
            db1: self.db1.data_dir_in_helper(),
            db2: self.db2.data_dir_in_helper(),
            db3: self.db3.data_dir_in_helper(),
            db4: self.db4.data_dir_in_helper(),
            db5: self.db5.data_dir_in_helper(),
            db6: self.db6.data_dir_in_helper(),
            db7: self.db7.data_dir_in_helper(),
        }
    }

    pub fn data_dir_out(self) -> DBArduino<D0, D1, D2, D3, D4, D5, D6, D7> {
        DBArduino {
            db0: self.db0.data_dir_out_helper(),
            db1: self.db1.data_dir_out_helper(),
            db2: self.db2.data_dir_out_helper(),
            db3: self.db3.data_dir_out_helper(),
            db4: self.db4.data_dir_out_helper(),
            db5: self.db5.data_dir_out_helper(),
            db6: self.db6.data_dir_out_helper(),
            db7: self.db7.data_dir_out_helper(),
        }
    }

    pub fn write_byte(&mut self, data: u8) {
        self.db0.set_state(PinState::from((data & 1) != 0)).unwrap();
        self.db1.set_state(PinState::from((data & 2) != 0)).unwrap();
        self.db2.set_state(PinState::from((data & 4) != 0)).unwrap();
        self.db3.set_state(PinState::from((data & 8) != 0)).unwrap();
        self.db4
            .set_state(PinState::from((data & 16) != 0))
            .unwrap();
        self.db5
            .set_state(PinState::from((data & 32) != 0))
            .unwrap();
        self.db6
            .set_state(PinState::from((data & 64) != 0))
            .unwrap();
        self.db7
            .set_state(PinState::from((data & 128) != 0))
            .unwrap();
    }

    pub fn read_byte(&self) -> u8 {
        let mut data: u8 = 0;

        if self.db0.is_high().unwrap() {
            data |= 1 << 0;
        };
        if self.db1.is_high().unwrap() {
            data |= 1 << 1;
        };
        if self.db2.is_high().unwrap() {
            data |= 1 << 2;
        };
        if self.db3.is_high().unwrap() {
            data |= 1 << 3;
        };
        if self.db4.is_high().unwrap() {
            data |= 1 << 4;
        };
        if self.db5.is_high().unwrap() {
            data |= 1 << 5;
        };
        if self.db6.is_high().unwrap() {
            data |= 1 << 6;
        };
        if self.db7.is_high().unwrap() {
            data |= 1 << 7;
        };
        data
    }

    pub fn ready_busy_status(&self) -> bool {
        self.db7.is_high().unwrap()
    }
}
