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
use embassy_rp::dma::Channel;
use embassy_rp::gpio;
use embassy_rp::i2c;
use embassy_rp::i2c::{SclPin, SdaPin};
use embassy_rp::peripherals;
use embassy_rp::peripherals::PIO0;
use embassy_rp::peripherals::PIO1;
use embassy_rp::pio::PioPin;
use embassy_rp::Peripheral;
use embassy_rp::Peripherals;
use gpio::{Input, Pin, Pull};

//
// Devices resources that are going to be used by Core 0.
//
pub struct Core0Resources<
    'a,
    BackLightDma0: Peripheral + Channel,
    BackLightData: PioPin,
    BackLightClk: PioPin,
    BackLightLatch: PioPin,
    BackLightClr: PioPin,
    BackLightTestData: Pin,
    BackLightTestClk: Pin,
    BackLightTestLatch: Pin,
    BackLightTestClr: Pin,
    DisplayI2C: i2c::Instance,
    DisplayI2CScl: SclPin<DisplayI2C>,
    DisplayI2CSda: SdaPin<DisplayI2C>,
> {
    pub backlight_pio: PIO1,
    pub backlight_dma0: BackLightDma0,
    pub backlight_data: BackLightData,
    pub backlight_clk: BackLightClk,
    pub backlight_latch: BackLightLatch,
    pub backlight_clr: BackLightClr,
    pub backlight_test_data: BackLightTestData,
    pub backlight_test_clk: BackLightTestClk,
    pub backlight_test_latch: BackLightTestLatch,
    pub backlight_test_clr: BackLightTestClr,
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
        BackLightDma0: Peripheral + Channel,
        BackLightData: PioPin,
        BackLightClk: PioPin,
        BackLightLatch: PioPin,
        BackLightClr: PioPin,
        BackLightTestData: Pin,
        BackLightTestClk: Pin,
        BackLightTestLatch: Pin,
        BackLightTestClr: Pin,
        DisplayI2C: i2c::Instance,
        DisplayI2CScl: SclPin<DisplayI2C>,
        DisplayI2CSda: SdaPin<DisplayI2C>,
    >
    Core0Resources<
        'a,
        BackLightDma0,
        BackLightData,
        BackLightClk,
        BackLightLatch,
        BackLightClr,
        BackLightTestData,
        BackLightTestClk,
        BackLightTestLatch,
        BackLightTestClr,
        DisplayI2C,
        DisplayI2CScl,
        DisplayI2CSda,
    >
{
    pub fn new(
        backlight_pio: PIO1,
        backlight_dma0: BackLightDma0,
        backlight_data: BackLightData,
        backlight_clk: BackLightClk,
        backlight_latch: BackLightLatch,
        backlight_clr: BackLightClr,
        backlight_test_data: BackLightTestData,
        backlight_test_clk: BackLightTestClk,
        backlight_test_latch: BackLightTestLatch,
        backlight_test_clr: BackLightTestClr,
        button_back: impl Pin,
        button_up: impl Pin,
        button_down: impl Pin,
        button_action: impl Pin,
        display_i2c_interface: DisplayI2C,
        display_i2c_scl: DisplayI2CScl,
        display_i2c_sda: DisplayI2CSda,
    ) -> Self {
        Self {
            backlight_pio,
            backlight_dma0,
            backlight_data,
            backlight_clk,
            backlight_latch,
            backlight_clr,
            backlight_test_data,
            backlight_test_clk,
            backlight_test_latch,
            backlight_test_clr,
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

pub struct Core1Resources<
    SoundDma0: Peripheral + Channel,
    SoundOut0: Pin,
    SoundOut1: Pin,
    SoundEna: Pin,
    SoundDebug: Pin,
> {
    pub sound_pio: PIO0,
    pub sound_dma_channel_0: SoundDma0,
    pub sound_out_0: SoundOut0,
    pub sound_out_1: SoundOut1,
    pub sound_ena: SoundEna,
    pub sound_debug: SoundDebug,
}

impl<
        SoundDma0: Peripheral + Channel,
        SoundOut0: Pin,
        SoundOut1: Pin,
        SoundEna: Pin,
        SoundDebug: Pin,
    > Core1Resources<SoundDma0, SoundOut0, SoundOut1, SoundEna, SoundDebug>
{
    pub fn new(
        sound_pio: PIO0,
        sound_dma_channel_0: SoundDma0,
        sound_out_0: SoundOut0,
        sound_out_1: SoundOut1,
        sound_ena: SoundEna,
        sound_debug: SoundDebug,
    ) -> Self {
        Self {
            sound_pio,
            sound_dma_channel_0,
            sound_out_0,
            sound_out_1,
            sound_ena,
            sound_debug,
        }
    }
}

pub struct DevicesCore0<'a> {
    pub display: DisplaySSD<'a, peripherals::I2C0>,
    pub buttons: Buttons<'a>,

    pub backlight: backlight::PioBacklight<'a, peripherals::DMA_CH0>,
    pub piosound: piosound::PioSound<'a, peripherals::DMA_CH1>,
}

impl DevicesCore0<'_> {
    pub fn new(p: Peripherals) -> Self {
        // Eventually, we'll need to separate whatever we get from p into
        // "core 0" resources and "core 1" resources.  This will (hopefully)
        // let me do it one sub-system at a time

        let core0_resources = Core0Resources::new(
            p.PIO1,    // Backlight PIO resource
            p.DMA_CH0, // Backlight DMA channel
            p.PIN_6,   // Backlight LED_CLK
            p.PIN_7,   // Backlight LED_DATA
            p.PIN_8,   // Backlight LED_LATCH
            p.PIN_9,   // Backlight LED_CLEAR
            p.PIN_22,  // Backlight LED_CLK
            p.PIN_23,  // Backlight LED_DATA
            p.PIN_24,  // Backlight LED_LATCH
            p.PIN_25,  // Backlight LED_CLEAR
            p.PIN_12,  // Button, Back
            p.PIN_14,  // Button, Up
            p.PIN_13,  // Button, Down
            p.PIN_15,  // Button, Action
            p.I2C0,    // Display I2C Interface
            p.PIN_1,   // Display Scl
            p.PIN_0,   // Display SDA
        );

        let core1_resources = Core1Resources::new(
            p.PIO0,    // Assign PIO0 resource to Core 1
            p.DMA_CH1, // Assign DMA Channel 1 to Core 1
            p.PIN_2,   // Sound A Pin
            p.PIN_3,   // Sound B Pin.  Must be consequtive
            p.PIN_4,   // ENA Pin, always on
            p.PIN_10,  // Debug Pin
        );

        let piosound = PioSound::new(
            core1_resources.sound_pio,
            core1_resources.sound_out_0,
            core1_resources.sound_out_1,
            core1_resources.sound_ena,
            core1_resources.sound_debug,
            core1_resources.sound_dma_channel_0,
        );

        let backlight = PioBacklight::new(
            backlight::Config {
                rows: 7,
                max_row_pixels: 19,
                num_intensity_levels: 255,
            },
            core0_resources.backlight_pio,
            core0_resources.backlight_data,
            core0_resources.backlight_clk,
            core0_resources.backlight_latch,
            core0_resources.backlight_clr,
            core0_resources.backlight_test_data,
            core0_resources.backlight_test_clk,
            core0_resources.backlight_test_latch,
            core0_resources.backlight_test_clr,
            core0_resources.backlight_dma0,
        );

        Self {
            buttons: Buttons::new(
                core0_resources.button_back,
                core0_resources.button_up,
                core0_resources.button_down,
                core0_resources.button_action,
            ),
            piosound,
            backlight,
            display: create_ssd_display(
                core0_resources.display_i2c_interface,
                core0_resources.display_i2c_scl,
                core0_resources.display_i2c_sda,
            ),
        }
    }
}
