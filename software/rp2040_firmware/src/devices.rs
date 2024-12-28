//! Abstraction wrapper for the devices used by hacker new year
//
// This file keeps all the hardware assignments in one place. If the interface
// is changed, the pins should only need to change here and nowhere else.

use crate::backlight;
use crate::backlight::PioBacklight;
use crate::piosound;
use crate::piosound::PioSound;
use crate::display::{create_ssd_display, DisplaySSD};
use crate::Buttons;
use crate::Sound;
use embassy_rp::bind_interrupts;
use embassy_rp::pio::InterruptHandler;
use embassy_rp::pio::Pio;
use embassy_rp::Peripherals;
use embassy_rp::peripherals;

// PIO State machines all have IRQ flags they can set/wait on, so I think
// that's why these are necessary? The Pio::new function internals don't actually use these, so unsure,
// these, so not sure.
bind_interrupts!(struct BacklightPioIrqs {
    PIO0_IRQ_0 => InterruptHandler<embassy_rp::peripherals::PIO0>;
});

bind_interrupts!(struct PioSoundIrqs {
    PIO1_IRQ_0 => InterruptHandler<embassy_rp::peripherals::PIO1>;
});

pub struct Devices<'a> {
    pub display: DisplaySSD<'a, peripherals::I2C0>,
    pub buttons: Buttons<'a>,
    pub sound: Sound<peripherals::PWM_SLICE0>,
    pub backlight: backlight::PioBacklight<
        'a,
        peripherals::PIO0,
        0, /*State machine number*/
        peripherals::DMA_CH0,
    >,
    pub piosound: piosound::PioSound<
        'a,
        peripherals::PIO1,
        0, /*State machine number*/
        peripherals::DMA_CH1,
    >,
}
impl Devices<'_> {
    pub fn new(
        p: Peripherals,
    ) -> Self {

        let pio0 = Pio::new(p.PIO0, BacklightPioIrqs);
        let mut pio0_common = pio0.common;
        let pio0_sm0 = pio0.sm0;

        let pio1 = Pio::new(p.PIO1, PioSoundIrqs);
        let mut pio1_common = pio1.common;
        let pio1_sm0 = pio1.sm0;

        Self {
            sound: Sound::new(
                p.PIN_0,      // sound_a
                p.PIN_1,      // sound_b
                p.PWM_SLICE0, // sound_pwm
            ),
            buttons: Buttons::new(p.PIN_2, p.PIN_3, p.PIN_4, p.PIN_5),
            backlight: PioBacklight::new(
                backlight::Config {
                    rows: 7,
                    max_row_pixels: 19,
                    num_intensity_levels: 255,
                },
                &mut pio0_common,
                pio0_sm0,
                p.PIN_6, // LED_DATA
                p.PIN_7, // LED_CLK
                p.PIN_8, // LED_LATCH
                p.PIN_9, // LED_CLEAR
                p.DMA_CH0,
            ),
            piosound: PioSound::new(
                piosound::PioSoundConfig {},
                &mut pio1_common,
                pio1_sm0,
                p.PIN_10,   // Sound A
                p.PIN_11,   // Sound B
                p.DMA_CH1,
            ),
            display: create_ssd_display(
                p.I2C0, p.PIN_17, // SCLR
                p.PIN_16, // SDA
            ),
        }
    }
}
