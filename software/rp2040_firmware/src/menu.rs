use crate::display;
use crate::Button;
use crate::Devices;
use crate::Timer;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::pixelcolor::BinaryColor;

pub struct MenuBinding<'a, T: Clone> {
    text: &'a str,
    binding: Option<T>,
}

impl<'a, T: Clone> MenuBinding<'a, T> {
    pub const fn new(text: &'a str, binding: Option<T>) -> Self {
        Self { text, binding }
    }
}

fn draw_menu<T: Clone>(
    menu_items: &[MenuBinding<T>],
    devices: &mut Devices<'_>,
    menu_item: usize,
    percent: i32,
) {
    const GAP: i32 = 16;
    const MID: i32 = 18;
    let main_loc = MID - GAP * percent / 100;
    let max_items = menu_items.len();

    devices.display.clear(BinaryColor::Off).unwrap();
    display::draw_text(
        &mut devices.display,
        menu_items[menu_item].text,
        main_loc,
        true,
    );
    if menu_item > 0 {
        display::draw_text(
            &mut devices.display,
            menu_items[menu_item - 1].text,
            main_loc - GAP,
            false,
        );
    }
    if menu_item + 1 < max_items {
        display::draw_text(
            &mut devices.display,
            menu_items[menu_item + 1].text,
            main_loc + GAP,
            false,
        );
    }
    devices.display.flush().unwrap();
}

pub async fn transition_upwards<T: Clone>(
    menu_items: &[MenuBinding<'_, T>],
    devices: &mut Devices<'_>,
    new_pos: usize,
) {
    let start_time = Timer::ms_from_start() as u32;
    let mut current_time = start_time;
    while current_time - start_time < 200 {
        let percent: i32 = (100 - (current_time - start_time) / 2).try_into().unwrap();
        draw_menu(menu_items, devices, new_pos, percent);
        current_time = Timer::ms_from_start() as u32;
    }
}

pub async fn transition_downwards<T: Clone>(
    menu_items: &[MenuBinding<'_, T>],
    devices: &mut Devices<'_>,
    new_pos: usize,
) {
    let start_time = Timer::ms_from_start() as u32;
    let mut current_time = start_time;
    while current_time - start_time < 200 {
        let percent: i32 = ((current_time - start_time) / 2).try_into().unwrap();
        draw_menu(menu_items, devices, new_pos, percent);
        current_time = Timer::ms_from_start() as u32;
    }
}

pub async fn run_menu<T: Clone>(
    menu_items: &[MenuBinding<'_, T>],
    up_menu: T,
    devices: &mut Devices<'_>,
) -> T {
    let mut current_pos: usize = 0;
    let max_items = menu_items.len();

    loop {
        draw_menu(menu_items, devices, current_pos, 0);
        let button = devices.buttons.wait_for_press().await;
        if button == Button::B0 {
            return up_menu;
        }
        if button == Button::B3 && menu_items[current_pos].binding.is_some() {
            return menu_items[current_pos].binding.as_ref().unwrap().clone();
        }
        if button == Button::B1 && current_pos > 0 {
            current_pos = current_pos - 1;
            transition_upwards(menu_items, devices, current_pos).await;
        }
        if button == Button::B2 && current_pos + 1 < max_items {
            transition_downwards(menu_items, devices, current_pos).await;
            current_pos = current_pos + 1;
        }
    }
}
