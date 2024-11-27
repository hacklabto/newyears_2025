//! Friendly wrapper (DisplaySSD) for the OLED display class and initializer

use embassy_rp::i2c;
use embassy_rp::i2c::{Instance, SclPin, SdaPin};
use embassy_rp::Peripheral;
use ssd1306::mode::BufferedGraphicsMode;
use ssd1306::mode::DisplayConfig;
use ssd1306::prelude::I2CInterface;
use ssd1306::rotation::DisplayRotation;
use ssd1306::size::DisplaySize128x32;
use ssd1306::Ssd1306;

/// Turn the actual display class into something readable.
pub type DisplaySSD<'a, I2C> = Ssd1306<
    I2CInterface<i2c::I2c<'a, I2C, i2c::Blocking>>,
    DisplaySize128x32,
    BufferedGraphicsMode<DisplaySize128x32>,
>;

/// Create a display
///
/// i2c0, sclr, and sda are the I2C interface and I2C Pins
/// Takes ownership of interface and pins from the caller.
///
pub fn create_ssd_display<'a, I2C: Instance>(
    i2c_interface: impl Peripheral<P = I2C> + 'a,
    sclr: impl Peripheral<P = impl SclPin<I2C>> + 'a,
    sda: impl Peripheral<P = impl SdaPin<I2C>> + 'a,
) -> DisplaySSD<'a, I2C> {
    let i2c = embassy_rp::i2c::I2c::new_blocking(
        i2c_interface,
        sclr, // SCLR
        sda,  // SDA
        Default::default(),
    );
    let interface = ssd1306::I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306::new(interface, DisplaySize128x32, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    display.init().unwrap();
    display
}

// Some sample code for drawing on the display
// TODO - this could just be a link to the github example I used.

    /*
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
    */
