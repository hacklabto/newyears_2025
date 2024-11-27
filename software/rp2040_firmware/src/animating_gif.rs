//! Bind preloaed animating gifs emums so Rust doesn't taz library users
//! Expose a simple way to play the animating gifs

use crate::display::DisplaySSD;
use embassy_rp::i2c::Instance;
use tinygif;

use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
};

/*
 TODO
#[derive(PartialEq, Copy, Clone)]
pub enum DisplayArt {
    Eyes,
}
*/

pub struct AnimatingGif<'a> {
    eyes: tinygif::Gif<'a, BinaryColor>,
    frame: u32,
}

impl AnimatingGif<'_> {
    pub fn new(
    ) -> Self {
        let frame = 0;
        let eyes =
            tinygif::Gif::<BinaryColor>::from_slice(include_bytes!("../assets/eyes.gif")).unwrap();

        Self {
            eyes,
            frame,
        }
    }

    pub fn update<I2C: Instance>(&mut self, display: &mut DisplaySSD<'_, I2C>) {
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
        image.unwrap().draw(display).unwrap();
        self.frame = self.frame + 1;
    }
}
