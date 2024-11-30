//! Bind preloaed animating gifs emums so Rust doesn't taze library users
//! Expose a simple way to play the animating gifs

use crate::devices::Devices;
//use crate::Timer;
use crate::Button;
use embassy_time::Instant;

use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
};

#[derive(PartialEq, Copy, Clone)]
pub enum AnimatingGif {
    Eyes,
    Abstract,
    Logo,
}

pub struct AnimatingGifs<'a> {
    eyes: tinygif::Gif<'a, BinaryColor>,
    abstract_animation: tinygif::Gif<'a, BinaryColor>,
    logo: tinygif::Gif<'a, BinaryColor>,
}

impl AnimatingGifs<'_> {
    pub fn new(
    ) -> Self {
        let eyes =
            tinygif::Gif::<BinaryColor>::from_slice(include_bytes!("../assets/eyes.gif")).unwrap();
        let abstract_animation =
            tinygif::Gif::<BinaryColor>::from_slice(include_bytes!("../assets/abstract.gif")).unwrap();
        let logo =
            tinygif::Gif::<BinaryColor>::from_slice(include_bytes!("../assets/logo.gif")).unwrap();

        Self {
            eyes,
            abstract_animation,
            logo,
        }
    }

    pub async fn animate(&self, animating_gif: AnimatingGif, devices: &mut Devices<'_> ) {
        let start_time = Instant::now();
        let mut animation_time: i32 = 0;

        let gif = match animating_gif {
            AnimatingGif::Eyes => &self.eyes,
            AnimatingGif::Abstract => &self.abstract_animation,
            AnimatingGif::Logo=> &self.logo,
        };
        devices.display.clear(BinaryColor::Off).unwrap();

        for frame in gif.frames() {
            if devices.buttons.is_pressed( Button::B0 ) {
                break;  // "escape"
            }
            let ms_since_start = start_time.elapsed().as_millis() as i32;
            let time_to_frame = animation_time - ms_since_start;
            if time_to_frame >= 0 {
                let mut ticker = embassy_time::Ticker::every(embassy_time::Duration::from_millis(time_to_frame.try_into().unwrap()));
                ticker.next().await;
                frame.draw(&mut devices.display).unwrap();
                devices.display.flush().unwrap();
            }
            animation_time += (frame.delay_centis as i32) * 10;
        }
    }
}
