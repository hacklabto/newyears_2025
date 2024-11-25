#![no_std]
#![no_main]

use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::mono_font::ascii::FONT_10X20;
use embedded_graphics::text::TextStyleBuilder;
use embedded_graphics::text::Alignment;
use embedded_graphics::text::LineHeight;
use embedded_graphics::text::Text;
use embedded_graphics::prelude::Point;
use embedded_graphics::Drawable;
use embedded_graphics::pixelcolor::BinaryColor;

use defmt_rtt as _;
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
    let p = embassy_rp::init(Default::default());
    let mut display = hackernewyears::display::create_ssd_display(
        p.I2C0, p.PIN_17, // SCLR
        p.PIN_16, // SDA
    );
    let mut animating_gif = hackernewyears::AnimatingGif::new();
    
    let mut leds = hackernewyears::LEDs::new(p.PIN_11, p.PIN_12, p.PIN_13);

    let mut gsound = hackernewyears::Sound::new(p.PIN_0, p.PIN_1, p.PWM_SLICE0);

    leds.set(1, 1, true);
    leds.set(1, 3, true);
    leds.set(3, 1, true);
    leds.set(3, 3, true);

    let mut ticker = embassy_time::Ticker::every(embassy_time::Duration::from_millis(200));
    loop {
        animating_gif.update( &mut display );

        // Create a new character style.
        let character_style = MonoTextStyle::new(&FONT_10X20, BinaryColor::On);

        // Create a new text style.
        let text_style = TextStyleBuilder::new()
                .alignment(Alignment::Center)
                .line_height(LineHeight::Percent(150))
                .build();

        let _ = Text::with_text_style("FOO!",
                Point::new(64,15),
                character_style,
                text_style).draw( &mut display );
        leds.update();
        gsound.update();
        ticker.next().await;
        display.flush().unwrap();
    }
}
