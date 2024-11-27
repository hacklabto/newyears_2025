//! A class to abstract buttons

use embassy_rp::gpio;
use gpio::{Input, Pin, Pull};

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
    pub fn new(b0_pin: impl Pin, b1_pin: impl Pin, b2_pin: impl Pin, b3_pin: impl Pin) -> Self {
        let b0: Input<'_> = Input::new(b0_pin, Pull::Up);
        let b1: Input<'_> = Input::new(b1_pin, Pull::Up);
        let b2: Input<'_> = Input::new(b2_pin, Pull::Up);
        let b3: Input<'_> = Input::new(b3_pin, Pull::Up);
        Self { b0, b1, b2, b3 }
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
