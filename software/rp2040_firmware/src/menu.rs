use crate::Devices;
use embedded_graphics::draw_target::DrawTarget;
use crate::display;
use crate::Button;
use embedded_graphics::pixelcolor::BinaryColor;
use crate::Timer;

fn draw_menu(menu_items: &[&str], devices: &mut Devices<'_>, menu_item: usize, percent: i32 )
{
    const GAP: i32 = 16;
    const MID: i32 = 18;
    let main_loc = MID - GAP * percent / 100;
    let max_items = menu_items.len();

    devices.display.clear(BinaryColor::Off).unwrap();
    display::draw_text(&mut devices.display, menu_items[menu_item], main_loc, true );
    if menu_item > 0 {
        display::draw_text(&mut devices.display, menu_items[menu_item-1], main_loc - GAP, false );
    }
    if menu_item+1 < max_items {
        display::draw_text(&mut devices.display, menu_items[menu_item+1], main_loc + GAP, false );
    }
    devices.display.flush().unwrap();
}

pub async fn transition_upwards(menu_items: &[&str], devices: &mut Devices<'_>, new_pos: usize )
{
    let start_time = Timer::ms_from_start() as u32;
    let mut current_time = start_time;
    while current_time - start_time < 200 {
        let percent : i32 = (100 - (current_time - start_time ) / 2).try_into().unwrap();
        draw_menu(menu_items, devices, new_pos, percent);
        current_time = Timer::ms_from_start() as u32;
    }
}

pub async fn transition_downwards(menu_items: &[&str], devices: &mut Devices<'_>, new_pos: usize )
{
    let start_time = Timer::ms_from_start() as u32;
    let mut current_time = start_time;
    while current_time - start_time < 200 {
        let percent : i32 = ((current_time - start_time ) / 2).try_into().unwrap();
        draw_menu(menu_items, devices, new_pos, percent);
        current_time = Timer::ms_from_start() as u32;
    }
}


pub async fn run_menu(menu_items: &[&str], devices: &mut Devices<'_> ) -> Option<usize> 
{
    let mut current_pos : usize = 0;
    let max_items = menu_items.len();

    loop {
        draw_menu(menu_items, devices, current_pos, 0 );
        let button = devices.buttons.wait_for_press().await;
        if button == Button::B0 {
            return None;
        }
        if button == Button::B3 {
            return Some( current_pos );
        }
        if button == Button::B1 && current_pos > 0 {
            current_pos = current_pos - 1;
            transition_upwards( menu_items, devices, current_pos ).await;
        }
        if button == Button::B2 && current_pos + 1 < max_items {
            transition_downwards( menu_items, devices, current_pos ).await;
            current_pos = current_pos + 1;
        }
    }
}

