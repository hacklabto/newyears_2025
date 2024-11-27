#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_rp::dma::Channel;
use embassy_rp::peripherals::PIO0;
use embassy_rp::pio::{InterruptHandler, Pio};
use embassy_rp::{bind_interrupts, Peripheral};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

// PIO State machines all have IRQ flags they can set/wait on, so I think
// that's why these are necessary? The Pio::new function doesn't actually use,
// these, so not sure.
bind_interrupts!(struct PioIrqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Init peripherals
    let p = embassy_rp::init(Default::default());
    // Assign IRQs, get state machine to program
    let Pio {
        mut common, sm0, ..
    } = Pio::new(p.PIO0, PioIrqs);

    let led_data_pin = p.PIN_0;
    let led_clk_pin = p.PIN_1;
    let led_latch_pin = p.PIN_2;
    let led_clear_pin = p.PIN_3;

    let mut pio_backlight = hackernewyears::backlight::PioBacklight::new(
        &mut common,
        sm0,
        hackernewyears::backlight::Config {
            rows: 7,
            max_row_pixels: 19,
            num_intensity_levels: 255,
        },
        led_data_pin,
        led_clk_pin,
        led_latch_pin,
        led_clear_pin,
    );
    pio_backlight.start();

    let mut dma_out_ref = p.DMA_CH0.into_ref();

    let dout_off = [0b0000_0000_0000_0000u32; 1];
    let dout_on = [0b1111_1111_1111_1111u32; 1];
    loop {
        pio_backlight
            .sm
            .tx()
            .dma_push(dma_out_ref.reborrow(), &dout_off)
            .await;
        Timer::after_millis(1000).await;
        pio_backlight
            .sm
            .tx()
            .dma_push(dma_out_ref.reborrow(), &dout_on)
            .await;
        Timer::after_millis(1000).await;

        // Just and example to show how to poll the DMA
        while dma_out_ref.reborrow().regs().ctrl_trig().read().busy() {}
    }
}
