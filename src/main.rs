#![no_std]
#![no_main]

mod arduino;
mod constants;
mod dbarduino;
mod ermc192641;

use arduino::Arduino;
use dbarduino::DBArduino;
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

    let mut db = DBArduino::new(
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

    let mut led = pins.d13.into_output();

    loop {
        led.toggle();
        arduino_hal::delay_ms(1000);
    }
}
