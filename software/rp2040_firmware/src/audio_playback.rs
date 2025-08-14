use midi_nostd::midi::Midi;
type NewYearsMidi<'a> = Midi<'a, 20292, { 89 * 3 }, 64, 32>;

pub struct AudioPlayback<'d, const PWM_BITS: u32, const PWM_REMAINDER_BITS: u32> {
    midi: &'d mut NewYearsMidi<'d>,
    clear_count: u32,
}

const fn generate_dither_array<const N: usize>() -> [u32; N] {
    let mut array = [0; N];
    let mut idx: usize = 0;
    while idx < N {
        let mut remainder: u32 = (N as u32) / 2;
        let mut result: u32 = 0;
        let mut j: usize = 0;
        while j < N {
            result = result << 1;
            remainder = remainder + (idx as u32);
            if remainder >= N as u32 {
                remainder -= N as u32;
                result |= 1;
            }
            j = j + 1;
        }
        array[idx] = result;
        idx = idx + 1;
    }
    return array;
}

impl<'d, const PWM_BITS: u32, const PWM_REMAINDER_BITS: u32>
    AudioPlayback<'d, PWM_BITS, PWM_REMAINDER_BITS>
{
    const PWM_TOP: u32 = 1 << PWM_BITS;
    const PWM_REMAINDER: u32 = 1 << PWM_REMAINDER_BITS;

    // 0 to 0x8000, or 2^15, is the range we get from the midi player.
    const PWM_TOP_SHIFT: u32 = 15 - PWM_BITS;
    const PWM_REMAINDER_SHIFT: u32 = Self::PWM_TOP_SHIFT - PWM_REMAINDER_BITS;

    const DITHERS: [u32; 16] = generate_dither_array::<16>();
    const DITHER_ARRAY_RIGHT_SIZE: () = assert!(16 == Self::PWM_REMAINDER); // should match array size

    pub fn new(midi: &'d mut NewYearsMidi<'d>) -> Self {
        let _ = Self::DITHER_ARRAY_RIGHT_SIZE; // execute static assert
        let clear_count: u32 = 0;
        Self { midi, clear_count }
    }

    pub fn populate_next_dma_buffer_with_audio(&mut self, buffer: &mut [u32]) {
        let mut value: u32 = 0;
        let sign_bits: u32 = 0;
        let mut dither: u32 = 0;
        let mut countdown: u32 = 0;

        for entry in buffer.iter_mut() {
            // refill at 0
            if countdown == 0 {
                //
                // We're're here once every PWM_REMAINDER, which is, right now, every 16 iterations.
                // Hacklab speaker loud.  Divide by 2.
                //
                let value_raw: i32 = self.midi.get_next().to_i32();

                //
                // Right now I'm taking an absolute value of the sound output so that if the sound
                // is zero I'm hot trying to hold the speaker at 50% power by pulsing half the
                // time.  That generates a lot of noise.  The current scheme generates noise too,
                // but at least some of the PWM noise is hidden in the music/ signal.
                //

                let value_abs: u32 = if value_raw >= 0 {
                    value_raw as u32
                } else {
                    (-value_raw) as u32
                };

                //
                // Value is what I'm sending to the PIO hardware to be PWMed
                //
                let value_u32: u32 = value_abs >> Self::PWM_TOP_SHIFT;

                //
                // Remainder is the bits below value in the sound sample.  I'm
                // ditherings I'm sending to the PIO to increase bit count.
                //
                let remainder =
                    (value_abs >> Self::PWM_REMAINDER_SHIFT) & (Self::PWM_REMAINDER - 1);

                //
                // DITHERs is basically a dither pattern for the current remainder.  If
                // PWM_REMAINDER is 16 then DITHERS[remainder=0] should be
                //
                // 0b0000000000000000
                //
                // and DITHERS[remainder=8] should be
                //
                // 0b0101010101010101
                //
                dither = Self::DITHERS[remainder as usize];
                value = if value_u32 >= Self::PWM_TOP {
                    Self::PWM_TOP - 1
                } else {
                    value_u32
                };
                if !self.midi.has_next() {
                    self.clear_count = 1;
                }
                countdown = Self::PWM_REMAINDER;
            }
            //
            // Fairly low overhead sound buffer population
            //
            *entry = ((value + (dither & 1) << 1)
                | ((value + ((dither & 2) >> 1)) << 9)
                | ((value + ((dither & 4) >> 2)) << 17)
                | ((value + ((dither & 8) >> 3)) << 25))
                | sign_bits;
            dither = dither >> 4;
            countdown = countdown - 4;
        }
    }

    pub fn is_done(&self) -> bool {
        return self.clear_count == 1;
    }
}
