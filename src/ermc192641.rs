use crate::{
    arduino::{Arduino, ChipSelect},
    constants::*,
    dbarduino::DBArduino,
};
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
    x: Cell<u8>,
    y: Cell<u8>,
    chip_page: Cell<ChipSelectPage>,
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
            x: Cell::new(0),
            y: Cell::new(0),
            chip_page: Cell::new(ChipSelectPage {
                page1: 0,
                page2: 0,
                page3: 0,
            }),
        }
    }

    fn wait_ready(&mut self, chip: ChipSelect) {
        self.control_bus.select_chip(chip);
        let db_in = self.data_bus.take().unwrap().data_dir_in();
        self.control_bus.set_rwrs(1, 0);
        self.control_bus.en_strobe_high();
        self.control_bus.delay_nano_seconds(TDDR);
        while db_in.ready_busy_status() {}
        self.control_bus.en_strobe_low();
        let x = Some(db_in.data_dir_out());
        self.data_bus.set(x);
    }

    fn write_command(&mut self, cmd: u8, chip: ChipSelect) {
        self.wait_ready(chip);
        self.control_bus.set_rwrs(0, 0);
        let mut db_out = self.data_bus.take().unwrap().data_dir_out();
        db_out.write_byte(cmd);
        self.control_bus.delay_nano_seconds(TAS);
        self.control_bus.en_strobe_high();
        self.control_bus.delay_nano_seconds(TWH);
        self.control_bus.en_strobe_low();
        let x = Some(db_out.data_dir_out());
        self.data_bus.set(x);
    }

    fn xyval_to_chip(&self, x: u8, y: u8) -> u8 {
        x / CHIP_WIDTH
    }
    // #define glcdDev_XYval2Chip(x,y) ((x/CHIP_WIDTH) + ((y/CHIP_HEIGHT) * (DISPLAY_WIDTH/CHIP_WIDTH)))

    fn xval_to_chip_col(&self, x: u8) -> u8 {
        x % CHIP_WIDTH
    }
    // #define glcdDev_Xval2ChipCol(x)		((x) % CHIP_WIDTH)

    fn goto_xy(&mut self, x: u8, y: u8) {
        if (self.x.get() == x) && (self.y.get() == y) {
            ()
        }

        if (self.x.get() > DISPLAY_WIDTH - 1) || (self.y.get() > DISPLAY_HEIGHT - 1) {
            ()
        }

        self.x.set(x);
        self.y.set(y);

        let chip = ChipSelect::try_from(self.xyval_to_chip(x, y)).unwrap();

        if y / 8 != self.chip_page.get()[chip] {
            self.chip_page.get()[chip] = y / 8;
            let cmd = LCD_SET_PAGE | self.chip_page.get()[chip];
            self.write_command(cmd, chip);
        }

        let xx = self.xval_to_chip_col(x);

        let cmd = LCD_SET_ADD | xx;
        self.write_command(cmd, chip);
    }

    fn do_read_data(&mut self) -> u8 {
        let chip = ChipSelect::try_from(self.xyval_to_chip(self.x.get(), self.y.get())).unwrap();

        self.wait_ready(chip);
        self.control_bus.set_rwrs(1, 1);
        self.control_bus.delay_nano_seconds(TAS);
        self.control_bus.en_strobe_high();
        self.control_bus.delay_nano_seconds(TDDR);

        let db_in = self.data_bus.take().unwrap().data_dir_in();
        let data = db_in.read_byte();

        self.control_bus.en_strobe_low();
        self.data_bus.set(Some(db_in.data_dir_out()));
        data
    }

    fn read_data(&mut self) -> u8 {
        let x = self.x.get();
        if x >= DISPLAY_WIDTH {
            return 0;
        }

        let _d = self.do_read_data();

        let data = self.do_read_data();

        self.x.set(0); // check this out
        self.goto_xy(x, self.y.get());

        data
    }

    fn write_data(&mut self, data: u8) {
        let x = self.x.get();
        if x >= DISPLAY_WIDTH {
            return ();
        }

        let chip = ChipSelect::try_from(self.xyval_to_chip(self.x.get(), self.y.get())).unwrap();
        let yoffset = self.y.get() % 8;

        if yoffset != 0 {
            let mut display_data = self.read_data();
            self.wait_ready(chip);
            self.control_bus.set_rwrs(0, 1);
            let mut db_out = self.data_bus.take().unwrap().data_dir_out();
            self.control_bus.delay_nano_seconds(TAS);
            self.control_bus.en_strobe_high();

            display_data |= data << yoffset;

            db_out.write_byte(display_data);
            self.control_bus.delay_nano_seconds(TWH);
            self.control_bus.en_strobe_low();

            self.data_bus.set(Some(db_out.data_dir_out()));

            let ysave = self.y.get();
            if ((ysave + 8) & !7) >= DISPLAY_HEIGHT {
                self.goto_xy(self.x.get() + 1, ysave);
                ()
            }

            self.goto_xy(self.x.get(), ((ysave + 8) & !7));

            let mut display_data = self.read_data();
            self.wait_ready(chip);

            self.control_bus.set_rwrs(0, 1);
            let mut db_out = self.data_bus.take().unwrap().data_dir_out();
            self.control_bus.delay_nano_seconds(TAS);
            self.control_bus.en_strobe_high();

            display_data |= data >> (8 - yoffset);

            db_out.write_byte(display_data);

            self.control_bus.delay_nano_seconds(TWH);
            self.control_bus.en_strobe_low();
            self.goto_xy(self.x.get() + 1, ysave);

            self.data_bus.set(Some(db_out.data_dir_out()));
        } else {
            self.wait_ready(chip);

            self.control_bus.set_rwrs(0, 1);
            let mut db_out = self.data_bus.take().unwrap().data_dir_out();
            self.control_bus.delay_nano_seconds(TAS);
            self.control_bus.en_strobe_high();

            db_out.write_byte(data);

            self.control_bus.delay_nano_seconds(TWH);
            self.control_bus.en_strobe_low();
            self.data_bus.set(Some(db_out.data_dir_out()));

            self.x.set(self.x.get() + 1);

            if ChipSelect::try_from(self.xyval_to_chip(self.x.get(), self.y.get())).unwrap() != chip
            {
                if self.x.get() < DISPLAY_WIDTH {
                    let x = self.x.get();
                    self.x.set(0);
                    self.goto_xy(x, self.y.get());
                }
            }
        }
    }

    pub fn set_pixels(&mut self, x: u8, mut y: u8, x2: u8, y2: u8, color: u8) {
        let height = y2 - y + 1;
        let width = x2 - x + 1;

        let page_offset = y % 8;
        y = y - page_offset;
        let mut mask = 0xFF;

        let mut h = if height < 8 - page_offset {
            mask >>= 8 - height;
            height
        } else {
            8 - page_offset
        };
        mask <<= page_offset;

        self.goto_xy(x, y);

        for i in 0..width {
            let mut data = self.read_data();

            if color == PIXEL_ON {
                data |= mask;
            } else {
                data &= !mask;
            }

            self.write_data(data);
        }

        //        while (h + 8) <= height {
        //            h += 8;
        //            y += 8;
        //            self.goto_xy(x, y);

        //            for i in 0..width {
        //                self.write_data(color);
        //            }
        //        }

        //        if h < height {
        //            mask = !(0xFF << (height - h));
        //            self.goto_xy(x, y + 8);

        //            for i in 0..width {
        //                let mut data = self.read_data();

        //                if color == PIXEL_ON {
        //                    data |= mask;
        //                } else {
        //                    data &= !mask;
        //                }

        //                self.write_data(data);
        //            }
        //        }
    }

    pub fn init_lcd(&mut self) {
        self.control_bus.unselect_chip();
        self.control_bus.en_strobe_low();

        self.control_bus.set_rwrs(0, 0);

        arduino_hal::delay_ms(50);

        ChipSelect::iter().for_each(|chip| {
            self.write_command(LCD_ON, chip);
            self.write_command(LCD_DISP_START, chip);
        });

        self.set_pixels(0, 0, DISPLAY_WIDTH - 1, DISPLAY_HEIGHT - 1, PIXEL_OFF);
        self.goto_xy(0, 0);
    }
}
