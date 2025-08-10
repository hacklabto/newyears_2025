#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Executor;
use hackernewyears::devices::split_resources_by_core;
use hackernewyears::devices::Core0Resources;
use hackernewyears::devices::Core1Resources;
use hackernewyears::menu::MenuBinding;
use hackernewyears::AnimatingGif;
use hackernewyears::AnimatingGifs;
use static_cell::StaticCell;

use defmt_rtt as _;
use panic_probe as _;

static EXECUTOR0: StaticCell<Executor> = StaticCell::new();

#[cortex_m_rt::entry]
fn main() -> ! {
    let p = embassy_rp::init(Default::default());
    let resources = split_resources_by_core(p);

    let executor0 = EXECUTOR0.init(Executor::new());
    executor0.run(|spawner| unwrap!(spawner.spawn(core0_task(resources))));
}

#[embassy_executor::task]
async fn core0_task(resources: (Core0Resources, Core1Resources)) {
    let (core0_resources, core1_resources) = resources;
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
