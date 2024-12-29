use core::time::Duration;
use embassy_rp::clocks;
use embassy_rp::dma::Channel;
use embassy_rp::gpio::Level;
use embassy_rp::pio::{Common, Direction, Instance, PioPin, StateMachine};
use embassy_time::Timer;
use pio::InstructionOperands;

const REFRESH_INTERVAL: u64 = 20000;

pub fn to_pio_cycles(duration: Duration) -> u32 {
    (clocks::clk_sys_freq() / 1_000_000) / 3 * duration.as_micros() as u32 // parentheses are required to prevent overflow
}

pub struct PioSound<'d, PIO: Instance, const STATE_MACHINE_IDX: usize, DMA: Channel> {
    pub state_machine: StateMachine<'d, PIO, STATE_MACHINE_IDX>,

    // TODO: May need to add double buffering. Decide after testing on the hardware. For now, just use it for testing.
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
                "pull noblock    side 0"
                "mov x, osr"
                "mov y, isr"
            "countloop:"
                "jmp x!=y noset"
                "jmp skip        side 1"
            "noset:"
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

        sm.set_config(&pio_cfg);

        // errr
        let mut return_value = Self {
            state_machine: sm,
            dma_channel: dma_channel,
        };
        return_value.set_period(Duration::from_micros(REFRESH_INTERVAL));
        return_value.start();
        return_value
    }

    pub fn set_period(&mut self, duration: Duration) {
        let is_enabled = self.state_machine.is_enabled();
        while !self.state_machine.tx().empty() {} // Make sure that the queue is empty
        self.state_machine.set_enable(false);
        self.state_machine.tx().push(to_pio_cycles(duration));
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
        self.state_machine.tx().push(level);
    }

    pub fn write(&mut self, duration: Duration) {
        self.set_level(to_pio_cycles(duration));
    }

    pub async fn strobe_led_3x(&mut self) {
        for _i in 0..3 {
            for duration in 0..1000 {
                self.write(Duration::from_micros(duration));
                Timer::after_millis(1).await;
            }
        }
    }
}
