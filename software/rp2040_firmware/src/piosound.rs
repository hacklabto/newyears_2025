use crate::audio_playback::AudioPlayback;
use embassy_rp::dma::Channel;
use embassy_rp::dma::Transfer;
use embassy_rp::gpio;
use embassy_rp::pio::{
    Common, Direction, FifoJoin, Instance, PioPin, ShiftConfig, ShiftDirection, StateMachine,
};
use embassy_rp::PeripheralRef;
use fixed::traits::ToFixed;
use gpio::{Level, Output, Pin};
use midi_nostd::midi::Midi;
use pio::InstructionOperands;
// 89 and 3 are factors of 20292.  89*3 has to be a factor of 20292.
type NewYearsMidi<'a> = Midi<'a, 20292, { 89 * 3 }, 64, 32>;

const PWM_BITS: u32 = 6;
const REMAINDER_BITS: u32 = 10 - PWM_BITS;
const DMA_BUFSIZE: usize = 16384;

#[allow(clippy::declare_interior_mutable_const)]
static mut DMA_BUFFER_0: [u8; DMA_BUFSIZE] = [0x80; DMA_BUFSIZE];

#[allow(clippy::declare_interior_mutable_const)]
static mut DMA_BUFFER_1: [u8; DMA_BUFSIZE] = [0x80; DMA_BUFSIZE];

pub struct PioSound<'d, PIO: Instance, const STATE_MACHINE_IDX: usize, Dma0: Channel, Dma1: Channel>
{
    state_machine: StateMachine<'d, PIO, STATE_MACHINE_IDX>,
    dma_channel_0: PeripheralRef<'d, Dma0>,
    dma_channel_1: PeripheralRef<'d, Dma1>,
    _ena_pin: Output<'d>,
    _debug_pin: Output<'d>,
}

impl<'d, PIO: Instance, const STATE_MACHINE_IDX: usize, Dma0: Channel, Dma1: Channel>
    PioSound<'d, PIO, STATE_MACHINE_IDX, Dma0, Dma1>
{
    pub fn new(
        common: &mut Common<'d, PIO>,
        mut sm: StateMachine<'d, PIO, STATE_MACHINE_IDX>,
        sound_a_pin: impl PioPin,
        sound_b_pin: impl PioPin,
        ena: impl Pin,
        debug: impl Pin,
        dma_channel_0: Dma0,
        dma_channel_1: Dma1,
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
                //"out x,8                   side 0b01"
                //"out x,8                   side 0b01"
                //"out x,8                   side 0b01"
                //"out x,8                   side 0b01"
                "pull                     side 0b01"
                "out x,8"
                //"mov x, osr"
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
            auto_fill: true,
        };
        pio_cfg.fifo_join = FifoJoin::TxOnly;

        pio_cfg.clock_divider = 1.to_fixed();

        sm.set_config(&pio_cfg);
        const PWM_TOP: u32 = 1 << PWM_BITS;
        Self::set_top(&mut sm, PWM_TOP as u32);
        sm.set_enable(true);

        let _debug_pin = Output::new(debug, Level::Low);
        let _ena_pin = Output::new(ena, Level::High);

        Self {
            state_machine: sm,
            dma_channel_0: dma_channel_0.into_ref(),
            dma_channel_1: dma_channel_1.into_ref(),
            _debug_pin,
            _ena_pin,
        }
    }

    //
    // Set the "top" of the PWM.  The PIO assembly doesn't seem to have
    // a suitable load immediate instruction, so instead we'll put top's
    // value into the ISR
    //
    pub fn set_top(state_machine: &mut StateMachine<'d, PIO, STATE_MACHINE_IDX>, top: u32) {
        let is_enabled = state_machine.is_enabled();
        while !state_machine.tx().empty() {} // Make sure that the queue is empty
        state_machine.set_enable(false);
        state_machine.tx().push(top);
        unsafe {
            state_machine.exec_instr(
                InstructionOperands::PULL {
                    if_empty: false,
                    block: false,
                }
                .encode(),
            );
            state_machine.exec_instr(
                InstructionOperands::OUT {
                    destination: ::pio::OutDestination::ISR,
                    bit_count: 32,
                }
                .encode(),
            );
        };
        if is_enabled {
            state_machine.set_enable(true) // Enable if previously enabled
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

    pub fn send_dma_buffer_to_pio(&mut self, buffer_num: u32) -> Transfer<'_, Dma0> {
        let dma_buffer = Self::get_writable_dma_buffer(buffer_num);
        self.state_machine
            .tx()
            .dma_push(self.dma_channel_0.reborrow(), dma_buffer)
    }

    #[allow(static_mut_refs)]
    pub fn get_writable_dma_buffer(buffer_num: u32) -> &'static mut [u8] {
        unsafe {
            if buffer_num == 0 {
                &mut DMA_BUFFER_0
            } else {
                &mut DMA_BUFFER_1
            }
        }
    }

    pub async fn play_sound(&mut self) {
        let (header, tracks) = midly::parse(include_bytes!("../assets/maple.mid"))
            .expect("It's inlined data, so its expected to parse");
        let mut midi = NewYearsMidi::new(&header, tracks);

        let mut playback_state = AudioPlayback::<PWM_BITS, REMAINDER_BITS>::new(&mut midi);
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
