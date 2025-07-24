use crate::midi_notes::FREQUENCY_MULTIPLIER;
use crate::sound_sample::SoundSampleI32;
use crate::sound_source_core::SoundSourceCore;
use crate::wave_tables::SAWTOOTH_WAVE;
use crate::wave_tables::SINE_WAVE;
use crate::wave_tables::SQUARE_WAVE;
use crate::wave_tables::TRIANGLE_WAVE;

use crate::wave_tables::WAVE_TABLE_SIZE;
use crate::wave_tables::WAVE_TABLE_SIZE_U32;

const ALL_WAVE_TABLES: [&[i32; WAVE_TABLE_SIZE]; 4] =
    [&TRIANGLE_WAVE, &SAWTOOTH_WAVE, &SINE_WAVE, &SQUARE_WAVE];

/// Different Wave Types
///
#[derive(Clone, Copy, PartialEq, Debug)]
#[repr(usize)]
pub enum OscillatorType {
    Triangle,
    SawTooth,
    Sine,
    PulseWidth,
}

impl OscillatorType {
    const fn from_usize(usize_value: usize) -> Self {
        match usize_value {
            0 => Self::Triangle,
            1 => Self::SawTooth,
            2 => Self::Sine,
            3 => Self::PulseWidth,
            4_usize.. => todo!(),
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct SoundSourceOscillatorInit {
    pub frequency: u32,
}

impl SoundSourceOscillatorInit {
    pub fn new(frequency: u32) -> Self {
        return Self { frequency };
    }
}

pub struct CoreOscillator<
    const PLAY_FREQUENCY: u32,
    const PULSE_WIDTH: u8,
    const VOLUME_U8: u8,
    const OSCILLATOR_TYPE: usize,
> {
    pub table_idx: u32,
    pub table_remainder: u32,
    pub table_idx_inc: u32,
    pub table_remainder_inc: u32,
}

impl<
        const PLAY_FREQUENCY: u32,
        const PULSE_WIDTH: u8,
        const VOLUME: u8,
        const OSCILATOR_TYPE: usize,
    > Default for CoreOscillator<PLAY_FREQUENCY, PULSE_WIDTH, VOLUME, OSCILATOR_TYPE>
{
    fn default() -> Self {
        let table_idx_inc: u32 = 0;
        let table_remainder_inc: u32 = 0;
        let table_idx: u32 = 0;
        let table_remainder: u32 = Self::INC_DENOMINATOR / 2;
        Self {
            table_idx,
            table_remainder,
            table_idx_inc,
            table_remainder_inc,
        }
    }
}

impl<
        const PLAY_FREQUENCY: u32,
        const PULSE_WIDTH: u8,
        const VOLUME: u8,
        const OSCILATOR_TYPE: usize,
    > CoreOscillator<PLAY_FREQUENCY, PULSE_WIDTH, VOLUME, OSCILATOR_TYPE>
{
    const PULSE_WIDTH_CUTOFF: u32 = WAVE_TABLE_SIZE_U32 * (PULSE_WIDTH as u32) / 100;
    const VOLUME_SCALE: SoundSampleI32 = SoundSampleI32::new_percent(VOLUME);
    const INC_DENOMINATOR: u32 = FREQUENCY_MULTIPLIER * PLAY_FREQUENCY;
    const OSCILATOR_TYPE_ENUM: OscillatorType = OscillatorType::from_usize(OSCILATOR_TYPE);
    const PULSE_MAX: SoundSampleI32 = SoundSampleI32::MAX.const_mul(Self::VOLUME_SCALE);
    const PULSE_MIN: SoundSampleI32 = SoundSampleI32::MIN.const_mul(Self::VOLUME_SCALE);

    // Read sample from table that has the wave's amplitude values
    //
    // The basic idea of this function is that we're going through the table
    // at some rate (i.e., N entries per second, where N might be a fractional
    // value).  If we go through the table faster, we play the wave back at a
    // higher frequeny.  Slower gives a lower frequency.
    //
    // The rate that we're going through the table is represented by
    //
    // self_table_idx_inc + self.table_remainder_inc / inc_denominator
    //
    // Where 0 <= self.table_remainder_inc/ inc_demonimator < 1 is always true.
    //
    // The position in the table is tracked by self.table_idx, but there's always
    // some fractional left over value.  That value is tracked by
    // self.table_remainder, and has a "real" value of
    //
    // self.table_remainder / inc_denominator, which is always [0..1) when the
    // function exits.
    //

    fn get_next_table(&self, table: &[i32; WAVE_TABLE_SIZE]) -> SoundSampleI32 {
        SoundSampleI32::new_i32(table[self.table_idx as usize]) * Self::VOLUME_SCALE
    }

    fn get_next_pulse_entry(&self) -> SoundSampleI32 {
        if self.table_idx < Self::PULSE_WIDTH_CUTOFF {
            Self::PULSE_MAX
        } else {
            Self::PULSE_MIN
        }
    }
}

impl<
        const PLAY_FREQUENCY: u32,
        const PULSE_WIDTH: u8,
        const VOLUME: u8,
        const OSCILATOR_TYPE: usize,
    > SoundSourceCore<PLAY_FREQUENCY>
    for CoreOscillator<PLAY_FREQUENCY, PULSE_WIDTH, VOLUME, OSCILATOR_TYPE>
{
    type InitValuesType = SoundSourceOscillatorInit;

    fn init(self: &mut Self, init_values: &Self::InitValuesType) {
        let inc_numerator: u32 = init_values.frequency * WAVE_TABLE_SIZE_U32;

        self.table_idx = 0;
        self.table_remainder = Self::INC_DENOMINATOR / 2;
        self.table_idx_inc = inc_numerator / Self::INC_DENOMINATOR;
        self.table_remainder_inc = inc_numerator % Self::INC_DENOMINATOR;
    }

    fn has_next(self: &Self) -> bool {
        true
    }
    fn get_next(self: &mut Self) -> SoundSampleI32 {
        let rval = if Self::OSCILATOR_TYPE_ENUM == OscillatorType::PulseWidth {
            self.get_next_pulse_entry()
        } else {
            self.get_next_table(ALL_WAVE_TABLES[Self::OSCILATOR_TYPE_ENUM as usize])
        };

        // Update table position and fractional value
        //
        self.table_idx += self.table_idx_inc;
        self.table_remainder += self.table_remainder_inc;

        // If the fractional value represents a number greater than 1, increment
        // the table index and decease the fractional value so it's [0..1).
        //
        if self.table_remainder > Self::INC_DENOMINATOR {
            self.table_remainder -= Self::INC_DENOMINATOR;
            self.table_idx += 1;
        }
        self.table_idx = self.table_idx & (WAVE_TABLE_SIZE_U32 - 1);

        rval
    }
    fn trigger_note_off(self: &mut Self) {
        // does nothing.
    }
}

#[cfg(test)]
mod tests {
    use crate::oscillator::*;

    fn abs_sample(sample: i32) -> u32 {
        if sample >= 0 {
            sample as u32
        } else {
            (-sample) as u32
        }
    }

    fn sample_core_wave<'a, T>(oscilator: &mut T) -> (u32, u32)
    where
        T: SoundSourceCore<24000>,
    {
        let mut last = oscilator.get_next();
        let mut transitions: u32 = 0;
        let mut area: u32 = abs_sample(last.to_i32());
        for _ in 1..24000 {
            let current = oscilator.get_next();
            let last_above_0 = last.to_i32() > 0;
            let current_above_0 = current.to_i32() > 0;
            if last_above_0 != current_above_0 {
                transitions = transitions + 1;
            }
            area = area + abs_sample(current.to_i32());
            last = current;
        }
        (transitions, area)
    }

    #[test]
    fn test_pulse_50_from_pool() {
        let init_vals = SoundSourceOscillatorInit::new(2600 * FREQUENCY_MULTIPLIER);
        let mut oscilator =
            CoreOscillator::<24000, 50, 100, { OscillatorType::PulseWidth as usize }>::default();
        oscilator.init(&init_vals);

        let (transitions, area) = sample_core_wave(&mut oscilator);
        assert_eq!(2600 * 2 - 1, transitions);

        assert_eq!(0x8000 * 24000, area);
    }

    #[test]
    fn test_pulse_50_vol_50_from_pool() {
        let init_vals = SoundSourceOscillatorInit::new(2600 * FREQUENCY_MULTIPLIER);
        let mut oscilator =
            CoreOscillator::<24000, 50, 50, { OscillatorType::PulseWidth as usize }>::default();
        oscilator.init(&init_vals);

        let (transitions, area) = sample_core_wave(&mut oscilator);

        assert_eq!(2600 * 2 - 1, transitions);
        assert_eq!(0x4000 * 12000 + 0x4000 * 12000, area);
    }

    #[test]
    fn test_pulse_25_from_pool() {
        let init_vals = SoundSourceOscillatorInit::new(2600 * FREQUENCY_MULTIPLIER);
        let mut oscilator =
            CoreOscillator::<24000, 25, 100, { OscillatorType::PulseWidth as usize }>::default();
        oscilator.init(&init_vals);

        let (transitions, area) = sample_core_wave(&mut oscilator);

        assert_eq!(2600 * 2 - 1, transitions); // we don't get the last transition in square.
        assert_eq!(0x8000 * 24000, area); // TODO - this is kind of uninteresting.
    }

    #[test]
    fn test_triangle_from_pool() {
        let init_vals = SoundSourceOscillatorInit::new(2600 * FREQUENCY_MULTIPLIER);
        let mut oscilator =
            CoreOscillator::<24000, 0, 100, { OscillatorType::Triangle as usize }>::default();
        oscilator.init(&init_vals);

        let (transitions, area) = sample_core_wave(&mut oscilator);
        assert_eq!(2600 * 2, transitions);

        // Triangles are half the area squares are.
        assert_eq!(24000 * 0x4000, area);
    }

    #[test]
    fn test_triangle_from_pool_vol_50percent() {
        let init_vals = SoundSourceOscillatorInit::new(2600 * FREQUENCY_MULTIPLIER);
        let mut oscilator =
            CoreOscillator::<24000, 0, 50, { OscillatorType::Triangle as usize }>::default();
        oscilator.init(&init_vals);

        let (transitions, area) = sample_core_wave(&mut oscilator);
        assert_eq!(transitions, 2600 * 2);

        // Triangles are half the area squares are.
        assert_eq!(24000 * 0x2000, area);
    }
}
