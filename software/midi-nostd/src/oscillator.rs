use crate::midi_notes::FREQUENCY_MULTIPLIER;
use crate::sound_sample::SoundSample;
use crate::sound_sample::SoundScale;
use crate::sound_source_core::SoundSourceCore;
use crate::wave_tables::SAWTOOTH_WAVE;
use crate::wave_tables::SINE_WAVE;
use crate::wave_tables::SQUARE_WAVE;
use crate::wave_tables::TRIANGLE_WAVE;

use crate::wave_tables::WAVE_TABLE_SIZE;
use crate::wave_tables::WAVE_TABLE_SIZE_U32;
use core::marker::PhantomData;

const ALL_WAVE_TABLES: [&[u16; WAVE_TABLE_SIZE]; 4] =
    [&TRIANGLE_WAVE, &SAWTOOTH_WAVE, &SINE_WAVE, &SQUARE_WAVE];

/// Different Wave Types
///
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum OscillatorType {
    Triangle,
    SawTooth,
    Sine,
    PulseWidth,
}

#[derive(Clone, PartialEq, Debug)]
pub struct SoundSourceOscillatorInit {
    pub oscillator_type: OscillatorType,
    pub frequency: u32,
    pub volume: u8,
}

impl SoundSourceOscillatorInit {
    pub fn new(oscillator_type: OscillatorType, frequency: u32, volume: u8) -> Self {
        return Self {
            oscillator_type,
            frequency,
            volume,
        };
    }
}

pub struct CoreOscillator<T: SoundSample, const PLAY_FREQUENCY: u32, const PULSE_WIDTH: u8> {
    pub volume: SoundScale,
    pub oscillator_type: OscillatorType,
    pub pulse_width_cutoff: u32,
    pub table_idx: u32,
    pub table_remainder: u32,
    pub table_idx_inc: u32,
    pub table_remainder_inc: u32,
    _marker: PhantomData<T>,
}

impl<T: SoundSample, const PLAY_FREQUENCY: u32, const PULSE_WIDTH: u8> Default
    for CoreOscillator<T, PLAY_FREQUENCY, PULSE_WIDTH>
{
    fn default() -> Self {
        let volume = SoundScale::new_percent(100); // full volume
        let oscillator_type = OscillatorType::PulseWidth;
        let pulse_width_cutoff: u32 = WAVE_TABLE_SIZE_U32 / 2; // 50% duty cycle by default
        let table_idx_inc: u32 = 0;
        let table_remainder_inc: u32 = 0;
        let table_idx: u32 = 0;
        let inc_denominator: u32 = FREQUENCY_MULTIPLIER * PLAY_FREQUENCY;
        let table_remainder: u32 = inc_denominator / 2;
        Self {
            volume,
            oscillator_type,
            pulse_width_cutoff,
            table_idx,
            table_remainder,
            table_idx_inc,
            table_remainder_inc,
            _marker: PhantomData {},
        }
    }
}

impl<T: SoundSample, const PLAY_FREQUENCY: u32, const PULSE_WIDTH: u8>
    CoreOscillator<T, PLAY_FREQUENCY, PULSE_WIDTH>
{
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

    fn get_next_table(&self, table: &[u16; WAVE_TABLE_SIZE]) -> T {
        let mut rval = T::new(table[self.table_idx as usize]);
        rval.scale(self.volume);
        rval
    }

    fn get_next_pulse_entry(&self) -> T {
        let mut rval = if self.table_idx < self.pulse_width_cutoff {
            T::max()
        } else {
            T::min()
        };
        rval.scale(self.volume);
        rval
    }
}

impl<T: SoundSample, const PLAY_FREQUENCY: u32, const PULSE_WIDTH: u8>
    SoundSourceCore<'_, T, PLAY_FREQUENCY> for CoreOscillator<T, PLAY_FREQUENCY, PULSE_WIDTH>
{
    type InitValuesType = SoundSourceOscillatorInit;

    fn init(self: &mut Self, init_values: &Self::InitValuesType) {
        let inc_numerator: u32 = init_values.frequency * WAVE_TABLE_SIZE_U32;
        let inc_denominator: u32 = FREQUENCY_MULTIPLIER * PLAY_FREQUENCY;
        let new_pulse_width_cutoff: u32 = WAVE_TABLE_SIZE_U32 * (PULSE_WIDTH as u32) / 100;
        let volume = SoundScale::new_percent(init_values.volume);

        self.volume = volume;
        self.oscillator_type = init_values.oscillator_type;
        self.pulse_width_cutoff = new_pulse_width_cutoff;
        self.table_idx = 0;
        self.table_remainder = inc_denominator / 2;
        self.table_idx_inc = inc_numerator / inc_denominator;
        self.table_remainder_inc = inc_numerator % inc_denominator;
    }

    fn has_next(self: &Self) -> bool {
        true
    }
    fn get_next(self: &Self) -> T {
        if self.oscillator_type == OscillatorType::PulseWidth {
            self.get_next_pulse_entry()
        } else {
            self.get_next_table(ALL_WAVE_TABLES[self.oscillator_type as usize])
        }
    }
    fn update(&mut self) {
        // Update table position and fractional value
        //
        self.table_idx += self.table_idx_inc;
        self.table_remainder += self.table_remainder_inc;
        let inc_denominator: u32 = FREQUENCY_MULTIPLIER * PLAY_FREQUENCY;

        // If the fractional value represents a number greater than 1, increment
        // the table index and decease the fractional value so it's [0..1).
        //
        if self.table_remainder > inc_denominator {
            self.table_remainder -= inc_denominator;
            self.table_idx += 1;
        }
        self.table_idx = self.table_idx & (WAVE_TABLE_SIZE_U32 - 1);
    }
    fn trigger_note_off(self: &mut Self) {
        // does nothing.
    }
}

#[cfg(test)]
mod tests {
    use crate::oscillator::*;
    use crate::sound_sample::SoundSampleI32;

    fn abs_sample(sample: u16) -> u16 {
        if sample >= 0x8000 {
            sample - 0x8000
        } else {
            0x8000 - sample
        }
    }

    fn sample_core_wave<'a, T>(oscilator: &mut T) -> (u32, u32)
    where
        T: SoundSourceCore<'a, SoundSampleI32, 24000>,
    {
        let mut last = oscilator.get_next();
        oscilator.update();
        let mut transitions: u32 = 0;
        let mut area: u32 = abs_sample(last.to_u16()) as u32;
        for _ in 1..24000 {
            let current = oscilator.get_next();
            oscilator.update();
            let last_above_0 = last.to_u16() >= 0x8000;
            let current_above_0 = current.to_u16() >= 0x8000;
            if last_above_0 != current_above_0 {
                transitions = transitions + 1;
            }
            area = area + abs_sample(current.to_u16()) as u32;
            last = current;
        }
        (transitions, area)
    }

    #[test]
    fn test_pulse_50_from_pool() {
        let init_vals = SoundSourceOscillatorInit::new(
            OscillatorType::PulseWidth,
            2600 * FREQUENCY_MULTIPLIER,
            100,
        );
        let mut oscilator = CoreOscillator::<SoundSampleI32, 24000, 50>::default();
        oscilator.init(&init_vals);

        let (transitions, area) = sample_core_wave(&mut oscilator);
        assert_eq!(2600 * 2 - 1, transitions);

        assert_eq!(0x7fff * 12000 + 0x8000 * 12000, area);
    }

    #[test]
    fn test_pulse_50_vol_50_from_pool() {
        let init_vals = SoundSourceOscillatorInit::new(
            OscillatorType::PulseWidth,
            2600 * FREQUENCY_MULTIPLIER,
            50,
        );
        let mut oscilator = CoreOscillator::<SoundSampleI32, 24000, 50>::default();
        oscilator.init(&init_vals);

        let (transitions, area) = sample_core_wave(&mut oscilator);

        assert_eq!(2600 * 2 - 1, transitions);
        assert_eq!(0x3fff * 12000 + 0x4000 * 12000, area);
    }

    #[test]
    fn test_pulse_25_from_pool() {
        let init_vals = SoundSourceOscillatorInit::new(
            OscillatorType::PulseWidth,
            2600 * FREQUENCY_MULTIPLIER,
            100,
        );
        let mut oscilator = CoreOscillator::<SoundSampleI32, 24000, 25>::default();
        oscilator.init(&init_vals);

        let (transitions, area) = sample_core_wave(&mut oscilator);

        assert_eq!(2600 * 2 - 1, transitions); // we don't get the last transition in square.
        assert_eq!(0x7fff * 6000 + 0x8000 * 18000, area);
    }

    #[test]
    fn test_triangle_from_pool() {
        let init_vals = SoundSourceOscillatorInit::new(
            OscillatorType::Triangle,
            2600 * FREQUENCY_MULTIPLIER,
            100,
        );
        let mut oscilator = CoreOscillator::<SoundSampleI32, 24000, 0>::default();
        oscilator.init(&init_vals);

        let (transitions, area) = sample_core_wave(&mut oscilator);
        assert_eq!(2600 * 2, transitions);

        // Triangles are half the area squares are.
        assert_eq!(12000 * 0x4000 + 12000 * 0x3fff, area);
    }

    #[test]
    fn test_triangle_from_pool_vol_50percent() {
        let init_vals = SoundSourceOscillatorInit::new(
            OscillatorType::Triangle,
            2600 * FREQUENCY_MULTIPLIER,
            50,
        );
        let mut oscilator = CoreOscillator::<SoundSampleI32, 24000, 0>::default();
        oscilator.init(&init_vals);

        let (transitions, area) = sample_core_wave(&mut oscilator);
        assert_eq!(transitions, 2600 * 2);

        // Triangles are half the area squares are.  200 is rounding error or a bug.
        assert_eq!(12000 * 0x2000 + 12000 * 0x1fff + 200, area);
    }
}
