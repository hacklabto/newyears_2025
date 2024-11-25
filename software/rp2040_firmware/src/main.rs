#![no_std]
#![no_main]

use defmt_rtt as _;
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
    let p = embassy_rp::init(Default::default());
    let mut display = hackernewyears::display::create_ssd_display(
        p.I2C0, p.PIN_17, // SCLR
        p.PIN_16, // SDA
    );
    let mut animating_gif = hackernewyears::AnimatingGif::new();
    
    let mut leds = hackernewyears::LEDs::new(p.PIN_11, p.PIN_12, p.PIN_13);

    let mut gsound = hackernewyears::Sound::new(p.PIN_0, p.PIN_1, p.PWM_SLICE0);

    leds.set(1, 1, true);
    leds.set(1, 3, true);
    leds.set(3, 1, true);
    leds.set(3, 3, true);

    let mut ticker = embassy_time::Ticker::every(embassy_time::Duration::from_millis(200));
    loop {
        animating_gif.update( &mut display );
        leds.update();
        gsound.update();
        ticker.next().await;
    }
}
