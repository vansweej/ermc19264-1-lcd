#![no_std]
#![no_main]

mod arduino;
mod constants;
mod dbarduino;
mod ermc192641;

use arduino::Arduino;
use constants::{LCD_ON, PIXEL_ON};
use dbarduino::DBArduino;
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Circle, Line, Primitive, PrimitiveStyle, PrimitiveStyleBuilder, Rectangle},
    text::Text,
    Drawable, Pixel,
};
use ermc192641::Ermc192641;
use panic_halt as _;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    //    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);

    /*
     * For examples (and inspiration), head to
     *
     *     https://github.com/Rahix/avr-hal/tree/main/examples
     *
     * NOTE: Not all examples were ported to all boards!  There is a good chance though, that code
     * for a different board can be adapted for yours.  The Arduino Uno currently has the most
     * examples available.
     */

    //    ufmt::uwriteln!(&mut serial, "Hello from Arduino!\r").void_unwrap();

    let db = DBArduino::new(
        pins.d8.into_output(),
        pins.d9.into_output(),
        pins.d10.into_output(),
        pins.d11.into_output(),
        pins.d4.into_output(),
        pins.d5.into_output(),
        pins.d6.into_output(),
        pins.d7.into_output(),
    );

    let ar = Arduino::new(
        pins.a2.into_output(), // RW
        pins.a3.into_output(), // RS
        pins.a4.into_output(), // EN
        pins.a0.into_output(), // CS1
        pins.a1.into_output(), // CS2
        pins.d3.into_output(), // CS3
    );

    let mut display = Ermc192641::new(db, ar);
    display.init_lcd();

    //Line::new(Point::new(50, 20), Point::new(60, 35))
    //    .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
    //    .draw(&mut display)
    //    .unwrap();

    Rectangle::new(Point::new(0, 0), Size::new(192, 64))
        .into_styled(
            PrimitiveStyleBuilder::new()
                .stroke_color(BinaryColor::On)
                .stroke_width(1)
                .fill_color(BinaryColor::Off)
                .build(),
        )
        .draw(&mut display)
        .unwrap();

    // Create a new character style
    let style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);

    // Create a text at position (20, 30) and draw it using the previously defined style
    Text::new("Hello Rust!", Point::new(20, 30), style)
        .draw(&mut display)
        .unwrap();

    let mut led = pins.d13.into_output();

    loop {
        led.toggle();
        arduino_hal::delay_ms(1000);
    }
}
