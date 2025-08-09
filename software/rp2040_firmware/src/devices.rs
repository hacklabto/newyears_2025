//! Abstraction wrapper for the devices used by hacker new year
//
// This file keeps all the hardware assignments in one place. If the interface
// is changed, the pins should only need to change here and nowhere else.

//use crate::backlight;
//use crate::backlight::PioBacklight;
use crate::display::{create_ssd_display, DisplaySSD};
use crate::piosound;
use crate::piosound::PioSound;
use crate::Buttons;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio;
use embassy_rp::i2c::{Instance, SclPin, SdaPin};
use embassy_rp::peripherals;
use embassy_rp::pio::InterruptHandler;
use embassy_rp::pio::Pio;
use embassy_rp::Peripherals;
use gpio::{Input, Pin, Pull};

// PIO State machines all have IRQ flags they can set/wait on, so I think
// that's why these are necessary? The Pio::new function internals don't actually use these, so unsure,
// these, so not sure.

bind_interrupts!(struct PioIrqs {
    PIO0_IRQ_0 => InterruptHandler<embassy_rp::peripherals::PIO0>;
});

//
// Devices resources that are going to be used by Core 0.
//
pub struct Core0Resources<
    'a,
    DisplayI2C: Instance,
    DisplayI2CScl: SclPin<DisplayI2C>,
    DisplayI2CSda: SdaPin<DisplayI2C>,
> {
    pub button_back: Input<'a>,
    pub button_up: Input<'a>,
    pub button_down: Input<'a>,
    pub button_action: Input<'a>,
    pub display_i2c_interface: DisplayI2C,
    pub display_i2c_scl: DisplayI2CScl,
    pub display_i2c_sda: DisplayI2CSda,
}

impl<
        'a,
        DisplayI2C: Instance,
        DisplayI2CScl: SclPin<DisplayI2C>,
        DisplayI2CSda: SdaPin<DisplayI2C>,
    > Core0Resources<'a, DisplayI2C, DisplayI2CScl, DisplayI2CSda>
{
    pub fn new(
        button_back: impl Pin,
        button_up: impl Pin,
        button_down: impl Pin,
        button_action: impl Pin,
        display_i2c_interface: DisplayI2C,
        display_i2c_scl: DisplayI2CScl,
        display_i2c_sda: DisplayI2CSda,
    ) -> Self {
        Self {
            button_back: Input::new(button_back, Pull::Up),
            button_up: Input::new(button_up, Pull::Up),
            button_down: Input::new(button_down, Pull::Up),
            button_action: Input::new(button_action, Pull::Up),
            display_i2c_interface,
            display_i2c_scl,
            display_i2c_sda,
        }
    }
}

pub struct DevicesCore0<'a> {
    pub display: DisplaySSD<'a, peripherals::I2C0>,
    pub buttons: Buttons<'a>,

    /*
        pub backlight: backlight::PioBacklight<
            'a,
            peripherals::PIO0,
            0, //State machine number
            peripherals::DMA_CH0,
        >,
    */
    pub piosound: piosound::PioSound<'a, peripherals::PIO0, peripherals::DMA_CH1>,
}
impl DevicesCore0<'_> {
    pub fn new(p: Peripherals) -> Self {
        // Eventually, we'll need to separate whatever we get from p into
        // "core 0" resources and "core 1" resources.  This will (hopefully)
        // let me do it one sub-system at a time

        let core0_resources = Core0Resources::new(
            p.PIN_12, // Button, Back
            p.PIN_14, // Button, Up
            p.PIN_13, // Button, Down
            p.PIN_15, // Button, Action
            p.I2C0,   // Display I2C Interface
            p.PIN_1,  // Display Scl
            p.PIN_0,  // Display SDA
        );

        Self {
            buttons: Buttons::new(
                core0_resources.button_back,
                core0_resources.button_up,
                core0_resources.button_down,
                core0_resources.button_action,
            ),

            /*
                        backlight: PioBacklight::new(
                            backlight::Config {
                                rows: 7,
                                max_row_pixels: 19,
                                num_intensity_levels: 255,
                            },
                            &mut pio_common,
                            pio_sm0,
                            p.PIN_6,  // LED_CLK
                            p.PIN_7,  // LED_DATA
                            p.PIN_8,  // LED_LATCH
                            p.PIN_9,  // LED_CLEAR
                            p.PIN_22, // LED_CLK
                            p.PIN_23, // LED_DATA
                            p.PIN_24, // LED_LATCH
                            p.PIN_25, // LED_CLEAR
                            p.DMA_CH0,
                        ),
            */
            piosound: PioSound::new(
                Pio::new(p.PIO0, PioIrqs),
                p.PIN_2,  // Sound A
                p.PIN_3,  // Sound B.  Must be consequtive
                p.PIN_4,  // ENA, always on
                p.PIN_10, // Debug
                p.DMA_CH1,
            ),
            display: create_ssd_display(
                core0_resources.display_i2c_interface,
                core0_resources.display_i2c_scl,
                core0_resources.display_i2c_sda,
            ),
        }
    }
}
