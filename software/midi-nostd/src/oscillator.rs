use crate::midi_notes::FREQUENCY_MULTIPLIER;
use crate::sound_sample::SoundSampleI32;
use crate::sound_source_core::OscillatorInterface;
use crate::sound_source_core::SoundSourceCore;
use crate::wave_tables::SAWTOOTH_WAVE;
use crate::wave_tables::SINE_WAVE;
use crate::wave_tables::SQUARE_WAVE;
use crate::wave_tables::TRIANGLE_WAVE;

use crate::wave_tables::WAVE_TABLE_SIZE;

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

pub struct CoreOscillator<
    const P_FREQ: u32,
    const U_FREQ: u32,
    const PULSE_WIDTH: u8,
    const VOLUME_U8: u8,
    const OSCILLATOR_TYPE: usize,
> {
    table_idx: u32,
    table_idx_inc: u32,
    max_amplitude: SoundSampleI32,
}

impl<
        const P_FREQ: u32,
        const U_FREQ: u32,
        const PULSE_WIDTH: u8,
        const VOLUME: u8,
        const OSCILATOR_TYPE: usize,
    > CoreOscillator<P_FREQ, U_FREQ, PULSE_WIDTH, VOLUME, OSCILATOR_TYPE>
{
    const PULSE_WIDTH_CUTOFF: u32 = ((1u64 << 32) * (PULSE_WIDTH as u64) / 100u64) as u32;
    const VOLUME_SCALE: SoundSampleI32 = SoundSampleI32::new_percent(VOLUME);
    const OSCILATOR_TYPE_ENUM: OscillatorType = OscillatorType::from_usize(OSCILATOR_TYPE);
    const INC_DENOMINATOR: u64 = (FREQUENCY_MULTIPLIER as u64) * (P_FREQ as u64);
}

impl<
        const P_FREQ: u32,
        const U_FREQ: u32,
        const PULSE_WIDTH: u8,
        const VOLUME: u8,
        const OSCILATOR_TYPE: usize,
    > SoundSourceCore<P_FREQ, U_FREQ>
    for CoreOscillator<P_FREQ, U_FREQ, PULSE_WIDTH, VOLUME, OSCILATOR_TYPE>
{
    type InitValuesType = u32;

    fn new(frequency: Self::InitValuesType) -> Self {
        Self {
            table_idx: 0,
            table_idx_inc: (((1u64 << 32) * (frequency as u64)) / Self::INC_DENOMINATOR) as u32,
            max_amplitude: Self::VOLUME_SCALE,
        }
    }

    #[inline]
    fn has_next(self: &Self) -> bool {
        true
    }
    #[inline]
    fn get_next(self: &mut Self) -> SoundSampleI32 {
        self.table_idx = self.table_idx.wrapping_add(self.table_idx_inc);

        if Self::OSCILATOR_TYPE_ENUM == OscillatorType::PulseWidth {
            if self.table_idx < Self::PULSE_WIDTH_CUTOFF {
                self.max_amplitude
            } else {
                SoundSampleI32::new_i32(-self.max_amplitude.to_i32()) // should implement neg
            }
        } else {
            let table = ALL_WAVE_TABLES[Self::OSCILATOR_TYPE_ENUM as usize];
            SoundSampleI32::new_i32(table[(self.table_idx >> 22) as usize]) * self.max_amplitude
        }
    }

    fn update(self: &mut Self) {}

    fn trigger_note_off(self: &mut Self) {
        // does nothing.
    }
    fn reset_oscillator(self: &mut Self) {
        self.table_idx = 0;
    }

    fn restart(self: &mut Self, _vel: u8) {}
}

impl<
        const P_FREQ: u32,
        const U_FREQ: u32,
        const PULSE_WIDTH: u8,
        const VOLUME: u8,
        const OSCILATOR_TYPE: usize,
    > OscillatorInterface<P_FREQ, U_FREQ>
    for CoreOscillator<P_FREQ, U_FREQ, PULSE_WIDTH, VOLUME, OSCILATOR_TYPE>
{
    fn set_amplitude_adjust(self: &mut Self, adjust: SoundSampleI32) {
        self.max_amplitude = Self::VOLUME_SCALE * adjust;
    }
    fn get_table_idx(self: &Self) -> u32 {
        return self.table_idx;
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
        T: SoundSourceCore<24000, 24000>,
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
        let mut oscilator =
            CoreOscillator::<24000, 24000, 50, 100, { OscillatorType::PulseWidth as usize }>::new(
                2600 * FREQUENCY_MULTIPLIER,
            );

        let (transitions, area) = sample_core_wave(&mut oscilator);
        assert_eq!(2600 * 2 - 1, transitions);

        assert_eq!(0x8000 * 24000, area);
    }

    #[test]
    fn test_pulse_50_vol_50_from_pool() {
        let mut oscilator =
            CoreOscillator::<24000, 24000, 50, 50, { OscillatorType::PulseWidth as usize }>::new(
                2600 * FREQUENCY_MULTIPLIER,
            );

        let (transitions, area) = sample_core_wave(&mut oscilator);

        assert_eq!(2600 * 2 - 1, transitions);
        assert_eq!(0x4000 * 12000 + 0x4000 * 12000, area);
    }

    #[test]
    fn test_pulse_25_from_pool() {
        let mut oscilator =
            CoreOscillator::<24000, 24000, 25, 100, { OscillatorType::PulseWidth as usize }>::new(
                2600 * FREQUENCY_MULTIPLIER,
            );

        let (transitions, area) = sample_core_wave(&mut oscilator);

        assert_eq!(2600 * 2 - 1, transitions); // we don't get the last transition in square.
        assert_eq!(0x8000 * 24000, area); // TODO - this is kind of uninteresting.
    }

    #[test]
    fn test_triangle_from_pool() {
        let mut oscilator =
            CoreOscillator::<24000, 24000, 0, 100, { OscillatorType::Triangle as usize }>::new(
                2600 * FREQUENCY_MULTIPLIER,
            );

        let (transitions, area) = sample_core_wave(&mut oscilator);
        assert_eq!(2600 * 2 - 1, transitions);

        // Triangles are half the area squares are.
        assert_eq!(24000 * 0x4000, area);
    }

    #[test]
    fn test_triangle_from_pool_vol_50percent() {
        let mut oscilator =
            CoreOscillator::<24000, 24000, 0, 50, { OscillatorType::Triangle as usize }>::new(
                2600 * FREQUENCY_MULTIPLIER,
            );

        let (transitions, area) = sample_core_wave(&mut oscilator);
        assert_eq!(2600 * 2 - 1, transitions);

        // Triangles are half the area squares are.
        assert_eq!(24000 * 0x2000, area);
    }
}
