use core::sync::atomic::AtomicU16;
use core::sync::atomic::Ordering;
use embassy_rp::dma::Channel;
use embassy_rp::dma::Transfer;
use embassy_rp::gpio;
use embassy_rp::pio::{
    Common, Direction, FifoJoin, Instance, PioPin, ShiftConfig, ShiftDirection, StateMachine,
};
use embassy_rp::PeripheralRef;
use fixed::traits::ToFixed;
use fixed_macro::types::U56F8;
use gpio::{Level, Output, Pin};
use midi_nostd::midi::Midi;
use midly::Smf;
use pio::InstructionOperands;
type NewYearsMidi = Midi<24000, 16, 8>;

const AUDIO: &[u8] = include_bytes!("../assets/ode.bin");

const TARGET_PLAYBACK: u64 = 72_000;
const PWM_TOP: u64 = 256;
const PWM_CYCLES_PER_READ: u64 = 6 * PWM_TOP + 4;

pub struct SoundDma<const BUFFERS: usize, const BUFSIZE: usize> {
    buffer: [[u32; BUFSIZE]; BUFFERS],
    being_dmaed: AtomicU16,
    next_available_slot: u16,
}

impl<const BUFFERS: usize, const BUFSIZE: usize> SoundDma<BUFFERS, BUFSIZE> {
    pub const fn new() -> Self {
        Self {
            buffer: [[0x80; BUFSIZE]; BUFFERS],
            being_dmaed: AtomicU16::new(0),
            next_available_slot: 2,
        }
    }
    pub const fn num_dma_buffers() -> usize {
        return BUFFERS;
    }

    pub fn next_writable(&mut self) -> Option<&mut [u32]> {
        let buffers_u16 = BUFFERS as u16;
        let buffer_being_dmaed: u16 = self.being_dmaed.load(Ordering::Relaxed);
        let next_available_slot = self.next_available_slot;

        if next_available_slot == buffer_being_dmaed {
            return None;
        }

        self.next_available_slot = (self.next_available_slot + 1) % buffers_u16;
        Some(&mut self.buffer[next_available_slot as usize])
    }

    pub fn get_writable_dma_buffer() -> &'static mut [u32] {
        unsafe { SOUND_DMA.next_writable().unwrap() }
        /*
                loop {
                    unsafe {
                        let writable_maybe = SOUND_DMA.next_writable();
                        if writable_maybe.is_some() {
                            return writable_maybe.unwrap();
                        }
                    }
                    let mut ticker = embassy_time::Ticker::every(embassy_time::Duration::from_millis(25));
                    ticker.next().await;
                }
        */
    }

    pub fn next_to_dma(&mut self) -> &mut [u32] {
        let mut being_dmaed: u16 = self.being_dmaed.load(Ordering::Relaxed);
        being_dmaed = (being_dmaed + 1) % (BUFFERS as u16);
        self.being_dmaed.store(being_dmaed, Ordering::Relaxed);
        &mut self.buffer[being_dmaed as usize]
    }
}

// Buffer size should be a multiple of 3 right now.
//
type SoundDmaType = SoundDma<3, 9600>;
static mut SOUND_DMA: SoundDmaType = SoundDmaType::new();

pub struct AudioPlayback<'d> {
    midi: &'d mut NewYearsMidi,
    smf: &'d Smf<'d>,
    audio_iter: &'d mut dyn Iterator<Item = &'d u8>,
    clear_count: u32,
}

impl<'d> AudioPlayback<'d> {
    pub fn new(
        audio_iter: &'d mut dyn Iterator<Item = &'d u8>,
        midi: &'d mut NewYearsMidi,
        smf: &'d Smf,
    ) -> Self {
        let clear_count: u32 = 0;
        Self {
            midi,
            smf,
            audio_iter,
            clear_count,
        }
    }

    fn populate_next_dma_buffer_with_audio(&mut self, buffer: &mut [u32]) {
        let mut value: u8 = 0;
        let mut read_on_zero: u8 = 0;

        // For now, copy every audio signal at our 24khz playback speed into
        // the buffer 3x.  That gives an effective PWM frequency of 144k
        // (24k * 6), which is as fast as I can get it with with 256 intensity
        // levels.

        for entry in buffer.iter_mut() {
            if read_on_zero == 0 {
                value = (((self.midi.get_next(self.smf).to_i32() >> 8) + 0x80) & 0xff) as u8;
            }
            *entry = value as u32;
            read_on_zero = read_on_zero + 1;
            if read_on_zero == 3 {
                read_on_zero = 0;
            }
            if !self.midi.has_next() {
                self.clear_count = 1;
            }
        }
    }

    fn is_done(&self) -> bool {
        return self.clear_count == 1;
    }

    pub fn populate_next_dma_buffer(&mut self) {
        let dma_write_buffer = SoundDmaType::get_writable_dma_buffer();
        self.populate_next_dma_buffer_with_audio(dma_write_buffer);
    }
}

pub struct PioSound<'d, PIO: Instance, const STATE_MACHINE_IDX: usize, DMA: Channel> {
    state_machine: StateMachine<'d, PIO, STATE_MACHINE_IDX>,
    dma_channel: PeripheralRef<'d, DMA>,
    _ena_pin: Output<'d>,
    _debug_pin: Output<'d>,
}

impl<'d, PIO: Instance, const STATE_MACHINE_IDX: usize, DMA: Channel>
    PioSound<'d, PIO, STATE_MACHINE_IDX, DMA>
{
    pub fn new(
        common: &mut Common<'d, PIO>,
        mut sm: StateMachine<'d, PIO, STATE_MACHINE_IDX>,
        sound_a_pin: impl PioPin,
        sound_b_pin: impl PioPin,
        ena: impl Pin,
        debug: impl Pin,
        dma_channel: DMA,
    ) -> Self {
        #[rustfmt::skip]
        let prg = pio_proc::pio_asm!(
            // From the PIO PWN embassy example, for now
            ".side_set 2 opt"
            "set x, 0"

            "begin:"
                // TSX FIFO -> OSR.  Do not block if the FIFO is empty.
                // If we run out of data, just hold the last PWM state.
                // Set the output to 0
                "pull noblock                   side 0b01"
                "mov x, osr"
                // y is the pwm hardware's equivalent of top
                // loaded using set_top
                "mov y, isr"

            // Loop y times, which is effectively top
            "countloop1:"
                // Switch state to 1 when y matches the pwm value
                "jmp x!=y noset1"
                "jmp skip1                      side 0b10"
            "noset1:"
                // For a consistent 3 cycle delay
                "nop"
            "skip1:"
                "jmp y-- countloop1"

            // Do the loop a 2nd time using loop unrolling
            "mov y, isr                         side 0b01"

            // Loop y times, which is effectively top
            "countloop2:"
                // Switch state to 1 when y matches the pwm value
                "jmp x!=y noset2"
                "jmp skip2                      side 0b10"
            "noset2:"
                // For a consistent 3 cycle delay
                "nop"
            "skip2:"
                "jmp y-- countloop2"

            // Go back for more data.
            "jmp begin"
        );
        let prg = common.load_program(&prg.program);

        let sound_a_pin = common.make_pio_pin(sound_a_pin);
        let sound_b_pin = common.make_pio_pin(sound_b_pin);
        sm.set_pin_dirs(Direction::Out, &[&sound_a_pin, &sound_b_pin]);
        sm.set_pins(Level::Low, &[&sound_a_pin, &sound_b_pin]);

        let mut pio_cfg = embassy_rp::pio::Config::default();
        pio_cfg.use_program(&prg, &[&sound_a_pin, &sound_b_pin]);

        pio_cfg.shift_out = ShiftConfig {
            threshold: 32,
            direction: ShiftDirection::Left,
            auto_fill: false,
        };
        pio_cfg.fifo_join = FifoJoin::TxOnly;

        pio_cfg.clock_divider =
            (U56F8!(125_000_000) / (TARGET_PLAYBACK * PWM_CYCLES_PER_READ)).to_fixed();

        sm.set_config(&pio_cfg);

        let _debug_pin = Output::new(debug, Level::Low);
        let _ena_pin = Output::new(ena, Level::High);

        // errr
        let mut return_value = Self {
            state_machine: sm,
            dma_channel: dma_channel.into_ref(),
            _debug_pin,
            _ena_pin,
        };
        // for the LED test, we'll PWM values from 0-255 with a top of 512.
        return_value.set_top(PWM_TOP as u32);
        return_value.start();
        return_value
    }

    //
    // Set the "top" of the PWM.  The PIO assembly doesn't seem to have
    // a suitable load immediate instruction, so instead we'll put top's
    // value into the ISR
    //
    pub fn set_top(&mut self, top: u32) {
        let is_enabled = self.state_machine.is_enabled();
        while !self.state_machine.tx().empty() {} // Make sure that the queue is empty
        self.state_machine.set_enable(false);
        self.state_machine.tx().push(top);
        unsafe {
            self.state_machine.exec_instr(
                InstructionOperands::PULL {
                    if_empty: false,
                    block: false,
                }
                .encode(),
            );
            self.state_machine.exec_instr(
                InstructionOperands::OUT {
                    destination: ::pio::OutDestination::ISR,
                    bit_count: 32,
                }
                .encode(),
            );
        };
        if is_enabled {
            self.state_machine.set_enable(true) // Enable if previously enabled
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

    pub fn send_dma_buffer_to_pio(&mut self) -> Transfer<'_, DMA> {
        unsafe {
            let dma_buffer = SOUND_DMA.next_to_dma();
            self.state_machine
                .tx()
                .dma_push(self.dma_channel.reborrow(), dma_buffer)
        }
    }

    pub async fn play_sound(&mut self) {
        let smf = midly::Smf::parse(include_bytes!("../assets/twinkle.mid"))
            .expect("It's inlined data, so its expected to parse");
        let mut midi = NewYearsMidi::new(&smf);

        let mut iter = AUDIO.iter();

        let mut playback_state: AudioPlayback = AudioPlayback::new(&mut iter, &mut midi, &smf);

        while !playback_state.is_done() {
            // Start DMA transfer
            let dma_buffer_in_flight = self.send_dma_buffer_to_pio();
            // While the DMA transfer runs, populate the next DMA buffer
            playback_state.populate_next_dma_buffer();
            // Wakes up when "DMA finished transfering" interrupt occurs.
            dma_buffer_in_flight.await;
        }
        self.set_level(0x80);
    }
}
