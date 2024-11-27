//! Central struct to hold all the initialized device data structures.
// The main program will pass this around to simplify Rust ownership of
// the hardware devices in the code.
//
// This file keeps all the hardware assignments in one place. If the interface
// is changed, the pins should only need to change here and nowhere else.

use crate::backlight;
use crate::backlight::PioBacklight;
use crate::display::{create_ssd_display, DisplaySSD};
use crate::Buttons;
use crate::LEDs;
use crate::Sound;
use embassy_rp::bind_interrupts;
use embassy_rp::pio::InterruptHandler;
use embassy_rp::pio::Pio;
use embassy_rp::Peripherals;

// PIO State machines all have IRQ flags they can set/wait on, so I think
// that's why these are necessary? The Pio::new function internals don't actually use these, so unsure,
// these, so not sure.
bind_interrupts!(struct BacklightPioIrqs {
    PIO0_IRQ_0 => InterruptHandler<embassy_rp::peripherals::PIO0>;
});

pub struct Devices<'a> {
    pub display: DisplaySSD<'a, embassy_rp::peripherals::I2C0>,
    pub leds: LEDs<'a>,
    pub buttons: Buttons<'a>,
    pub gsound: Sound<embassy_rp::peripherals::PWM_SLICE0>,
    pub backlight: backlight::PioBacklight<
        'a,
        embassy_rp::peripherals::PIO0,
        0, /*State machine number*/
        embassy_rp::peripherals::DMA_CH0,
    >,
}

impl<'a> Devices<'a> {
    pub fn new(
        // The struct containing all peripherals
        p: Peripherals,
    ) -> Self {
        let Pio {
            mut common, /* pio handle */
            sm0,        /* state machine */
            ..
        } = Pio::new(p.PIO0, BacklightPioIrqs);
        Self {
            gsound: Sound::new(
                p.PIN_0,      // pin_pos
                p.PIN_1,      // pin_neg
                p.PWM_SLICE0, // pwm_slice
            ),
            buttons: Buttons::new(p.PIN_2, p.PIN_3, p.PIN_4, p.PIN_5),
            backlight: PioBacklight::new(
                backlight::Config {
                    rows: 7,
                    max_row_pixels: 19,
                    num_intensity_levels: 255,
                },
                &mut common,
                sm0,
                p.PIN_6, // LED_DATA
                p.PIN_7, // LED_CLK
                p.PIN_8, // LED_LATCH
                p.PIN_9, // LED_CLEAR
                p.DMA_CH0,
            ),
            leds: LEDs::new(
                p.PIN_11, // Clock
                p.PIN_12, // Data
                p.PIN_13, // Release
            ),
            display: create_ssd_display(
                p.I2C0, p.PIN_17, // SCLR
                p.PIN_16, // SDA
            ),
        }
    }
}
