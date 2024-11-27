#![no_std]
#![no_main]

use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::draw_target::DrawTarget;
use hackernewyears::Button;
use heapless::String;

use defmt_rtt as _;
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
    let p = embassy_rp::init(Default::default());
    let mut devices = hackernewyears::Devices::new(p);

    hackernewyears::menu::run_menu(
        &[ "Main Menu", "Eyes Animated Gif", "Ode to Joy" ], &mut devices
    ).await;

    let mut ticker = embassy_time::Ticker::every(embassy_time::Duration::from_millis(200));
    loop {
        devices.display.clear(BinaryColor::Off).unwrap();

        //animating_gif.update( &mut devices.display );
        let time = hackernewyears::Timer::ms_from_start() as u32;

        let time_as_str: String<100> = String::try_from(time).unwrap();

        let mut button_status : String<100> = Default::default();
        if devices.buttons.is_pressed(Button::B0) {
            button_status.push('A').unwrap();
        }
        if devices.buttons.is_pressed(Button::B1) {
            button_status.push('B').unwrap();
        }
        if devices.buttons.is_pressed(Button::B2) {
            button_status.push('C').unwrap();
        }
        if devices.buttons.is_pressed(Button::B3) {
            button_status.push('D').unwrap();
        }
        hackernewyears::display::draw_text(&mut devices.display, time_as_str.as_str(), 15 , true );
        hackernewyears::display::draw_text(&mut devices.display, button_status.as_str(), 31, false );

        ticker.next().await;
        devices.display.flush().unwrap();
    }
}
