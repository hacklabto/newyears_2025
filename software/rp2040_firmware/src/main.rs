#![no_std]
#![no_main]

use hackernewyears::devices::split_resources_by_core;
use hackernewyears::menu::MenuBinding;
use hackernewyears::AnimatingGif;
use hackernewyears::AnimatingGifs;

use defmt_rtt as _;
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
    let p = embassy_rp::init(Default::default());
    let (core0_resources, core1_resources) = split_resources_by_core(p);

    let mut devices = hackernewyears::DevicesCore0::new(core0_resources);
    let mut devices_1 = hackernewyears::DevicesCore1::new(core1_resources);
    let animating_gifs = AnimatingGifs::new();

    for _ in 0..5 {
        animating_gifs
            .animate(AnimatingGif::Logo, &mut devices)
            .await;
    }

    //devices.backlight.test_pattern();

    let mut current_pos: Option<usize> = None;
    loop {
        #[derive(Clone)]
        pub enum MainMenuResult {
            UpMenu,
            Eyes,
            Abstract,
            Music,
        }

        let (result, return_pos) = hackernewyears::menu::run_menu::<MainMenuResult>(
            &[
                MenuBinding::new("Main Menu", None),
                MenuBinding::new("Eyes Animated Gif", Some(MainMenuResult::Eyes)),
                MenuBinding::new("Abstract", Some(MainMenuResult::Abstract)),
                MenuBinding::new("Music", Some(MainMenuResult::Music)),
            ],
            MainMenuResult::UpMenu,
            current_pos,
            &mut devices,
        )
        .await;
        current_pos = Some(return_pos);

        match result {
            MainMenuResult::UpMenu => {} // Already at the top
            MainMenuResult::Eyes => {
                animating_gifs
                    .animate(AnimatingGif::Eyes, &mut devices)
                    .await
            }
            MainMenuResult::Abstract => {
                animating_gifs
                    .animate(AnimatingGif::Abstract, &mut devices)
                    .await
            }
            MainMenuResult::Music => devices_1.piosound.play_sound().await,
        }
    }
}
