use embassy_rp::dma::Channel;
use embassy_rp::gpio::Level;
use embassy_rp::pio::{
    Common, Direction, Instance, PioPin, ShiftConfig, ShiftDirection, StateMachine,
};
use fixed::traits::ToFixed;
use fixed_macro::types::U56F8;
use pio::InstructionOperands;

const TARGET_PLAYBACK: u64 = 24_000;
const PWM_TOP: u64 = 512;
const PWM_CYCLES_PER_READ: u64 = 6 * PWM_TOP + 4;

pub struct PioSound<'d, PIO: Instance, const STATE_MACHINE_IDX: usize, DMA: Channel> {
    pub state_machine: StateMachine<'d, PIO, STATE_MACHINE_IDX>,
    pub dma_channel: DMA,
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
            dma_channel: dma_channel,
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

    pub async fn strobe_led_3x(&mut self) {
        for _i in 0..30 {
            for duration in 0..=240 {
                // Target 1 seconds for each strobe.
                // 240 * 100 = 24000, our target playback speed
                for _j in 0..100 {
                    self.set_level(duration);
                }
            }
        }
        self.set_level(0x20);
    }
}
