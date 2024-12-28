use embassy_rp::dma::Channel;
use embassy_rp::gpio::Level;
use embassy_rp::pio::{
    Common, Direction, Instance, PioPin, ShiftConfig, ShiftDirection, StateMachine,
};
//use pio::InstructionOperands;

pub struct PioSoundConfig {
}

pub struct PioSound<'d, PIO: Instance, const STATE_MACHINE_IDX: usize, DMA: Channel> {
    pub config: PioSoundConfig,
    pub state_machine: StateMachine<'d, PIO, STATE_MACHINE_IDX>,

    // TODO: May need to add double buffering. Decide after testing on the hardware. For now, just use it for testing.
    pub dma_channel: DMA,
}
impl<'d, PIO: Instance, const STATE_MACHINE_IDX: usize, DMA: Channel>
    PioSound<'d, PIO, STATE_MACHINE_IDX, DMA>
{
    pub fn new(
        config: PioSoundConfig,
        common: &mut Common<'d, PIO>,
        mut sm: StateMachine<'d, PIO, STATE_MACHINE_IDX>,
        sound_a_pin: impl PioPin,
        sound_b_pin: impl PioPin,
        dma_channel: DMA,
    ) -> Self {

        // Set all pins to outputs
        let sound_a_pin = common.make_pio_pin(sound_a_pin);
        let sound_b_pin = common.make_pio_pin(sound_b_pin);
        sm.set_pin_dirs(
            Direction::Out,
            &[&sound_a_pin, &sound_b_pin],
        );

        // Set all pins to low at the start
        // The LED_CLEAR input has an internal pullup (drivers off by default),
        // so we never touch it after this to keep the LED drivers always on
        sm.set_pins(
            Level::Low,
            &[&sound_a_pin, &sound_b_pin],
        );

        let mut pio_cfg = embassy_rp::pio::Config::default();
        // The PIO state machine OUT command will only control Sound A for now
        pio_cfg.set_out_pins(&[&sound_a_pin]);

        // Automatically refill the internal shift register from the FIFO
        // when OUT empties it
        pio_cfg.shift_out = ShiftConfig {
            auto_fill: true,
            threshold: 32,
            direction: ShiftDirection::Left,
        };

        /*
        unsafe {
            sm.exec_instr(
                InstructionOperands::SET {
                    destination: pio::SetDestination::X,
                    data: bits_per_row_minus_1,
                }
                .encode(),
            );
            sm.exec_instr(
                InstructionOperands::SET {
                    destination: pio::SetDestination::Y,
                    data: bits_per_row_minus_1,
                }
                .encode(),
            );
        }
        */

        /*
            Timing constraints from shift register datasheet:
            - LED_CLK: Min clock pulse width = 20ns
            - LED_DATA needs to be held 3ns before the rising LED_CLK edge and 4ns after.
            - LED_LATCH must be held high for 20ns, can only go low 15ns after the
            rising clock edge, and the next rising clock edge can only start 15ns
            after LED_LATCH goes low.

            Applied to the sequence:
            - Write to LED_DATA, hold for 3ns
            - Rising LED_CLK, hold LED_DATA for 4ns.
            - Clock remains high for 20ns, then we set it low for 1 cycle (we can
                set LED_LATCH in parallel)
            - If ready to write,
                - Hold LED_LATCH high for at least 20ns
                - Hold LED_LATCH low for 15ns before next clock
        */

        // LED_CLK and LED_LATCH will be controlled via side-set commands
        // (i.e. can be set in parallel with other PIO assembly commands)
        let prg = pio_proc::pio_asm!(
            // Note: At default 125MHz clock, every instruction except the
            // blocking PULL takes 8ns
            ".side_set 2 opt"
            "fillrow:"
                // Load the LED_DATA bit
                "out pins, 1"
                // 8ns > LED_DATA t_setup (3ns) has passed, so bring CLK high
                "nop        side 0b01"
                // 8ns > LED_DATA t_hold (4ns) has passed
                // Skip latch if row is not full
                "jmp x--, skiplatch"

                // Row is full, so bring LED_LATCH high and keep clock high
                // It's been 16ns > 15ns since LED_CLK went high, so this is ok
                "mov x, y   side 0b11"
                // 24ns > min LED_CLK pulse (20ns) has passed, so we can set it low.
                // Wait one extra cycle so 24ns > min LED_LATCH pulse (20ns) passes
                "nop        side 0b01 [1]"

                // Set LED_LATCH low so there's 16ns > 15ns until the next
                // LED_CLK high edge
                "jmp fillrow side 0b00"

            "skiplatch:"
                // Wait one extra cycle so 24ns > min LED_CLK pulse (20ns) passes
                "nop"
                // The shift register datasheet doesn't specify a min LED_CLK off period
                // To be safe, turn it off and wait an extra cycle so at least
                // 24ns (> 20ns min LED_CLK high pulse) passes before it's set high again
                // TODO: Test whether this is necessary
                "nop        side 0b00 [1]"
        );

        let prg = common.load_program(&prg.program);
        pio_cfg.use_program(&prg, &[&sound_a_pin, &sound_b_pin]);
        sm.set_config(&pio_cfg);
        Self {
            config: config,
            state_machine: sm,
            dma_channel: dma_channel,
        }
    }

    pub fn start(&mut self) {
        self.state_machine.set_enable(true);
    }
}
