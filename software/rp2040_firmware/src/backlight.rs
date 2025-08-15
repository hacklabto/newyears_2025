use embassy_rp::bind_interrupts;
use embassy_rp::dma::Channel;
use embassy_rp::gpio;
use embassy_rp::peripherals::PIO1;
use embassy_rp::pio::program::pio_asm;
use embassy_rp::pio::InterruptHandler;
use embassy_rp::pio::Pio;
use embassy_rp::pio::{Direction, FifoJoin, PioPin, ShiftConfig, ShiftDirection, StateMachine};
use embassy_rp::Peri;
use embassy_time::Instant;
use fixed::traits::ToFixed;
use gpio::{Level, Output, Pin};
use pio::InstructionOperands;

const LED_ROWS: usize = 5;
const LED_LEVELS: usize = 256;
const LED_DMA_BUFFER_SIZE: usize = LED_ROWS * LED_LEVELS;

pub const fn init_dma_buffer() -> [u32; LED_DMA_BUFFER_SIZE] {
    let mut init_dma_buffer: [u32; LED_DMA_BUFFER_SIZE] = [0; LED_DMA_BUFFER_SIZE];
    let mut row = 0;
    let mut idx: usize = 0;

    while idx < LED_DMA_BUFFER_SIZE {
        init_dma_buffer[idx] = 1 << (row + 27);
        row = (row + 1) % 5;
        idx = idx + 1;
    }

    init_dma_buffer
}

#[allow(clippy::declare_interior_mutable_const)]
static mut DMA_BUFFER: [u32; LED_DMA_BUFFER_SIZE] = init_dma_buffer();

bind_interrupts!(struct PioIrqs1 {
    PIO1_IRQ_0 => InterruptHandler<embassy_rp::peripherals::PIO1>;
});

pub struct PioBacklight<'d, Dma1: Channel> {
    pub state_machine: StateMachine<'d, PIO1, 0>,

    // TODO: May need to add double buffering. Decide after testing on the hardware. For now, just use it for testing.
    pub dma_channel: Peri<'d, Dma1>,
    test_clk_pin: Output<'d>,
    test_data_pin: Output<'d>,
    test_latch_pin: Output<'d>,
    test_clear_pin: Output<'d>,
}

impl<'d, Dma1: Channel> PioBacklight<'d, Dma1> {
    pub fn new(
        arg_pio: Peri<'d, PIO1>,
        led_data_pin: Peri<'d, impl PioPin>,
        led_clk_pin: Peri<'d, impl PioPin>,
        led_latch_pin: Peri<'d, impl PioPin>,
        led_clear_pin: Peri<'d, impl PioPin>,
        test_clk: Peri<'d, impl Pin>,
        test_data: Peri<'d, impl Pin>,
        test_latch: Peri<'d, impl Pin>,
        test_clear: Peri<'d, impl Pin>,
        dma_channel: Peri<'d, Dma1>,
    ) -> Self {
        let pio = Pio::new(arg_pio, PioIrqs1);
        let mut common = pio.common;
        let mut sm = pio.sm0;

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

        // LED_CLK and LED_LATCH will be controlled via side-set commands
        // (i.e. can be set in parallel with other PIO assembly commands)
        let prg = pio_asm!(
            // Note: At default 125MHz clock, every instruction except the
            // blocking PULL takes 8ns
            ".side_set 2 opt"

            //
            // The shift register has 32 registers in it, so we want to latch out once
            // every 32 bits read.  The reads are 27 bits of valid "row" data and 5 bits to
            // select a row.  So it makes sense to DMA over u32s.
            //
            // 5 u32s give us a column, and 1280 u32s give us a pattern with 256
            // levels of RGB for each of the 45 LEDs.
            //
            "start_fill_row:"
                "mov x, y"

            "fillrow:"
                // Load the LED_DATA bit from the FIFO
                "out pins, 1"
                // 8ns > LED_DATA t_setup (3ns) has passed, so bring CLK high to latch LED_DATA in
                // TODO, confirm the clock latches on a positive edge.  It looks like it does
                // from the test program.  TODO, get this on the scope.
                "nop        side 0b10"
                // 8ns > LED_DATA t_hold (4ns) has passed.  Bring the clock back down
                "nop        side 0b00"
                // Loop around until the row is full (32 values, stored in y)
                "jmp x--, fillrow"

                // Row is full, so bring LED_LATCH high and keep clock low.
                // TODO, check data sheet to see if I need a pause before I set latch to high
                // TODO, the low clock is what we did when I doing the hardware test code,
                // and might not be right.  Check data sheet
                // TODO, On the same test code I also changed the clear.  Check data sheet.

                // It's been 16ns > 15ns since LED_CLK went high, so this is ok
                "mov x, ISR   side 0b01"

            "latch_on_delay_loop:"
                "jmp x--, latch_on_delay_loop"

                // 24ns > min LED_CLK pulse (20ns) has passed, so we can set it low.
                // Wait one extra cycle so 24ns > min LED_LATCH pulse (20ns) passes
                // TODO, data sheet for any timings needed her before we latch in the next row
                "nop        side 0b00 [1]"

                "jmp start_fill_row"
        );
        let prg = common.load_program(&prg.program); // TODO, name overlap

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

        pio_cfg.use_program(&prg, &[&led_clk_pin, &led_latch_pin]);

        // Automatically refill the internal shift register from the FIFO
        // when OUT empties it
        pio_cfg.shift_out = ShiftConfig {
            auto_fill: true,
            threshold: 32,
            direction: ShiftDirection::Right,
        };
        pio_cfg.fifo_join = FifoJoin::TxOnly;

        pio_cfg.clock_divider = 1.to_fixed();
        sm.set_config(&pio_cfg);

        // TODO, improve this in terms of readability

        //
        // Set the delay for the latch on by pushing a value to the state machine,
        // 64, and then executing some assembler to pull it and send it to the ISR
        //
        sm.tx().push(64);
        unsafe {
            sm.exec_instr(
                InstructionOperands::PULL {
                    if_empty: false,
                    block: false,
                }
                .encode(),
            );
            sm.exec_instr(
                InstructionOperands::OUT {
                    destination: ::pio::OutDestination::ISR,
                    bit_count: 32,
                }
                .encode(),
            );
        };
        sm.tx().push(32);
        unsafe {
            sm.exec_instr(
                InstructionOperands::PULL {
                    if_empty: false,
                    block: false,
                }
                .encode(),
            );
            sm.exec_instr(
                InstructionOperands::OUT {
                    destination: ::pio::OutDestination::Y,
                    bit_count: 32,
                }
                .encode(),
            );
        };

        // Turn on the machine.  What could go wrong.

        sm.set_enable(true);

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
        Self {
            state_machine: sm,
            dma_channel: dma_channel,
            test_clk_pin: Output::new(test_clk, Level::Low),
            test_data_pin: Output::new(test_data, Level::Low),
            test_latch_pin: Output::new(test_latch, Level::Low),
            test_clear_pin: Output::new(test_clear, Level::Low),
        }
    }

    pub fn delay() {
        let start_time = Instant::now();
        while start_time.elapsed().as_millis() < 2 {}
    }

    pub fn test_pattern(&mut self) {
        self.test_latch_pin.set_high();
        self.test_clear_pin.set_low();
        let mut count: u32 = 0;
        while count < 20 {
            let mut bit_count: u32 = 0;
            while bit_count < 32 {
                self.test_data_pin.set_high();
                Self::delay();
                self.test_clk_pin.set_high();
                Self::delay();
                self.test_clk_pin.set_low();
                Self::delay();
                self.test_data_pin.set_high();
                Self::delay();
                self.test_clk_pin.set_high();
                Self::delay();
                self.test_clk_pin.set_low();
                Self::delay();
                bit_count = bit_count + 1;
            }
            self.test_clear_pin.set_high();
            Self::delay();
            self.test_latch_pin.set_high();
            Self::delay();
            self.test_latch_pin.set_low();
            Self::delay();
            self.test_clear_pin.set_low();
            Self::delay();

            let mut delay_count: u32 = 0;
            while delay_count < 50 {
                Self::delay();
                delay_count = delay_count + 1;
            }
            count = count + 1;
        }
    }

    pub fn start(&mut self) {
        self.state_machine.set_enable(true);
    }

    #[allow(static_mut_refs)]
    pub async fn display_and_update(&mut self) {
        let dma_buffer = unsafe { &DMA_BUFFER };

        let dma_buffer_in_flight =
            self.state_machine
                .tx()
                .dma_push(self.dma_channel.reborrow(), dma_buffer, true);

        // TODO, update code.

        dma_buffer_in_flight.await;
    }
}
