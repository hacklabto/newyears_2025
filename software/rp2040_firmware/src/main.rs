#![no_std]
#![no_main]

use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::mono_font::ascii::FONT_10X20;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::Point;
use embedded_graphics::text::Alignment;
use embedded_graphics::text::LineHeight;
use embedded_graphics::text::Text;
use embedded_graphics::text::TextStyleBuilder;
use embedded_graphics::Drawable;
use hackernewyears::devices::Devices;
use hackernewyears::Button;
use heapless::String;

use defmt_rtt as _;
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
    let p = embassy_rp::init(Default::default());
    let mut devices = Devices::new(p);

    let mut animating_gif = hackernewyears::AnimatingGif::new();

    devices.leds.set(1, 1, true);
    devices.leds.set(1, 3, true);
    devices.leds.set(3, 1, true);
    devices.leds.set(3, 3, true);

    let mut ticker = embassy_time::Ticker::every(embassy_time::Duration::from_millis(200));
    loop {
        devices.display.clear(BinaryColor::Off).unwrap();

        //animating_gif.update( &mut devices.display );
        let time = hackernewyears::Timer::ms_from_start() as u32;

        // Create a new character style.
        let character_style = MonoTextStyle::new(&FONT_10X20, BinaryColor::On);

        // Create a new text style.
        let text_style = TextStyleBuilder::new()
            .alignment(Alignment::Center)
            .line_height(LineHeight::Percent(150))
            .build();

        let mut time_as_str: String<100> = String::try_from(time).unwrap();
        time_as_str.push(' ').unwrap();
        if devices.buttons.is_pressed(Button::B0) {
            time_as_str.push('A').unwrap();
        }
        if devices.buttons.is_pressed(Button::B1) {
            time_as_str.push('B').unwrap();
        }
        if devices.buttons.is_pressed(Button::B2) {
            time_as_str.push('C').unwrap();
        }
        if devices.buttons.is_pressed(Button::B3) {
            time_as_str.push('D').unwrap();
        }
        let _ = Text::with_text_style(
            &time_as_str,
            Point::new(64, 15),
            character_style,
            text_style,
        )
        .draw(&mut devices.display);
        devices.leds.update();
        devices.gsound.update();
        ticker.next().await;
        devices.display.flush().unwrap();
    }
}
