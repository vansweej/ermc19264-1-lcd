use crate::{
    arduino::{Arduino, ChipSelect},
    constants::*,
    dbarduino::DBArduino,
};
use arduino_hal::delay_us;
use avr_hal_generic::port::PinOps;
use core::cell::Cell;
use core::ops::{Index, IndexMut};

#[derive(Copy, Clone)]
struct ChipSelectPage {
    page1: u8,
    page2: u8,
    page3: u8,
}

impl Index<ChipSelect> for ChipSelectPage {
    type Output = u8;

    fn index(&self, index: ChipSelect) -> &Self::Output {
        match index {
            ChipSelect::CHIP1 => &self.page1,
            ChipSelect::CHIP2 => &self.page2,
            ChipSelect::CHIP3 => &self.page3,
        }
    }
}

impl IndexMut<ChipSelect> for ChipSelectPage {
    fn index_mut(&mut self, index: ChipSelect) -> &mut Self::Output {
        match index {
            ChipSelect::CHIP1 => &mut self.page1,
            ChipSelect::CHIP2 => &mut self.page2,
            ChipSelect::CHIP3 => &mut self.page3,
        }
    }
}

// handling of ermc19264-1 display
pub struct Ermc192641<D0, D1, D2, D3, D4, D5, D6, D7, RW, RS, EN, CS1, CS2, CS3> {
    data_bus: Cell<Option<DBArduino<D0, D1, D2, D3, D4, D5, D6, D7>>>,
    control_bus: Arduino<RW, RS, EN, CS1, CS2, CS3>,
}

impl<D0, D1, D2, D3, D4, D5, D6, D7, RW, RS, EN, CS1, CS2, CS3>
    Ermc192641<D0, D1, D2, D3, D4, D5, D6, D7, RW, RS, EN, CS1, CS2, CS3>
where
    D0: PinOps,
    D1: PinOps,
    D2: PinOps,
    D3: PinOps,
    D4: PinOps,
    D5: PinOps,
    D6: PinOps,
    D7: PinOps,
    RW: PinOps,
    RS: PinOps,
    EN: PinOps,
    CS1: PinOps,
    CS2: PinOps,
    CS3: PinOps,
{
    pub fn new(
        dbus: DBArduino<D0, D1, D2, D3, D4, D5, D6, D7>,
        cbus: Arduino<RW, RS, EN, CS1, CS2, CS3>,
    ) -> Self {
        Ermc192641 {
            data_bus: Cell::new(Some(dbus)),
            control_bus: cbus,
        }
    }

    fn enable_pulse(&mut self) {
        self.control_bus.en_strobe_high();
        delay_us(5);
        self.control_bus.en_strobe_low();
        delay_us(5);
    }

    fn write_command(&mut self, data: u8) {
        self.control_bus.set_rwrs(0, 0);
        let mut db_out = self.data_bus.take().unwrap().data_dir_out();
        db_out.write_byte(data);
        self.enable_pulse();
        self.data_bus.set(Some(db_out));
    }

    fn lcd_on(&mut self) {
        self.control_bus.select_all_chips();
        self.write_command(LCD_ON);
    }

    fn lcd_off(&mut self) {
        self.control_bus.select_all_chips();
        self.write_command(LCD_OFF);
    }

    fn set_start_line(&mut self, line: u8) {
        self.control_bus.select_all_chips();
        self.write_command(LCD_START_LINE | line);
    }

    fn goto_col(&mut self, x: u8) {
        let chip = ChipSelect::try_from(x / 64).unwrap();
        let col = x - (64 * (x / 64));
        self.control_bus.select_chip(chip);
        self.write_command((LCD_SET_COLUMN | col) & 0x7F);
    }

    fn goto_row(&mut self, y: u8) {
        self.write_command((LCD_SET_ROW | y) & 0xBF);
    }

    fn goto_xy(&mut self, x: u8, y: u8) {
        self.goto_col(x);
        self.goto_row(y);
    }

    fn write(&mut self, data: u8) {
        self.control_bus.set_rwrs(0, 1);
        let mut db_out = self.data_bus.take().unwrap().data_dir_out();
        db_out.write_byte(data);
        delay_us(1);
        self.enable_pulse();
        self.data_bus.set(Some(db_out));
    }

    fn read(&mut self, x: u8) -> u8 {
        //let chip = ChipSelect::try_from(x / 64).unwrap();
        //self.control_bus.select_chip(chip);

        let db_in = self.data_bus.take().unwrap().data_dir_in();
        self.control_bus.set_rwrs(1, 1);
        delay_us(1);
        self.control_bus.en_strobe_high();
        delay_us(1);
        self.control_bus.en_strobe_low();
        delay_us(1);
        self.control_bus.en_strobe_high();
        delay_us(5);
        let data = db_in.read_byte();
        self.control_bus.en_strobe_low();
        delay_us(1);

        self.data_bus.set(Some(db_in.data_dir_out()));

        data
    }

    fn clear_line(&mut self, line: u8) {
        self.goto_xy(0, line);
        self.goto_xy(64, line);
        self.goto_xy(128, line);
        for _i in 0..64 {
            self.write(PIXEL_OFF);
        }
        self.control_bus.unselect_all_chips();
    }

    fn clear_screen(&mut self) {
        for i in 0..8 {
            self.clear_line(i);
        }
    }

    fn draw_point(&mut self, x: u8, y: u8, color: u8) {
        self.control_bus.unselect_all_chips();
        self.goto_xy(x, y / 8);
        let col = self.read(x);
        let col_new = match color {
            PIXEL_OFF => Some(!(0x01 << (y % 8)) & col),
            PIXEL_ON => Some((0x01 << (y % 8)) | col),
            _ => None,
        };
        self.goto_xy(x, y / 8);
        self.write(col_new.unwrap());
    }

    pub fn init_lcd(&mut self) {
        self.control_bus.unselect_all_chips();
        self.control_bus.en_strobe_low();

        self.control_bus.set_rwrs(0, 0);

        arduino_hal::delay_ms(50);

        self.lcd_on();
        self.set_start_line(0);
        self.clear_screen();

        for u in (0..64).step_by(6) {
            for v in (0..192).step_by(2) {
                self.draw_point(v, u, PIXEL_ON);
            }
        }
    }
}
