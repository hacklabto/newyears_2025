#![no_std]
#![no_main]

use hackernewyears::AnimatingGif;
use hackernewyears::AnimatingGifs;
use hackernewyears::menu::MenuBinding;

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
/*
        pub enum MainMenu {
            UpMenu,
            Eyes,
            Abstract
        }
*/
        let result = hackernewyears::menu::run_menu (
                &[ &MenuBinding::new("Main Menu", 0),
                   &MenuBinding::new("Eyes Animated Gif", 1),
                   &MenuBinding::new("Abstract", 2)],
                &mut devices
        ).await;
//, "Eyes Animated Gif", "Abstract" ], &mut devices

        match result {
            0 => {} ,
            1 => animating_gifs.animate(AnimatingGif::Eyes, &mut devices).await,
            2 => animating_gifs.animate(AnimatingGif::Abstract, &mut devices).await,
            3.. => todo!(),
        }
    }
}
