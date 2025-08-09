//! A class to abstract buttons

use embassy_rp::gpio;
use gpio::Input;

#[derive(PartialEq, Copy, Clone)]
pub enum Button {
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

impl<'a> Buttons<'a> {
    pub fn new(b0: Input<'a>, b1: Input<'a>, b2: Input<'a>, b3: Input<'a>) -> Self {
        Self { b0, b1, b2, b3 }
    }

    pub fn is_pressed(&self, button: Button) -> bool {
        match button {
            Button::B0 => !self.b0.is_high(),
            Button::B1 => !self.b1.is_high(),
            Button::B2 => !self.b2.is_high(),
            Button::B3 => !self.b3.is_high(),
        }
    }

    fn index_to_button(&self, index: usize) -> Button {
        match index {
            0 => Button::B0,
            1 => Button::B1,
            2 => Button::B2,
            3 => Button::B3,
            4_usize.. => todo!(),
        }
    }

    pub fn all_buttons(&self) -> [bool; 4] {
        return [
            self.is_pressed(Button::B0),
            self.is_pressed(Button::B1),
            self.is_pressed(Button::B2),
            self.is_pressed(Button::B3),
        ];
    }

    pub async fn wait_for_press(&self) -> Button {
        let mut ticker = embassy_time::Ticker::every(embassy_time::Duration::from_millis(100));
        let mut last_state = self.all_buttons();
        loop {
            let new_state = self.all_buttons();
            for idx in 0..=3 {
                if !last_state[idx] && new_state[idx] {
                    return self.index_to_button(idx);
                }
            }
            last_state = new_state;
            ticker.next().await;
        }
    }
}
