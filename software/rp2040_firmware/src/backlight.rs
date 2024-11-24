use embassy_rp::gpio::Level;
use embassy_rp::pio::{
    Common, Direction, Instance, PioPin, ShiftConfig, ShiftDirection, StateMachine,
};
use pio::InstructionOperands;

// The backlight is a 2D array of R,G,B LED triplets per pixel
// Every screen update writes one row of LED on/off states at a time
// The row update bitstring has the form:
// [row_selector_bitstring], [row_pixel RGB bits]
pub struct Config {
    // The row_selector_bitstring has one bit for every row
    pub rows: u8,
    pub max_row_pixels: u8,
    pub num_intensity_levels: u8,
}

pub struct PioBacklight<'d, PIO: Instance, const SM: usize> {
    pub config: Config,
    pub sm: StateMachine<'d, PIO, SM>,
}
impl<'d, PIO: Instance, const SM: usize> PioBacklight<'d, PIO, SM> {
    pub fn new(
        common: &mut Common<'d, PIO>,
        mut sm: StateMachine<'d, PIO, SM>,
        config: Config,
        led_data_pin: impl PioPin,
        led_clk_pin: impl PioPin,
        led_latch_pin: impl PioPin,
        led_clear_pin: impl PioPin,
    ) -> Self {
        /*
            We are using daisy-chained TLC5926IDBQR shift registers, and supply
            these signals: LED_DATA, LED_CLK, LED_LATCH and LED_CLEAR.
            - LED_DATA bits are transferred into the shift register on the rising
            LED_CLK edge
            - When LED_LATCH is high, the shift register state is reflected in the
            output LED drivers
            - When LED_CLEAR is low, the output LED drivers are supplying power

            Basic sequence:
            - Set LED_LATCH low, LED_CLEAR low
            - Write a bit to LED_DATA and set LED_CLK high shortly after.
            Repeat until the shiftregisters are full.
            - Set LED_LATCH high and let data propagate to the LED drivers.
            - Repeat

            Shift register datasheet:
            https://www.ti.com/lit/ds/symlink/tlc5926.pdf?HQS=dis-dk-null-digikeymode-dsf-pf-null-wwe&ts=1732256906146&ref_url=https%253A%252F%252Fwww.ti.com%252Fgeneral%252Fdocs%252Fsuppproductinfo.tsp%253FdistId%253D10%2526gotoUrl%253Dhttps%253A%252F%252Fwww.ti.com%252Flit%252Fgpn%252Ftlc5926
        */

        // Set all pins to outputs
        let led_data_pin = common.make_pio_pin(led_data_pin);
        let led_clk_pin = common.make_pio_pin(led_clk_pin);
        let led_latch_pin = common.make_pio_pin(led_latch_pin);
        let led_clear_pin = common.make_pio_pin(led_clear_pin);
        sm.set_pin_dirs(
            Direction::Out,
            &[&led_data_pin, &led_clk_pin, &led_latch_pin, &led_clear_pin],
        );

        // Set all pins to low at the start
        // The LED_CLEAR input has an internal pullup (drivers off by default),
        // so we never touch it after this to keep the LED drivers always on
        sm.set_pins(
            Level::Low,
            &[&led_data_pin, &led_clk_pin, &led_latch_pin, &led_clear_pin],
        );

        let mut pio_cfg = embassy_rp::pio::Config::default();
        // The PIO state machine OUT command will only control LED_DATA
        pio_cfg.set_out_pins(&[&led_data_pin]);

        // Automatically refill the internal shift register from the FIFO
        // when OUT empties it
        pio_cfg.shift_out = ShiftConfig {
            auto_fill: true,
            threshold: 32,
            direction: ShiftDirection::Left,
        };

        let bits_per_row_minus_1 = config.rows + config.max_row_pixels * 3;
        // Load (bits per row - 1) to scratch registers so we set LED_LATCH after (bits per row) iterations
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

        /*
            update periods, assuming PULL never needs to wait on an empty FIFO:
            7 * 8 = 56ns/bit when finishing a row
            6 * 8 = 48ns/bit within a row
            => A screen costs (48ns/bit * bits / screen) + (8ns/row * rows/screen)

            The screen is 128 px with 7 rows => 1 row has max ceil(128/7) = 19 pixels
            3 colors/px * 19px => 57 pixel bits / row
            We have 1 shift register selector bit / row => every row needs 7 + 57 = 64 bits/row
            So we probably have 4 shift registers.
            TODO: Test whether the propagation delay between daisy chained shift registers is significant.

            => A screen costs (48ns/bit * (7 rows * 64 bits/row)) + (8ns/row * 7 rows) = 21.56us

            We control 256 pixel brightness states with PWM
            => 256 screen writes/image update
            => 21.56us/screen write * 256 screen writes/image update = ~5.5ms/image update
            => ~181 Hz image updates, assuming no extra delays needed for cascading
               bits between daisy chained shift registers
        */
        let prg = common.load_program(&prg.program);
        pio_cfg.use_program(&prg, &[&led_clk_pin, &led_latch_pin]);
        sm.set_config(&pio_cfg);
        Self { config, sm }
    }

    pub fn start(&mut self) {
        self.sm.set_enable(true);
    }
}
