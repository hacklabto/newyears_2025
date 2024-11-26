//! Abstraction wrapper for the devices used by hacker new year

use crate::Buttons;
use crate::DisplaySSD;
use crate::display;

pub struct Devices<'a> {
    pub buttons: Buttons<'a>,
    pub display: DisplaySSD<'a>,
}

impl Devices<'_> {
    pub fn new(
        button_0: embassy_rp::peripherals::PIN_2,
        button_1: embassy_rp::peripherals::PIN_3,
        button_2: embassy_rp::peripherals::PIN_4,
        button_3: embassy_rp::peripherals::PIN_5,
        display_i2c: embassy_rp::peripherals::I2C0,
        display_sclr: embassy_rp::peripherals::PIN_17,
        display_sda: embassy_rp::peripherals::PIN_16,
    ) -> Self {
        let buttons = Buttons::new( button_0, button_1, button_2, button_3 );

        let display = display::create_ssd_display(
            display_i2c, display_sclr, display_sda );

        Self{ buttons, display }
    }
}

