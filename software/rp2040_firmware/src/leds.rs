use embassy_rp::gpio;
use gpio::{Level, Output, Pin};

const XSIZE: usize = 8;
const YSIZE: usize = 8;

pub struct LEDs<'a> {
    led_state: [bool; XSIZE * YSIZE],

    clock: Output<'a>,
    data: Output<'a>,
    release: Output<'a>,
    state: u32,
}

impl LEDs<'_> {
    pub fn new(clock_pin: impl Pin, data_pin: impl Pin, release_pin: impl Pin) -> Self {
        let led_state: [bool; XSIZE * YSIZE] = [false; XSIZE * YSIZE];

        let clock: Output<'_> = Output::new(clock_pin, Level::Low);
        let data: Output<'_> = Output::new(data_pin, Level::Low);
        let release: Output<'_> = Output::new(release_pin, Level::Low);

        Self {
            clock,
            data,
            release,
            state: 0,
            led_state,
        }
    }

    pub fn set(&mut self, x: usize, y: usize, state: bool) {
        self.led_state[y * XSIZE + x] = state;
    }

    pub fn update(&mut self) {
        let cycle = self.state % 4;
        let pixel = (self.state / 4) % ((XSIZE * YSIZE) as u32);
        let led = self.led_state[pixel as usize];

        match cycle {
            0 => {
                self.clock.set_low();
                self.release.set_low();
            }
            1 => {
                if led {
                    self.data.set_high();
                } else {
                    self.data.set_low();
                }
            }
            2 => {
                self.clock.set_high();
            }
            3..=u32::MAX => {
                self.release.set_high();
            }
        }

        self.state = self.state + 1;
    }
}
