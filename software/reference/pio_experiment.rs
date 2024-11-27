#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_rp::dma::Channel;
use embassy_rp::Peripheral;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let mut devices = hackernewyears::devices::Devices::new(p);
    devices.backlight.start();

    let dout_off = [0b0000_0000_0000_0000u32; 1];
    let dout_on = [0b1111_1111_1111_1111u32; 1];
    let mut ch = devices.backlight.dma_channel.into_ref();
    loop {
        devices
            .backlight
            .state_machine
            .tx()
            .dma_push(ch.reborrow(), &dout_off)
            .await;
        Timer::after_millis(1000).await;
        devices
            .backlight
            .state_machine
            .tx()
            .dma_push(ch.reborrow(), &dout_on)
            .await;
        Timer::after_millis(1000).await;

        // Just and example to show how to poll the DMA
        while ch.reborrow().regs().ctrl_trig().read().busy() {}
    }
}
