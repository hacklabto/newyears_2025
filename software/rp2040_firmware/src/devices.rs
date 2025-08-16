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
use assign_resources::assign_resources;
use core::marker::PhantomData;
use embassy_rp::peripherals;
use embassy_rp::Peri;
use embassy_rp::Peripherals;

assign_resources! {
    core_1_resources: Core1Resources {
        sound_pio: PIO0,
        sound_dma_channel_0: DMA_CH1,
        sound_out_0: PIN_2,
        sound_out_1: PIN_3,
        sound_ena: PIN_4,
        sound_debug: PIN_10,
    }
    core_0_resources_menu: Core0ResourcesMenu {
        button_back: PIN_12,
        button_up: PIN_14,
        button_down: PIN_13,
        button_action: PIN_15,
        display_i2c_interface: I2C0,
        display_i2c_scl: PIN_1,
        display_i2c_sda: PIN_0,
    }
    core_0_resources_backlight: Core0ResourcesBacklight {
        backlight_pio: PIO1,
        backlight_dma0: DMA_CH0,
        backlight_data: PIN_6,
        backlight_clk: PIN_7,
        backlight_latch: PIN_8,
        backlight_clr: PIN_9,
        backlight_test_data: PIN_22,
        backlight_test_clk: PIN_23,
        backlight_test_latch: PIN_24,
        backlight_test_clr: PIN_25,
    }
    core_1_handle: Core1Handle {
        core_1: CORE1
    }
}

pub fn split_resources_by_core<'a>(
    p: Peripherals,
) -> (
    Core0ResourcesMenu,
    Core0ResourcesBacklight,
    Core1Resources,
    Core1Handle,
) {
    let r = split_resources!(p);
    (
        r.core_0_resources_menu,
        r.core_0_resources_backlight,
        r.core_1_resources,
        r.core_1_handle,
    )
}

pub struct DevicesCore0Menu<'a> {
    pub display: DisplaySSD<'a, peripherals::I2C0>,
    pub buttons: Buttons<'a>,
}

impl<'a> DevicesCore0Menu<'a> {
    pub fn new(core0_resources: Core0ResourcesMenu) -> Self {
        Self {
            buttons: Buttons::new(
                core0_resources.button_back,
                core0_resources.button_up,
                core0_resources.button_down,
                core0_resources.button_action,
            ),
            display: create_ssd_display(
                core0_resources.display_i2c_interface,
                core0_resources.display_i2c_scl,
                core0_resources.display_i2c_sda,
            ),
        }
    }
}

pub struct DevicesCore0Backlight<'a> {
    pub backlight: backlight::PioBacklight<'a, peripherals::DMA_CH0>,
}

impl<'a> DevicesCore0Backlight<'a> {
    pub fn new(resources: Core0ResourcesBacklight) -> Self {
        let backlight = PioBacklight::new(
            resources.backlight_pio,
            resources.backlight_data,
            resources.backlight_clk,
            resources.backlight_latch,
            resources.backlight_clr,
            resources.backlight_test_data,
            resources.backlight_test_clk,
            resources.backlight_test_latch,
            resources.backlight_test_clr,
            resources.backlight_dma0,
        );
        Self { backlight }
    }
}

pub struct DevicesCore1<'a> {
    pub piosound: piosound::PioSound<'a, peripherals::DMA_CH1>,
    _phantom: PhantomData<&'a ()>,
}

impl DevicesCore1<'_> {
    pub fn new(core1_resources: Core1Resources) -> Self {
        let piosound = PioSound::new(
            core1_resources.sound_pio,
            core1_resources.sound_out_0,
            core1_resources.sound_out_1,
            core1_resources.sound_ena,
            core1_resources.sound_debug,
            core1_resources.sound_dma_channel_0,
        );
        Self {
            piosound,
            _phantom: PhantomData,
        }
    }
}
