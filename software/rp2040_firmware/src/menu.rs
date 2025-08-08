use crate::display;
use crate::Button;
use crate::DevicesCore0;
use embassy_time::Instant;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::pixelcolor::BinaryColor;

const ONE_HUNDRED_PERCENT: u64 = 100;

// Bind menu text to an optional Enum of type T
//
pub struct MenuBinding<'a, T: Clone> {
    text: &'a str,
    binding: Option<T>,
}

// Implement new to create a MenuBinding
//
impl<'a, T: Clone> MenuBinding<'a, T> {
    pub const fn new(text: &'a str, binding: Option<T>) -> Self {
        Self { text, binding }
    }
}

///
/// Draw a menu at a position
///
/// menu_items - A slice of menu bindings that contain the menu text
/// menu_item - The menu item the display should focus on
/// percent_offset_from_target - How much should we offset the display from the target.  Used for transitions between menu items.  Value inputs are -100 to 100.
/// devices - The device to display to.
///
fn draw_menu<T: Clone>(
    menu_items: &[MenuBinding<T>],
    menu_item: usize,
    percent_offset_from_target: i32,
    devices: &mut DevicesCore0<'_>,
) {
    const GAP: i32 = 16;
    const MID: i32 = 18;
    let main_loc = MID - GAP * percent_offset_from_target / (ONE_HUNDRED_PERCENT as i32);
    let max_items = menu_items.len();

    devices.display.clear(BinaryColor::Off).unwrap();

    // Display the target in bold
    //
    display::draw_text(
        &mut devices.display,
        menu_items[menu_item].text,
        main_loc,
        true, // bold
    );
    // Display menu item above the target
    //
    if menu_item > 0 {
        display::draw_text(
            &mut devices.display,
            menu_items[menu_item - 1].text,
            main_loc - GAP,
            false, // not bold
        );
    }
    // Display menu item below the target
    //
    if menu_item + 1 < max_items {
        display::draw_text(
            &mut devices.display,
            menu_items[menu_item + 1].text,
            main_loc + GAP,
            false, // not bold
        );
    }
    devices.display.flush().unwrap();
}

pub async fn transition_to_new_target_pos<T: Clone>(
    menu_items: &[MenuBinding<'_, T>],
    devices: &mut DevicesCore0<'_>,
    target_pos: usize,
    direction: i32,
) {
    const TRANSITION_TIME: u64 = 200;
    let start_time = Instant::now();
    while start_time.elapsed().as_millis() < TRANSITION_TIME {
        let percent_into_transition: i32 =
            ((start_time.elapsed().as_millis()) * ONE_HUNDRED_PERCENT / TRANSITION_TIME)
                .try_into()
                .unwrap();
        let percent_offset_from_target =
            ((ONE_HUNDRED_PERCENT as i32) - percent_into_transition) * direction;
        draw_menu(menu_items, target_pos, percent_offset_from_target, devices);
    }
}

pub async fn run_menu<T: Clone>(
    menu_items: &[MenuBinding<'_, T>],
    up_menu: T,
    start_pos: Option<usize>,
    devices: &mut DevicesCore0<'_>,
) -> (T, usize) {
    let mut current_pos: usize = if start_pos.is_some() {
        start_pos.unwrap()
    } else {
        0
    };
    let max_items = menu_items.len();

    loop {
        draw_menu(menu_items, current_pos, 0, devices);
        let button = devices.buttons.wait_for_press().await;
        if button == Button::B0 {
            // Exit out of the menu
            return (up_menu, current_pos);
        }
        if button == Button::B3 && menu_items[current_pos].binding.is_some() {
            // Menu item selected.  Return the binding if it exists.
            return (
                menu_items[current_pos].binding.as_ref().unwrap().clone(),
                current_pos,
            );
        }
        if button == Button::B2 && current_pos + 1 < max_items {
            // "Down arrow" button.
            current_pos = current_pos + 1;
            transition_to_new_target_pos(menu_items, devices, current_pos, -1).await;
        }
        if button == Button::B1 && current_pos > 0 {
            // "Up arrow" button.
            current_pos = current_pos - 1;
            transition_to_new_target_pos(menu_items, devices, current_pos, 1).await;
        }
    }
}
