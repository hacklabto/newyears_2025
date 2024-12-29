use embassy_rp::dma::Channel;
use embassy_rp::gpio::Level;
use embassy_rp::pio::{
    Common, Direction, Instance, PioPin, ShiftConfig, ShiftDirection, StateMachine,
};
use pio::InstructionOperands;

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
                // TSX FIFO -> OSR.  Block if the FIFO is empty
                // Set the output to 0
                //"pull side 0"
                //"mov x, osr"
                "out x,32  side 0"
                // y is the pwm hardware's equivalent of top
                // loaded using set_top
                "mov y, isr"

            // Loop y times, which is effectively top
            "countloop:"
                // Switch state to 1 when y matches the pwm value
                "jmp x!=y noset"
                "jmp skip        side 1"
            "noset:"
                // For a consistent 3 cycle delay
                "nop"
            "skip:"
                "jmp y-- countloop"
        );
        let prg = common.load_program(&prg.program);

        let sound_a_pin = common.make_pio_pin(sound_a_pin);
        sm.set_pins(Level::High, &[&sound_a_pin]);
        sm.set_pin_dirs(Direction::Out, &[&sound_a_pin]);

        let mut pio_cfg = embassy_rp::pio::Config::default();
        pio_cfg.use_program(&prg, &[&sound_a_pin]);

        pio_cfg.shift_out = ShiftConfig {
            auto_fill: true,
            threshold: 32,
            direction: ShiftDirection::Left,
        };

        sm.set_config(&pio_cfg);

        // errr
        let mut return_value = Self {
            state_machine: sm,
            dma_channel: dma_channel,
        };
        // for the LED test, we'll PWM values from 0-255 with a top of 512.
        return_value.set_top(512);
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

    pub fn set_level(&mut self, level: u32) {
        while !self.state_machine.tx().try_push(level) {}
    }

    pub async fn strobe_led_3x(&mut self) {
        for _i in 0..3 {
            for duration in 0..256 {
                // Target 2 seconds for each strobe.
                // clock speed is 125000000 , top is 512, 256 updates
                // 2 seconds * 125000000 / 512 / 256 / 3 = 635
                // Confirmed this is close by manually timing
                for _j in 0..635 {
                    self.set_level(duration);
                }
            }
        }
    }
}
