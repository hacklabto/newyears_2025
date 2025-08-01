#![no_std]
#![no_main]

use hackernewyears::menu::MenuBinding;
use hackernewyears::AnimatingGif;
use hackernewyears::AnimatingGifs;

extern crate alloc;

use cortex_m_rt::entry;
use embedded_alloc::LlffHeap as Heap;

// For midly, probably, hopefully.
#[global_allocator]
static HEAP: Heap = Heap::empty();

use defmt_rtt as _;
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {

    use core::mem::MaybeUninit;
    const HEAP_SIZE: usize = 40*1024;
    static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
    unsafe { HEAP.init(HEAP_MEM.as_ptr() as usize, HEAP_SIZE) }

    let p = embassy_rp::init(Default::default());
    let mut devices = hackernewyears::Devices::new(p);
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
            MainMenuResult::Music => devices.piosound.play_sound().await,
        }
    }
}
