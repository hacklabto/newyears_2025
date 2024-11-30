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

        #[derive(Copy, Clone, PartialEq)]
        pub enum MainMenu {
            UpMenu,
            Eyes,
            Abstract
        }

        let result = hackernewyears::menu::run_menu::<MainMenu> (
                &[ &MenuBinding::new("Main Menu", MainMenu::UpMenu),
                   &MenuBinding::new("Eyes Animated Gif", MainMenu::Eyes),
                   &MenuBinding::new("Abstract", MainMenu::Abstract)],
                MainMenu::UpMenu,
                &mut devices
        ).await;

        match result {
            MainMenu::UpMenu => {} ,
            MainMenu::Eyes => animating_gifs.animate(AnimatingGif::Eyes, &mut devices).await,
            MainMenu::Abstract => animating_gifs.animate(AnimatingGif::Abstract, &mut devices).await,
        }
    }
}
