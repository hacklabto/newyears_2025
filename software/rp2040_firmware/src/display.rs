use embassy_rp::i2c;
use embassy_rp::peripherals::I2C0;
use ssd1306::mode::BufferedGraphicsMode;
use ssd1306::mode::DisplayConfig;
use ssd1306::prelude::I2CInterface;
use ssd1306::rotation::DisplayRotation;
use ssd1306::size::DisplaySize128x32;
use ssd1306::Ssd1306;
use tinygif;

use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Circle, PrimitiveStyleBuilder, Rectangle, Triangle},
};

/*
 TODO
#[derive(PartialEq, Copy, Clone)]
pub enum DisplayArt {
    Eyes,
}
*/

pub struct Display<'a> {
    display: Ssd1306<
        I2CInterface<i2c::I2c<'a, I2C0, i2c::Blocking>>,
        DisplaySize128x32,
        BufferedGraphicsMode<DisplaySize128x32>,
    >,

    eyes: tinygif::Gif<'a, BinaryColor>,

    frame: u32,
}

impl Display<'_> {
    pub fn new(
        i2c0: embassy_rp::peripherals::I2C0,
        sclr: embassy_rp::peripherals::PIN_17,
        sda: embassy_rp::peripherals::PIN_16,
    ) -> Self {
        let i2c = embassy_rp::i2c::I2c::new_blocking(
            i2c0,
            sclr, // SCLR
            sda,  // SDA
            Default::default(),
        );
        let interface = ssd1306::I2CDisplayInterface::new(i2c);
        let mut display = Ssd1306::new(interface, DisplaySize128x32, DisplayRotation::Rotate0)
            .into_buffered_graphics_mode();
        display.init().unwrap();

        let eyes =
            tinygif::Gif::<BinaryColor>::from_slice(include_bytes!("../assets/eyes.gif")).unwrap();

        let frame = 0;

        Self {
            display,
            eyes,
            frame,
        }
    }

    pub fn update(&mut self) {
        let mut iterator = self.eyes.frames();
        let mut image = iterator.next();
        let mut c = 1;
        while c < self.frame && image.is_some() {
            image = iterator.next();
            c = c + 1;
        }
        if image.is_none() {
            self.frame = 1;
            image = self.eyes.frames().next();
        }
        image.unwrap().draw(&mut self.display).unwrap();
        self.display.flush().unwrap();
        self.frame = self.frame + 1;
    }

    pub fn drawing_reference(&mut self) {
        let yoffset = 8;

        let style = PrimitiveStyleBuilder::new()
            .stroke_width(1)
            .stroke_color(BinaryColor::On)
            .build();

        // screen outline
        // default display size is 128x64 if you don't pass a _DisplaySize_
        // enum to the _Builder_ struct
        Rectangle::new(Point::new(0, 0), Size::new(127, 31))
            .into_styled(style)
            .draw(&mut self.display)
            .unwrap();

        // triangle
        Triangle::new(
            Point::new(16, 16 + yoffset),
            Point::new(16 + 16, 16 + yoffset),
            Point::new(16 + 8, yoffset),
        )
        .into_styled(style)
        .draw(&mut self.display)
        .unwrap();
        // square
        Rectangle::new(Point::new(52, yoffset), Size::new_equal(16))
            .into_styled(style)
            .draw(&mut self.display)
            .unwrap();

        // circle
        Circle::new(Point::new(88, yoffset), 16)
            .into_styled(style)
            .draw(&mut self.display)
            .unwrap();

        self.display.flush().unwrap();
    }
}
