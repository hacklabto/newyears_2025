//! Abstraction wrapper for the devices used by hacker new year
//
// This file keeps all the hardware assignments in one place. If the interface
// is changed, the pins should only need to change here and nowhere else.

use crate::backlight;
use crate::backlight::PioBacklight;
use crate::display::{create_ssd_display, DisplaySSD};
use crate::piosound;
use crate::piosound::PioSound;
use crate::Buttons;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals;
use embassy_rp::pio::InterruptHandler;
use embassy_rp::pio::Pio;
use embassy_rp::Peripherals;

// PIO State machines all have IRQ flags they can set/wait on, so I think
// that's why these are necessary? The Pio::new function internals don't actually use these, so unsure,
// these, so not sure.

bind_interrupts!(struct PioIrqs {
    PIO0_IRQ_0 => InterruptHandler<embassy_rp::peripherals::PIO0>;
});

pub struct Devices<'a> {
    #[cfg(feature = "workingssd")]
    pub display: DisplaySSD<'a, peripherals::I2C0>,
    pub buttons: Buttons<'a>,

    pub backlight: backlight::PioBacklight<
        'a,
        peripherals::PIO0,
        0, /*State machine number*/
        peripherals::DMA_CH0,
    >,

    pub piosound: piosound::PioSound<
        'a,
        peripherals::PIO0,
        1, /*State machine number*/
        peripherals::DMA_CH1,
    >,
}
impl Devices<'_> {
    pub fn new(p: Peripherals) -> Self {
        let pio = Pio::new(p.PIO0, PioIrqs);
        let mut pio_common = pio.common;
        let pio_sm0 = pio.sm0;
        let pio_sm1 = pio.sm1;

        Self {
            buttons: Buttons::new(
                p.PIN_12,       // TODO, left, down, up, etc. 
                p.PIN_13, 
                p.PIN_14, 
                p.PIN_15
            ),

            backlight: PioBacklight::new(
                backlight::Config {
                    rows: 7,
                    max_row_pixels: 19,
                    num_intensity_levels: 255,
                },
                &mut pio_common,
                pio_sm0,
                p.PIN_23, // LED_DATA
                p.PIN_22, // LED_CLK
                p.PIN_24, // LED_LATCH
                p.PIN_25, // LED_CLEAR
                p.DMA_CH0,
            ),

            piosound: PioSound::new(
                &mut pio_common,
                pio_sm1,
                p.PIN_2,  // Sound A
                p.PIN_3,  // Sound B.  Must be consequtive
                p.PIN_4,  // Ena
                p.PIN_9, // Debug
                p.DMA_CH1,
            ),
            #[cfg(feature = "workingssd")]
            display: create_ssd_display(
                p.I2C0, p.PIN_1, // SCL Serial Clock 
                p.PIN_0, // SDA Serial Data
                //p.I2C0, p.PIN_17, // SCL Serial Clock 
                //p.PIN_16, // SDA Serial Data
            ),
        }
    }
}
