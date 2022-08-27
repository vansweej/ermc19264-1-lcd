pub const DISPLAY_WIDTH: u8 = 192;
pub const DISPLAY_HEIGHT: u8 = 64;
pub const CHIP_WIDTH: u8 = 64;
pub const CHIP_HEIGHT: u8 = 64;

pub const TDDR: u16 = 320; /* Data Delay time (E high to valid read data)        */
pub const TAS: u16 = 140; /* Address setup time (ctrl line changes to E HIGH   */
pub const TDSW: u16 = 200; /* Data setup time (data lines setup to dropping E)   */
pub const TWH: u16 = 450; /* E hi level width (minimum E hi pulse width)        */
pub const TWL: u16 = 450; /* E lo level width (minimum E lo pulse width)        */

pub const LCD_ON: u8 = 0x3F;
pub const LCD_OFF: u8 = 0x3E;
pub const LCD_SET_ADD: u8 = 0x40;
pub const LCD_DISP_START: u8 = 0xC0;
pub const LCD_SET_PAGE: u8 = 0xB8;

pub const PIXEL_ON: u8 = 0xFF;
pub const PIXEL_OFF: u8 = 0x00;
