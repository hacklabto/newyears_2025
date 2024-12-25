use crate::devices::Devices;
use crate::Button;
use core::marker::PhantomData;
use core::sync::atomic::AtomicU16;
use core::sync::atomic::AtomicU32;
use core::sync::atomic::Ordering;
use embassy_rp::interrupt;
use embassy_rp::pwm::{ChannelAPin, ChannelBPin, Config, Pwm, Slice};
use embassy_rp::Peripheral;
use fixed::FixedU16;

const AUDIO: &[u8] = include_bytes!("../assets/ode.bin");

pub struct SoundDma<const BUFFERS: usize, const BUFSIZE: usize> {
    buffer: [[u8; BUFSIZE]; BUFFERS],
    being_dmaed: AtomicU16,
    fakey_fakey_dma_pos: AtomicU32,
    next_available_slot: u16
}

impl<const BUFFERS: usize, const BUFSIZE: usize> SoundDma<BUFFERS, BUFSIZE> {
    pub const fn new() -> Self {
        Self {
            buffer: [[0x0; BUFSIZE]; BUFFERS],
            being_dmaed: AtomicU16::new(0),
            fakey_fakey_dma_pos: AtomicU32::new(0),
            next_available_slot: 2,
        }
    }
    pub fn next_writable(&mut self) -> Option<&mut [u8]> {
        let buffer_being_dmaed: u16 = self.being_dmaed.load(Ordering::Relaxed);

        if self.next_available_slot == buffer_being_dmaed {
            return None
        }
        self.next_available_slot = self.next_available_slot + 1;
        Some(&mut self.buffer[ self.next_available_slot as usize ])
    }

    pub fn next_to_go_to_sound(&mut self) -> u8 {
        let mut dma_buffer_u16: u16 = self.being_dmaed.load(Ordering::Relaxed);
        let dma_buffer: usize = dma_buffer_u16 as usize;
        let mut fakey_fakey_dma_pos_u32: u32 = self.fakey_fakey_dma_pos.load(Ordering::Relaxed);
        let dma_pos: usize = fakey_fakey_dma_pos_u32 as usize;
        let value = self.buffer[dma_buffer][dma_pos];

        fakey_fakey_dma_pos_u32 = fakey_fakey_dma_pos_u32 + 1;
        if fakey_fakey_dma_pos_u32 == BUFSIZE as u32 {
            fakey_fakey_dma_pos_u32 = 0;
            dma_buffer_u16 = (dma_buffer_u16 + 1) % (BUFFERS as u16);
        }

        self.fakey_fakey_dma_pos
            .store(fakey_fakey_dma_pos_u32, Ordering::Relaxed);
        self.being_dmaed.store(dma_buffer_u16, Ordering::Relaxed);

        value
    }
}

static mut SOUND_DMA: SoundDma<3, 16384> = SoundDma::new();

const CONFIG_TOP: u16 = 512;
//static mut SOUND_PIPE: Option<Pipe<u8, 48000>> = None;
static mut PWM_AB: Option<Pwm> = None;
static mut PWM_CONFIG: Option<Config> = None;
// target frequency is 48 khz
// PWM 125Mhz.  PWM resoltion is 8 bits, or 256.
// 125*1024*1024/256 (states)/48k
// = 10.6666666
//
const FRACTION_BITS_IN_CLOCK_DIVIDER: u16 = 16;
const RPI_FREQUENCY: u128 = 133000000;
//const TARGET_FREQUENCY : u128 = 48000;  bad squee sound
//const TARGET_FREQUENCY : u128 = 51853;  bad squee sound
//const TARGET_FREQUENCY : u128 = 43294;  bad squee sound
//const TARGET_FREQUENCY : u128 = 47500;  //bad squee sound
const TARGET_FREQUENCY: u128 = 48800; // cound sound, but why?
const CLOCK_DIVIDER: u128 = (FRACTION_BITS_IN_CLOCK_DIVIDER as u128) * RPI_FREQUENCY
    / (CONFIG_TOP as u128)
    / TARGET_FREQUENCY;
const CLOCK_DIVIDER_U16: u16 = CLOCK_DIVIDER as u16;

pub struct Sound<PwmSlice: Slice> {
    // Add a dummy member so the struct can be tied to the PWM
    // interface being used
    pwm_device: PhantomData<PwmSlice>,
}

impl<PwmSlice: Slice> Sound<PwmSlice> {
    // These are hardware interfaces, so they will live for the entire program (+ 'static)
    pub fn new(
        pin_pos: impl Peripheral<P = impl ChannelAPin<PwmSlice>> + 'static,
        pin_neg: impl Peripheral<P = impl ChannelBPin<PwmSlice>> + 'static,
        pwm_slice: impl Peripheral<P = PwmSlice> + 'static,
    ) -> Self {
        let pwm_ab =
            embassy_rp::pwm::Pwm::new_output_ab(pwm_slice, pin_pos, pin_neg, Default::default());

        unsafe {
            PWM_AB = Some(pwm_ab);
            PWM_CONFIG = Some(Config::default());
            let config = PWM_CONFIG.as_mut().unwrap();
            config.top = CONFIG_TOP;
            config.compare_b = CONFIG_TOP / 2;
            config.divider = FixedU16::from_bits(CLOCK_DIVIDER_U16);
            config.invert_b = true;
            PWM_AB.as_mut().unwrap().set_config(config);
        }

        // Enable the interrupt for pwm slice 0
        embassy_rp::pac::PWM.inte().modify(|w| w.set_ch0(true));
        unsafe {
            cortex_m::peripheral::NVIC::unmask(interrupt::PWM_IRQ_WRAP);
        }

        Self {
            pwm_device: PhantomData,
        }
    }

    pub async fn play_sound(&self, devices: &Devices<'_>) {
        let mut iter = AUDIO.iter();

        loop {
            unsafe {
                let writable_maybe = SOUND_DMA.next_writable();
                let mut done=false;

                if writable_maybe.is_some() {
                    let writable = writable_maybe.unwrap();
                    for entry in writable.iter_mut() {
                        let next_audio = iter.next();
                        if next_audio.is_some() {
                            *entry = *next_audio.unwrap();
                        }
                        else {
                            *entry = 0x80;
                            done = true;
                        }
                    }
                }
                if devices.buttons.is_pressed(Button::B0) {
                    done = true;    // escape
                }
                if done {
                    break;
                }
            }
        }
    }
}

#[interrupt]
fn PWM_IRQ_WRAP() {
    unsafe {
        let value = SOUND_DMA.next_to_go_to_sound();
        let config = PWM_CONFIG.as_mut().unwrap();
        config.compare_a = value as u16;
        config.compare_b = value as u16;

        let pwm = PWM_AB.as_mut().unwrap();
        pwm.set_config(&config);
        pwm.clear_wrapped();
    }
}
