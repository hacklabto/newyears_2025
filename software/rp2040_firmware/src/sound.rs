// Resistor 32 ohm

use core::cell::{RefCell};
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_rp::pwm::{Config, Pwm};
use embassy_rp::interrupt;
use portable_atomic::{AtomicU32, Ordering};
use fixed::{FixedU16};

static COUNTER: AtomicU32 = AtomicU32::new(0);
static PWM_AB: Mutex<CriticalSectionRawMutex, RefCell<Option<Pwm>>> = Mutex::new(RefCell::new(None));
const BUFFER_SIZE: usize =256;
const CONFIG_TOP: u16 = 256;
static mut BUFFER: [u8; BUFFER_SIZE ] = [0; BUFFER_SIZE ];
static mut BUFFER_POS: usize = 0;
// target frequency is 48 khz
// PWM 125Mhz.  PWM resoltion is 8 bits, or 256.
// 125*1024*1024/256 (states)/48k
// = 10.6666666
//
const CLOCK_DIVIDER: u16 = 10*16+11;

pub struct Sound {
}

impl Sound {
    pub fn new(
        pin_pos: embassy_rp::peripherals::PIN_0,
        pin_neg: embassy_rp::peripherals::PIN_1,
        pwm_slice: embassy_rp::peripherals::PWM_SLICE0 
    ) -> Self {

        for c in 0..BUFFER_SIZE {
            let a: f32 = ( c as f32 ) / (BUFFER_SIZE as f32 ) * 2.0*3.14159265358979323846264338327950288_f32;
            let s = micromath::F32Ext::sin( a );
            let _ints = (s * 127.0 + 128.0 ) as u8;

            unsafe {
                //BUFFER[c as usize] = ints;
                BUFFER[ c as usize ] = if c < BUFFER_SIZE/2 { 255 } else {0}
            }
        }

        let pwm_ab = embassy_rp::pwm::Pwm::new_output_ab(pwm_slice, pin_pos, pin_neg, Default::default());
        PWM_AB.lock(|p| p.borrow_mut().replace(pwm_ab));

        let mut config = Config::default();
        config.top = CONFIG_TOP;
        config.compare_b = config.top/2;
        config.divider= FixedU16::from_bits( CLOCK_DIVIDER );
        PWM_AB.lock(|p| p.borrow_mut().as_mut().unwrap().set_config(&config));

        // Enable the interrupt for pwm slice 0
        embassy_rp::pac::PWM.inte().modify(|w| w.set_ch0(true));
        unsafe {
            cortex_m::peripheral::NVIC::unmask(interrupt::PWM_IRQ_WRAP);
        }

        Self {}
    }

    pub fn update(&mut self)
    {
    }
}

#[interrupt]
fn PWM_IRQ_WRAP() {
    critical_section::with(|cs| {
        if (COUNTER.load(Ordering::Relaxed) % 1) == 0 {
            let value:u8;
            unsafe {
                BUFFER_POS = ( BUFFER_POS + 2 ) % BUFFER_SIZE;
                value = BUFFER[ BUFFER_POS ];
            }
            let mut config: Config = Config::default();
            config.divider= FixedU16::from_bits( CLOCK_DIVIDER );
            config.top = 256;
            config.compare_a = value as u16;
            config.compare_b = value as u16;
            config.invert_b = true;
            PWM_AB.lock(|p| p.borrow_mut().as_mut().unwrap().set_config(&config));
        }

        PWM_AB.borrow(cs).borrow_mut().as_mut().unwrap().clear_wrapped();
    });
    COUNTER.fetch_add(1, Ordering::Relaxed);
}

