//
// High level sound playback documentation
// =======================================
//
// Sound playback in hacker new year is done using reasonably high frequency
// pulse width modulation. The number of pulses / second is given by
// 125,000,000 mhz (the IO frequency) /
// 64 (the number of levels of each output pulse) /
// 3  (the approximate numer of IO clock cycles per pulse pulse level)
//
// or about 651000 pulses per second.  The hope is that, by outputting the pulses this fast,
// any weird harmonic problems from trying to trying to use a digial output - two IO pins that
// can only bet set on and off, will be high frequency enough that they'll be out of the range
// humans can hear.
//
// As just mentioned, the output is two IO pins.  The three legal configurations supported
// are...
//
// 0  0     Speaker is off, no sound
// 1  0     Speaker wire 1 has voltage, Speaker wire 2 is grounded
// 0  1     Speaker wire 2 has voltage, Speaker wire 2 is grounded
//
// A nice thing about this configuration is that we won't needed to genrate pulses to output
// silence, which is when imperfections in the scheme are most noticable.
//
// Why not output pulses faster, or slower?  Right now, the answer is that this seems to
// work in testing.
//
// Problems with outputting pulses faster
// ======================================
//
// The current implementation has one of the cores prepare a DMA buffer that goes to the
// RPi Pico's PIO hardware and is then played back.  Faster playback increases the chance
// that the PIO will run out of bytes before the next DMA buffer can be lauched.  This
// could be fixed of embassy supported ping pong style DMA output.  The actual hardware
// totally supports it, but the embassy framework is stll a work in progress.
//
// The second problem is that the DMA buffer has to be prepared on the core. Outputting
// pulses faster puts more load on the core, which is already struggling to do the midi
// playback.
//
// Problems with output pulses slower
// ==================================
//
// It just doesn't sound as good.  When PWM is used on slower hardware, like a stepper motor
// or a selonoid, you take advantage of the fact that the hardware can't physically move that
// fast, but speakers are designed to move fast.
//
// Should we just put a low pass filter into the hardware?  Hey, I'm a programmer, not a
// hardware designer.
//
// Sound sample Dithering
// ======================
//
// The midi playback rate is defined in audio_playback.rs and is currenty 20292hz
// which means we generate 65100/20292, or 32 pulses per audio sample played back
//

use crate::audio_playback::AudioPlayback;
use embassy_rp::dma;
use embassy_rp::dma::Transfer;
use embassy_rp::gpio;
use embassy_rp::peripherals::PIO0;
use embassy_rp::pio::program::pio_asm;
use embassy_rp::pio::{Direction, FifoJoin, PioPin, ShiftConfig, ShiftDirection, StateMachine, Common};
use embassy_rp::Peri;
use embassy_rp::interrupt;
use fixed::traits::ToFixed;
use gpio::{Level, Output, Pin};
use midi_nostd::midi::Midi;

// 89 and 3 are factors of 20292.  89*3 has to be a factor of 20292.
type NewYearsMidi<'a> = Midi<'a, 20292, { 89 * 3 }, 64, 32>;

// Right noiw the playback time for each buffer is 16384/20292/16 seconds, ~= .05s
//
const DMA_BUFSIZE: usize = 8192;

#[allow(clippy::declare_interior_mutable_const)]
static mut DMA_BUFFER_0: [u32; DMA_BUFSIZE] = [0x00; DMA_BUFSIZE];

#[allow(clippy::declare_interior_mutable_const)]
static mut DMA_BUFFER_1: [u32; DMA_BUFSIZE] = [0x00; DMA_BUFSIZE];

pub struct PioSound<'d> {
    state_machine: StateMachine<'d, PIO0, 0>,
    dma: dma::Channel<'d>,
    _debug_pin: Output<'d>,
}

impl<'d> PioSound<'d> {
    pub fn new<D: dma::ChannelInstance> (
        common: &mut Common<'d, PIO0>,
        mut sm: StateMachine<'d, PIO0, 0>,
        dma: Peri<'d, D>,
        irq: impl interrupt::typelevel::Binding<D::Interrupt, dma::InterruptHandler<D>> + 'd,
        sound_data_pin: Peri<'d, impl PioPin>,
        sound_bclk_pin: Peri<'d, impl PioPin>,
        sound_lrclk_pin: Peri<'d, impl PioPin>,
        debug: Peri<'d, impl Pin>,
    ) -> Self {

        #[rustfmt::skip]
        let prg = pio_asm!(
            // All BCLK toggles should be 2 cycles to guarantee we output
            // a square wave...
            ".side_set 2 opt"
            ".wrap_target"
            "start_sample_left:"
                "set pins, 0                    side 0b00 [2]"
                "set y, 15                      side 0b01 [2]"
            "fillrow_bit_left:"
                // Left channel is unused.
                "set pins,0                     side 0b00 [2]"
                "jmp y--, fillrow_bit_left      side 0b01 [2]"

            "start_sample_right:"
                // Repeat logic for right side
                "set pins, 0                    side 0b10 [2]"
                "set y, 15                      side 0b11 [2]"
            "fillrow_bit_right:"
                "out pins,1                     side 0b10 [2]"
                "jmp y--, fillrow_bit_right     side 0b11 [2]"
            ".wrap"
        );
        let prg = common.load_program(&prg.program);

        let sound_data_pin = common.make_pio_pin(sound_data_pin);
        let sound_bclk_pin = common.make_pio_pin(sound_bclk_pin);
        let sound_lrclk_pin = common.make_pio_pin(sound_lrclk_pin);

        let pio0_cfg = {
            let mut cfg = embassy_rp::pio::Config::default();
            cfg.use_program(&prg, &[&sound_bclk_pin, &sound_lrclk_pin]);
            cfg.set_out_pins(&[&sound_data_pin]);
            // Note 100% right, but close...
            cfg.clock_divider = 30.to_fixed();
            cfg.shift_out = ShiftConfig {
                auto_fill: true,
                threshold: 32,
                direction: ShiftDirection::Left,
            };
            cfg.fifo_join = FifoJoin::TxOnly;
            cfg
        };
        sm.set_config(&pio0_cfg);
        sm.set_pin_dirs(
            Direction::Out,
            &[&sound_data_pin, &sound_bclk_pin, &sound_lrclk_pin],
        );
        sm.set_enable(true);

        let _debug_pin = Output::new(debug, Level::Low);

        Self {
            state_machine: sm,
            dma: dma::Channel::new(dma, irq),
            _debug_pin,
        }
    }

    pub fn start(&mut self) {
        self.state_machine.set_enable(true);
    }

    pub fn stop(&mut self) {
        self.state_machine.set_enable(false);
    }

    pub fn set_level(&mut self, level: u8) {
        let level_u32 = level as u32;
        //let value_to_send = level_u32 | (level_u32 << 8) | (level_u32 << 16) | (level_u32 << 24);
        while !self.state_machine.tx().try_push(level_u32) {}
    }

    pub async fn fill_dma_buffer() {}

    pub fn send_dma_buffer_to_pio(&mut self, buffer_num: u32) -> Transfer<'_> {
        let dma_buffer = Self::get_writable_dma_buffer(buffer_num);
        self.state_machine
            .tx()
            .dma_push(&mut self.dma, dma_buffer, true)
    }

    #[allow(static_mut_refs)]
    pub fn get_writable_dma_buffer(buffer_num: u32) -> &'static mut [u32] {
        unsafe {
            if buffer_num == 0 {
                &mut DMA_BUFFER_0
            } else {
                &mut DMA_BUFFER_1
            }
        }
    }

    pub async fn play_sound(&mut self) {
        //let (header, tracks) = midly::parse(include_bytes!("../assets/maple.mid"))
        //let (header, tracks) = midly::parse(include_bytes!("../assets/vivaldi.mid"))
        let (header, tracks) = midly::parse(include_bytes!("../assets/entertainer.mid"))
            .expect("It's inlined data, so its expected to parse");
        let mut midi = NewYearsMidi::new(&header, tracks);

        let mut playback_state = AudioPlayback::new(&mut midi);
        let mut buffer_sending: u32 = 0;

        while !playback_state.is_done() {
            buffer_sending = 1 - buffer_sending;
            // Start DMA transfer
            let dma_buffer_in_flight = self.send_dma_buffer_to_pio(buffer_sending);
            // While the DMA transfer runs, populate the next DMA buffer
            let dma_write_buffer = Self::get_writable_dma_buffer(1 - buffer_sending);
            playback_state.populate_next_dma_buffer_with_audio(dma_write_buffer);
            //playback_state.populate_next_dma_buffer();
            // Wakes up when "DMA finished transfering" interrupt occurs.
            dma_buffer_in_flight.await;
        }
        self.set_level(0x80);
    }
}
