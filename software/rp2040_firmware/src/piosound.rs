use core::sync::atomic::AtomicU16;
use core::sync::atomic::Ordering;
use embassy_futures::join::join;
use embassy_rp::dma::Channel;
use embassy_rp::gpio::Level;
use embassy_rp::pio::{
    Common, Direction, Instance, PioPin, ShiftConfig, ShiftDirection, StateMachine,
};
use embassy_rp::PeripheralRef;
use fixed::traits::ToFixed;
use fixed_macro::types::U56F8;
use pio::InstructionOperands;

const AUDIO: &[u8] = include_bytes!("../assets/ode.bin");

const TARGET_PLAYBACK: u64 = 24_000;
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

    pub async fn get_dma_buffer() -> &'static mut [u32] {
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
    }

    pub fn next_to_dma(&mut self) -> &mut [u32] {
        let mut being_dmaed: u16 = self.being_dmaed.load(Ordering::Relaxed);
        being_dmaed = (being_dmaed + 1) % (BUFFERS as u16);
        self.being_dmaed.store(being_dmaed, Ordering::Relaxed);
        &mut self.buffer[being_dmaed as usize]
    }
}

type SoundDmaType = SoundDma<3, 12000>;
static mut SOUND_DMA: SoundDmaType = SoundDmaType::new();

pub struct AudioPlayback<'d> {
    audio_iter: &'d mut dyn Iterator<Item = &'d u8>,
    clear_count: u32,
}

impl<'d> AudioPlayback<'d> {
    pub fn new(audio_iter: &'d mut dyn Iterator<Item = &'d u8>) -> Self {
        let clear_count: u32 = 0;
        Self {
            audio_iter,
            clear_count,
        }
    }

    fn populate_dma_buffer_with_audio(&mut self, buffer: &mut [u32]) {
        for entry in buffer.iter_mut() {
            let value_maybe = self.audio_iter.next();
            let value: u8 = if value_maybe.is_some() {
                *value_maybe.unwrap()
            } else {
                self.clear_count = 1;
                0x80
            };
            *entry = value as u32;
        }
    }

    fn is_done(&self) -> bool {
        return self.clear_count == 1;
    }

    pub async fn populate_dma_buffer(&mut self) {
        let dma_write_buffer = SoundDmaType::get_dma_buffer().await;
        self.populate_dma_buffer_with_audio(dma_write_buffer);
    }
}

pub struct PioSound<'d, PIO: Instance, const STATE_MACHINE_IDX: usize, DMA: Channel> {
    state_machine: StateMachine<'d, PIO, STATE_MACHINE_IDX>,
    dma_channel: PeripheralRef<'d, DMA>,
}

impl<'d, PIO: Instance, const STATE_MACHINE_IDX: usize, DMA: Channel>
    PioSound<'d, PIO, STATE_MACHINE_IDX, DMA>
{
    pub fn new(
        common: &mut Common<'d, PIO>,
        mut sm: StateMachine<'d, PIO, STATE_MACHINE_IDX>,
        sound_a_pin: impl PioPin,
        _sound_b_pin: impl PioPin, // abandonned for now
        dma_channel: DMA,
    ) -> Self {
        let prg = pio_proc::pio_asm!(
            // From the PIO PWN embassy example, for now
             ".side_set 1 opt"
                // TSX FIFO -> OSR.  Do not block if the FIFO is empty.
                // If we run out of data, just hold the last PWM state.
                // Set the output to 0
                "pull noblock side 0"
                "mov x, osr"
                // y is the pwm hardware's equivalent of top
                // loaded using set_top
                "mov y, isr"

            // Loop y times, which is effectively top
            "countloop1:"
                // Switch state to 1 when y matches the pwm value
                "jmp x!=y noset1"
                "jmp skip1        side 1"
            "noset1:"
                // For a consistent 3 cycle delay
                "nop"
            "skip1:"
                "jmp y-- countloop1"

                // Do the loop a 2nd time using loop unrolling
                "mov y, isr  side 0"

            // Loop y times, which is effectively top
            "countloop2:"
                // Switch state to 1 when y matches the pwm value
                "jmp x!=y noset2"
                "jmp skip2        side 1"
            "noset2:"
                // For a consistent 3 cycle delay
                "nop"
            "skip2:"
                "jmp y-- countloop2"
        );
        let prg = common.load_program(&prg.program);

        let sound_a_pin = common.make_pio_pin(sound_a_pin);
        sm.set_pins(Level::High, &[&sound_a_pin]);
        sm.set_pin_dirs(Direction::Out, &[&sound_a_pin]);

        let mut pio_cfg = embassy_rp::pio::Config::default();
        pio_cfg.use_program(&prg, &[&sound_a_pin]);

        pio_cfg.shift_out = ShiftConfig {
            auto_fill: false,
            threshold: 32,
            direction: ShiftDirection::Left,
        };
        pio_cfg.clock_divider =
            (U56F8!(125_000_000) / (TARGET_PLAYBACK * PWM_CYCLES_PER_READ)).to_fixed();

        sm.set_config(&pio_cfg);

        // errr
        let mut return_value = Self {
            state_machine: sm,
            dma_channel: dma_channel.into_ref(),
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

    pub async fn drain_dma_buffer(&mut self) {
        unsafe {
            let dma_buffer = SOUND_DMA.next_to_dma();
            self.state_machine
                .tx()
                .dma_push(self.dma_channel.reborrow(), dma_buffer)
                .await;
        }
    }

    pub async fn play_sound(&mut self) {
        let mut iter = AUDIO.iter();
        let mut playback_state: AudioPlayback = AudioPlayback::new(&mut iter);

        while !playback_state.is_done() {
            join(
                playback_state.populate_dma_buffer(),
                self.drain_dma_buffer(),
            )
            .await;
        }
        self.set_level(0x20);

        /*
                // Send the DMA packet 3 times.
                for _i in 0..3 {
                }

                // Reset to lower intensity to show the PIO will continue
                // to run a default PWM program if it doesn't have data.
        */
    }
}
