#![no_std]
#![no_main]

use embedded_graphics::pixelcolor::BinaryColor;
use heapless::String;
use embedded_graphics::draw_target::DrawTarget;
use hackernewyears::Button;

use defmt_rtt as _;
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
    let p = embassy_rp::init(Default::default());

    let mut devices = hackernewyears::Devices::new(
        p.PIN_2, // button 0
        p.PIN_3, // button 1
        p.PIN_4, // button 2
        p.PIN_5, // button 3
        p.I2C0,  // Display I2C channel
        p.PIN_17, // Display I2C SCLR
        p.PIN_16, // Display I2C SDA
        p.PWM_SLICE0,   // Sound PWM slice
        p.PIN_0,    // Sound "a" output
        p.PIN_1,    // Sound "b" output
    );

    hackernewyears::menu::run_menu(
        &[ "Main Menu", "Eyes Animated Gif", "Ode to Joy" ], &mut devices
    ).await;

    let mut ticker = embassy_time::Ticker::every(embassy_time::Duration::from_millis(200));
    loop {
        devices.display.clear(BinaryColor::Off).unwrap();

        //animating_gif.update( &mut display );
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
