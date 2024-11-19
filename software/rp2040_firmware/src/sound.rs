// Resistor 32 ohm

use core::cell::RefCell;
use embassy_rp::interrupt;
use embassy_rp::pwm::{Config, Pwm};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::blocking_mutex::Mutex;
use fixed::FixedU16;
//use portable_atomic::{AtomicU32, Ordering};

const AUDIO_SIZE : usize = 1462987;
const AUDIO: &[u8; AUDIO_SIZE] = include_bytes!("../assets/ode.bin");

//static COUNTER: AtomicU32 = AtomicU32::new(0);
static PWM_AB: Mutex<CriticalSectionRawMutex, RefCell<Option<Pwm>>> =
    Mutex::new(RefCell::new(None));
const BUFFER_SIZE: usize = 48000;
const CONFIG_TOP: u16 = 256;
static mut SOUND_PIPE: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
static mut SOUND_PIPE_READ: usize = 0;
static mut SOUND_PIPE_WRITE: usize = 0;
// target frequency is 48 khz
// PWM 125Mhz.  PWM resoltion is 8 bits, or 256.
// 125*1024*1024/256 (states)/48k
// = 10.6666666
//
const CLOCK_DIVIDER: u16 = 10 * 16 + 11;

pub struct Sound {
    //state: bool,
    //time_to_state_change: u32
    audio_pos: usize
}

impl Sound {
    pub fn new(
        pin_pos: embassy_rp::peripherals::PIN_0,
        pin_neg: embassy_rp::peripherals::PIN_1,
        pwm_slice: embassy_rp::peripherals::PWM_SLICE0,
    ) -> Self {
        let pwm_ab =
            embassy_rp::pwm::Pwm::new_output_ab(pwm_slice, pin_pos, pin_neg, Default::default());
        PWM_AB.lock(|p| p.borrow_mut().replace(pwm_ab));

        let mut config = Config::default();
        config.top = CONFIG_TOP;
        config.compare_b = config.top / 2;
        config.divider = FixedU16::from_bits(CLOCK_DIVIDER);
        PWM_AB.lock(|p| p.borrow_mut().as_mut().unwrap().set_config(&config));

        // Enable the interrupt for pwm slice 0
        embassy_rp::pac::PWM.inte().modify(|w| w.set_ch0(true));
        unsafe {
            cortex_m::peripheral::NVIC::unmask(interrupt::PWM_IRQ_WRAP);
        }

        Self { 
            audio_pos: 0
        }
    }

    pub fn update_one(&mut self) {
        let value: u8 = AUDIO[ self.audio_pos/2 ];
        self.audio_pos = ( self.audio_pos + 1 ) % (AUDIO_SIZE*2);
        unsafe {
            SOUND_PIPE[ SOUND_PIPE_WRITE ] = value;
            SOUND_PIPE_WRITE = ( SOUND_PIPE_WRITE + 1 ) % BUFFER_SIZE;
        }
    }

    pub fn update(&mut self) {
        unsafe {
            let mut next_write: usize = (SOUND_PIPE_WRITE+1) % BUFFER_SIZE;
            while next_write != SOUND_PIPE_READ {
                self.update_one();
                next_write = (SOUND_PIPE_WRITE+1) % BUFFER_SIZE;
            }
        }
    }
}

#[interrupt]
fn PWM_IRQ_WRAP() {
    critical_section::with(|cs| {
        let value: u8;
        unsafe {
            value = if SOUND_PIPE_READ == SOUND_PIPE_WRITE {
                0
            }
            else {
                let rval = SOUND_PIPE[ SOUND_PIPE_READ ];
                SOUND_PIPE_READ = ( SOUND_PIPE_READ + 1 ) % BUFFER_SIZE;
                rval
            }
        }
        let mut config: Config = Config::default();
        config.divider = FixedU16::from_bits(CLOCK_DIVIDER);
        config.top = 256;
        config.compare_a = value as u16;
        config.compare_b = value as u16;
        config.invert_b = true;
        PWM_AB.lock(|p| p.borrow_mut().as_mut().unwrap().set_config(&config));

        PWM_AB
            .borrow(cs)
            .borrow_mut()
            .as_mut()
            .unwrap()
            .clear_wrapped();
    });
}
