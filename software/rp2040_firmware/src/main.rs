#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Executor;
use embassy_rp::multicore::{spawn_core1, Stack};
use embassy_time::{Duration, Timer};
use hackernewyears::devices::split_resources_by_core;
use hackernewyears::devices::Core0ResourcesBacklight;
use hackernewyears::devices::Core0ResourcesMenu;
use hackernewyears::devices::Core1Resources;
use hackernewyears::menu::MenuBinding;
use hackernewyears::AnimatingGif;
use hackernewyears::AnimatingGifs;
use static_cell::StaticCell;

use {defmt_rtt as _, panic_probe as _};

//use defmt_rtt as _;
//use panic_probe as _;

static mut CORE1_STACK: Stack<32768> = Stack::new();
static EXECUTOR0: StaticCell<Executor> = StaticCell::new();
static EXECUTOR1: StaticCell<Executor> = StaticCell::new();

#[cortex_m_rt::entry]
fn main() -> ! {
    let p = embassy_rp::init(Default::default());
    let (core0_resources_menu, core0_resources_backlight, core1_resources, core1_handle) =
        split_resources_by_core(p);

    defmt::info!("Spawning Core 1");

    spawn_core1(
        core1_handle.core_1,
        unsafe { &mut *core::ptr::addr_of_mut!(CORE1_STACK) },
        move || {
            let executor1 = EXECUTOR1.init(Executor::new());
            executor1.run(|spawner| unwrap!(spawner.spawn(core1_task(core1_resources))));
        },
    );
    defmt::info!("Executing Core 0");

    let executor0 = EXECUTOR0.init(Executor::new());
    executor0.run(|spawner| {
        unwrap!(spawner.spawn(core0_task(core0_resources_menu, core0_resources_backlight)))
    });
}

#[embassy_executor::task]
async fn core0_task(
    core0_resources_menu: Core0ResourcesMenu,
    core0_resources_backlight: Core0ResourcesBacklight,
) {
    let mut devices = hackernewyears::DevicesCore0Menu::new(core0_resources_menu);
    let mut devices_backlight =
        hackernewyears::DevicesCore0Backlight::new(core0_resources_backlight);
    let animating_gifs = AnimatingGifs::new();

    for _ in 0..5 {
        animating_gifs
            .animate(AnimatingGif::Logo, &mut devices)
            .await;
    }

    for _ in 0..1000 {
        devices_backlight.backlight.display_and_update().await;
    }

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
            MainMenuResult::Music => {
                animating_gifs
                    .animate(AnimatingGif::Abstract, &mut devices)
                    .await
            }
        }
    }
}

#[embassy_executor::task]
async fn core1_task(core1_resources: Core1Resources) {
    Timer::after(Duration::from_millis(500)).await;

    let mut devices = hackernewyears::DevicesCore1::new(core1_resources);

    loop {
        devices.piosound.play_sound().await;
    }
}
