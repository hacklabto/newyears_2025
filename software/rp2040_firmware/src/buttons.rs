//! A class to abstract buttons

use embassy_rp::gpio;
use gpio::{Input, Pull};

#[derive(PartialEq, Copy, Clone)]
pub enum Button{
    B0,
    B1,
    B2,
    B3,
}


pub struct Buttons<'a> {
    b0: Input<'a>,
    b1: Input<'a>,
    b2: Input<'a>,
    b3: Input<'a>,
}

impl Buttons<'_> {
    pub fn new(
        pin_2: embassy_rp::peripherals::PIN_2,
        pin_3: embassy_rp::peripherals::PIN_3,
        pin_4: embassy_rp::peripherals::PIN_4,
        pin_5: embassy_rp::peripherals::PIN_5,
    ) -> Self {
        let b0: Input<'_> = Input::new( pin_2, Pull::Up );
        let b1: Input<'_> = Input::new( pin_3, Pull::Up );
        let b2: Input<'_> = Input::new( pin_4, Pull::Up );
        let b3: Input<'_> = Input::new( pin_5, Pull::Up );
        Self{ b0, b1, b2, b3 }
    }

    pub fn is_pressed( &self, button: Button ) -> bool {
        match button {
            Button::B0 => !self.b0.is_high(),
            Button::B1 => !self.b1.is_high(),
            Button::B2 => !self.b2.is_high(),
            Button::B3 => !self.b3.is_high(),
        }
    }
}

