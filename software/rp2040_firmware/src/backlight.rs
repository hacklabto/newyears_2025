use core::sync::atomic::{AtomicU8, Ordering};
use embassy_rp::bind_interrupts;
use embassy_rp::dma::Channel;
use embassy_rp::gpio;
use embassy_rp::peripherals::PIO1;
use embassy_rp::pio::program::pio_asm;
use embassy_rp::pio::InterruptHandler;
use embassy_rp::pio::Pio;
use embassy_rp::pio::{Direction, FifoJoin, PioPin, ShiftConfig, ShiftDirection, StateMachine};
use embassy_rp::Peri;
use fixed::traits::ToFixed;
use gpio::{Level, Output, Pin};
use pio::InstructionOperands;
use embassy_time::{Duration, Timer};

pub const LED_COLUMNS: usize = 9;
pub const LED_ROWS: usize = 5;
const COLORS_PER_LED: usize = 3;
const LED_LEVELS: usize = 256;
const LED_DMA_BUFFER_SIZE: usize = LED_ROWS * LED_LEVELS;

pub const fn init_dma_buffer() -> [u32; LED_DMA_BUFFER_SIZE] {
    // This should light up all the LEDs
    let init_dma_buffer: [u32; LED_DMA_BUFFER_SIZE] = [0xffffffff; LED_DMA_BUFFER_SIZE];
    init_dma_buffer
}

// Why three buffers?  Okay, first, this really should be a pipe :(
//
// The third buffer is giving a bit of head room between the buffer being DMAed to
// the PIO and the buffer being written.
//

#[allow(clippy::declare_interior_mutable_const)]
static mut DMA_BUFFER_0: [u32; LED_DMA_BUFFER_SIZE] = init_dma_buffer();
static mut DMA_BUFFER_1: [u32; LED_DMA_BUFFER_SIZE] = init_dma_buffer();
static mut DMA_BUFFER_2: [u32; LED_DMA_BUFFER_SIZE] = init_dma_buffer();
static DMA_READ_BUFFER: AtomicU8 = AtomicU8::new(0);

#[allow(static_mut_refs)]
fn get_read_dma_buffer() -> &'static mut [u32] {
    let read_buffer: u8 = DMA_READ_BUFFER.load(Ordering::Relaxed);
    unsafe {
        if read_buffer == 0 {
            &mut DMA_BUFFER_0
        } else if read_buffer == 1 {
            &mut DMA_BUFFER_1
        } else {
            &mut DMA_BUFFER_2
        }
    }
}

#[allow(static_mut_refs)]
fn get_write_dma_buffer() -> &'static mut [u32] {
    let write_buffer: u8 = (DMA_READ_BUFFER.load(Ordering::Relaxed) + 1) % 3;
    unsafe {
        if write_buffer == 0 {
            &mut DMA_BUFFER_0
        } else if write_buffer == 1 {
            &mut DMA_BUFFER_1
        } else {
            &mut DMA_BUFFER_2
        }
    }
}

fn advance_read_dma_buffer() {
    let read_buffer: u8 = DMA_READ_BUFFER.load(Ordering::Relaxed);
    let new_read_buffer = (read_buffer + 1) % 3;
    DMA_READ_BUFFER.store(new_read_buffer, Ordering::Relaxed);
}

bind_interrupts!(struct PioIrqs1 {
    PIO1_IRQ_0 => InterruptHandler<embassy_rp::peripherals::PIO1>;
});

pub struct LedLevel {
    level: u8,
    dither: u8,
}

impl LedLevel {
    #[inline]
    pub fn set(self: &mut Self, new_level: u8) {
        self.level = new_level;
    }
    #[inline]
    pub fn update(self: &mut Self) -> u32 {
        let orig = self.dither;
        self.dither = self.dither.wrapping_add(self.level);
        if orig < self.dither {
            0
        } else {
            1
        }
    }
}

impl Default for LedLevel {
    fn default() -> Self {
        Self {
            level: 0,
            dither: 0,
        }
    }
}

// User interface/ DMA buffer updater
pub struct BacklightUser {
    led_levels: [[LedLevel; LED_COLUMNS * COLORS_PER_LED]; LED_ROWS],
}

impl BacklightUser {
    pub fn set(self: &mut Self, column: usize, row: usize, r: u8, g: u8, b: u8) {
        // I know, I'm trusting the optimizer a lot.
        self.led_levels[column][row * 3 + 0].set(r);
        self.led_levels[column][row * 3 + 1].set(g);
        self.led_levels[column][row * 3 + 2].set(b);
    }
    #[inline]
    pub fn assemble_column(self: &mut Self, column: usize) -> u32 {
        //
        // u32 stored as little endian, read into the IO as big endian.  Lol.
        //
        // Shere's the shift register bit to u32 mapping
        //
        // Hardware shift reg 00     u32 bit 7
        // Hardware shift reg 01     u32 bit 6
        // Hardware shift reg 02     u32 bit 5
        // Hardware shift reg 03     u32 bit 4
        // Hardware shift reg 04     u32 bit 3
        // Hardware shift reg 05     u32 bit 2
        // Hardware shift reg 06     u32 bit 1
        // Hardware shift reg 07     u32 bit 0
        // Hardware shift reg 08     u32 bit 15 
        // Hardware shift reg 09     u32 bit 14
        // Hardware shift reg 10     u32 bit 13
        // Hardware shift reg 11     u32 bit 12
        // Hardware shift reg 12     u32 bit 11
        // Hardware shift reg 13     u32 bit 10
        // Hardware shift reg 14     u32 bit 9
        // Hardware shift reg 15     u32 bit 8
        // Hardware shift reg 16     u32 bit 23
        // Hardware shift reg 17     u32 bit 22
        // Hardware shift reg 18     u32 bit 21
        // Hardware shift reg 19     u32 bit 20
        // Hardware shift reg 20     u32 bit 19
        // Hardware shift reg 21     u32 bit 18
        // Hardware shift reg 22     u32 bit 17
        // Hardware shift reg 23     u32 bit 16
        // Hardware shift reg 24     u32 bit 31
        // Hardware shift reg 25     u32 bit 30
        // Hardware shift reg 26     u32 bit 29
        // Hardware shift reg 27     u32 bit 28
        // Hardware shift reg 28     u32 bit 27
        // Hardware shift reg 29     u32 bit 26
        // Hardware shift reg 30     u32 bit 25
        // Hardware shift reg 31     u32 bit 24

        // Will this go fast?  If the assembler doesn't work out, refactoring is an option
        // Also, might need to change indexing so the index is more intuitive.
        (self.led_levels[column][00].update() << 02)
            | (self.led_levels[column][01].update() << 01)
            | (self.led_levels[column][02].update() << 00)
            | (self.led_levels[column][03].update() << 15)
            | (self.led_levels[column][04].update() << 14)
            | (self.led_levels[column][05].update() << 13)
            | (self.led_levels[column][06].update() << 12)
            | (self.led_levels[column][07].update() << 11)
            | (self.led_levels[column][08].update() << 10)
            | (self.led_levels[column][09].update() <<  9)
            | (self.led_levels[column][10].update() <<  8)
            | (self.led_levels[column][11].update() << 23)
            | (self.led_levels[column][12].update() << 22)
            | (self.led_levels[column][13].update() << 21)
            | (self.led_levels[column][14].update() << 20)
            | (self.led_levels[column][15].update() << 19)
            | (self.led_levels[column][16].update() << 18)
            | (self.led_levels[column][17].update() << 17)
            | (self.led_levels[column][18].update() << 16)
            | (self.led_levels[column][19].update() << 31)
            | (self.led_levels[column][20].update() << 30)
            | (self.led_levels[column][21].update() << 29)
            | (self.led_levels[column][22].update() << 28)
            | (self.led_levels[column][23].update() << 27)
            | (self.led_levels[column][24].update() << 26)
            | (self.led_levels[column][25].update() << 25)
            | (self.led_levels[column][26].update() << 24)
    }

    pub fn update_led_dma_buffer(self: &mut Self) {
        let mut row:  u32 = 0;
        let mut row_usize:  usize = 0;
        let dma_buffer = get_write_dma_buffer();

        for item in dma_buffer.iter_mut() {
            *item = (8 << row) | self.assemble_column(row_usize);
            row = row + 1;
            row_usize = row_usize + 1;
            if row_usize >= LED_ROWS {
                row = 0;
                row_usize = 0;
            }
        }
        advance_read_dma_buffer();
    }
}

impl Default for BacklightUser {
    fn default() -> Self {
        Self {
            led_levels: core::array::from_fn(|_idx| {
                core::array::from_fn(|_idx| LedLevel::default())
            }),
        }
    }
}

pub struct PioBacklight<'d, Dma1: Channel> {
    pub state_machine: StateMachine<'d, PIO1, 0>,

    // TODO: May need to add double buffering. Decide after testing on the hardware. For now, just use it for testing.
    pub dma_channel: Peri<'d, Dma1>,
    test_clk_pin: Output<'d>,
    test_data_pin: Output<'d>,
    test_latch_pin: Output<'d>,
    test_blank_pin: Output<'d>,
    cycle: u32,
}


impl<'d, Dma1: Channel> PioBacklight<'d, Dma1> {
    pub fn new(
        arg_pio: Peri<'d, PIO1>,
        led_data_pin: Peri<'d, impl PioPin>,
        led_clk_pin: Peri<'d, impl PioPin>,
        led_latch_pin: Peri<'d, impl PioPin>,
        led_blank_pin: Peri<'d, impl PioPin>,
        test_data: Peri<'d, impl Pin>,
        test_clk: Peri<'d, impl Pin>,
        test_latch: Peri<'d, impl Pin>,
        test_blank: Peri<'d, impl Pin>,
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
            ".side_set 3 opt"

            //
            // The shift register has 32 registers in it, so we want to latch out once
            // every 32 bits read.  The reads are 27 bits of valid "row" data and 5 bits to
            // select a row.  So it makes sense to DMA over u32s.
            //
            // 5 u32s give us a column, and 1280 u32s give us a pattern with 256
            // levels of RGB for each of the 45 LEDs.
            //
            "start_fill_row:"
                "set y, 31"

            "fillrow_bit:"
                // Get a bit from the FIFO...
                "out x, 1"
                "jmp !x, input_is_zero"
                // Input is 0.  Bit 0 = clk, bit 1 = data, bit 2 = latch
                // shift in a a 0
                "nop        side 0b010 [1]"
                // The delay to the next instruction is 1/125000000, or 8ns.
                // 8ns > LED_DATA t_setup (3ns) has passed, so bring CLK high to latch LED_DATA in.
                // From the spec sheet, data latches in on a positive clock edge.
                "nop        side 0b011 [1]"
                // The delay to the next instruction is 1/125000000, or 8ns.
                // 8ns > LED_DATA t_hold (4ns) has passed.  Bring the clock back down
                "nop        side 0b010"
                // Loop around until the row is full (32 values, stored in y)
                "jmp y--, fillrow_bit"
                "jmp do_latching"

            "input_is_zero:"
                // Input is 1.  Bit 0 = clk, bit 1 = data, bit 2 = latch
                // shift in a a 0
                "nop        side 0b000 [1]"
                // The delay to the next instruction is 1/125000000, or 8ns.
                // 8ns > LED_DATA t_setup (3ns) has passed, so bring CLK high to latch LED_DATA in.
                // From the spec sheet, data latches in on a positive clock edge.
                "nop        side 0b001 [1]"
                // The delay to the next instruction is 1/125000000, or 8ns.
                // 8ns > LED_DATA t_hold (4ns) has passed.  Bring the clock back down
                "nop        side 0b000"
                // Loop around until the row is full (32 values, stored in y)
                "jmp y--, fillrow_bit"

            "do_latching:"
                // It's been 16ns > 15ns since LED_CLK went high, so this is ok.  Also, we're
                // holding this high for a good block of time.
                "nop        side 0b100 [1]"
                "nop        side 0b000"

                "jmp start_fill_row"
        );
        let program = common.load_program(&prg.program);

        // Set all pins to outputs
        let led_data_pin = common.make_pio_pin(led_data_pin);
        let led_clk_pin = common.make_pio_pin(led_clk_pin);
        let led_latch_pin = common.make_pio_pin(led_latch_pin);
        let led_blank_pin = common.make_pio_pin(led_blank_pin);

        let pio1_cfg = {
            let mut cfg = embassy_rp::pio::Config::default();
            // 0 - data pin
            // 1 - clk pin
            // 2 - latch pin
            cfg.use_program(&program, &[&led_clk_pin, &led_data_pin, &led_latch_pin]);
            // No out ins used by this set machine...

            cfg.clock_divider = 1.to_fixed();

            // Automatically refill the internal shift register from the FIFO
            // when OUT empties it
            cfg.shift_out = ShiftConfig {
                auto_fill: true,
                threshold: 32,
                direction: ShiftDirection::Right,
            };
            cfg.fifo_join = FifoJoin::TxOnly;
            cfg
        };

        sm.set_config(&pio1_cfg);

        sm.set_pin_dirs(
            Direction::Out,
            &[&led_clk_pin, &led_data_pin, &led_latch_pin, &led_blank_pin],
        );
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
            test_blank_pin: Output::new(test_blank, Level::Low),
            cycle: 0,
        }
    }

    pub async fn delay() {
        Timer::after(Duration::from_micros(10)).await;
    }

    // Test Pattern Notes...
    //
    // CLK is documented as clock input pin for data shift on rising edge...
    // SDI/ data is latched in on the falling clock 
    // LE data strobe.  Data is latched when LE goes low.
    // Blank is active high
    //
    // The bc807 might be active low.
    //
    pub async fn test_pattern_2(&mut self) {
        // Set latch to high so we just output whatever comes in
        self.test_latch_pin.set_low();
        // Clear/ blank is active low
        self.test_blank_pin.set_low();
            let mut bit_count: u32 = 0;
            while bit_count < 32
            {
                // 32 shift registers total...
                //
                // This pattern (alternating on and off) does not appear
                // when the transistor board pins are tested.  
                // TODO, get the scope on the breadboard outputs.
                //
                if bit_count >= 32-4 {
                    // Active high = current sink = enable BC807
                    self.test_data_pin.set_high();
                }
                else if bit_count == 32-5 {
                    // Turn one BC807 off for testing
                    self.test_data_pin.set_low();
                }
                else {
                    if self.cycle == bit_count {
                        self.test_data_pin.set_high();
                    }
                    else {
                        self.test_data_pin.set_low();
                    }
                }
                Self::delay().await;
                self.test_clk_pin.set_high();
                Self::delay().await;
                self.test_clk_pin.set_low();
                Self::delay().await;

                bit_count = bit_count + 1;
            }

            // From the data sheet...  I'm not 100% sure how
            // much this pattern is needed...
            
            self.test_latch_pin.set_high();
            Self::delay().await;
            self.test_latch_pin.set_low();
            Self::delay().await;

            Timer::after(Duration::from_millis(1)).await;

        self.cycle = self.cycle + 1;
        if self.cycle > 32-5 {
            self.cycle = 0;
        }

        Timer::after(Duration::from_millis(100)).await;
    }

    pub async fn test_pattern(&mut self) {
        let dma_buffer = get_read_dma_buffer();

        //dma_buffer[0] = 0x000000f8 | (self.cycle & 7) | ((self.cycle >> 3) << 8);

            defmt::info!("Single DMA Launch");
        let dma_buffer_in_flight =
            self.state_machine
                .tx()
                .dma_push(self.dma_channel.reborrow(), dma_buffer, true);
            defmt::info!("Single DMA Launch Return");

        dma_buffer_in_flight.await;
            defmt::info!("Dma Packet Await Finished");
        Timer::after(Duration::from_millis(1)).await;
        self.cycle = self.cycle + 1;
    }
    //pub fn start(&mut self) {
    //    self.state_machine.set_enable(true);
    //}

    pub async fn display_one_frame(&mut self) {
        let dma_buffer = get_read_dma_buffer();

        let dma_buffer_in_flight =
            self.state_machine
                .tx()
                .dma_push(self.dma_channel.reborrow(), dma_buffer, true);

        dma_buffer_in_flight.await;
        Timer::after(Duration::from_micros(1)).await;
    }
}
