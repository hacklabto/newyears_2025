#![no_std]
#![no_main]

use hackernewyears::AnimatingGif;
use hackernewyears::AnimatingGifs;

use defmt_rtt as _;
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
    let p = embassy_rp::init(Default::default());
    let mut devices = hackernewyears::Devices::new(p);
    let animating_gifs = AnimatingGifs::new();

    for _ in 0..5 {
        animating_gifs.animate(AnimatingGif::Logo, &mut devices).await;
    }

    loop {
        let result = hackernewyears::menu::run_menu(
                &[ "Main Menu", "Eyes Animated Gif", "Abstract" ], &mut devices
        ).await;

        match result {
            None => {} ,
            Some(0) => todo!(),
            Some(1) => animating_gifs.animate(AnimatingGif::Eyes, &mut devices).await,
            Some(2) => animating_gifs.animate(AnimatingGif::Abstract, &mut devices).await,
            Some(3..) => todo!(),
        }
    }
}
