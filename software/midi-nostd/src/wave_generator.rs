use crate::midi_notes::FREQUENCY_MULTIPLIER;
use crate::sound_sample::SoundSample;
use crate::sound_sample::SoundSampleI32;
use crate::sound_source::SoundSource;
use crate::sound_source::SoundSourceAttributes;
use crate::sound_source::SoundSourceId;
use crate::sound_source::SoundSourceType;
use crate::sound_source::WaveType;
use crate::sound_source_pool_impl::GenericSoundPool;
use crate::wave_tables::SAWTOOTH_WAVE;
use crate::wave_tables::SINE_WAVE;
use crate::wave_tables::SQUARE_WAVE;
use crate::wave_tables::TRIANGLE_WAVE;

use crate::wave_tables::WAVE_TABLE_SIZE;
use core::marker::PhantomData;

const ALL_WAVE_TABLES: [&[u16; WAVE_TABLE_SIZE]; 4] =
    [&SQUARE_WAVE, &TRIANGLE_WAVE, &SAWTOOTH_WAVE, &SINE_WAVE];

///
/// Wave source generic for a sample type and frequency
///
#[allow(unused)]
struct GenericWaveSource<T: SoundSample, const PLAY_FREQUENCY: u32> {
    wave_type: WaveType,
    table_idx: u32,
    table_remainder: u32,
    table_idx_inc: u32,
    table_remainder_inc: u32,
    _marker: PhantomData<T>,
}
//impl<T: SoundSample, const PLAY_FREQUENCY: u32> Drop for GenericWaveSource<T, PLAY_FREQUENCY> {
//    fn drop(&mut self) {}
//}

#[allow(unused)]
impl<T: SoundSample, const PLAY_FREQUENCY: u32> Default for GenericWaveSource<T, PLAY_FREQUENCY> {
    fn default() -> Self {
        let wave_type = WaveType::Square;
        let table_idx: u32 = 0;
        let table_remainder: u32 = 0;
        let table_idx_inc: u32 = 0;
        let table_remainder_inc: u32 = 0;
        Self {
            wave_type,
            table_idx,
            table_remainder,
            table_idx_inc,
            table_remainder_inc,
            _marker: PhantomData {},
        }
    }
}

#[allow(unused)]
impl<T: SoundSample, const PLAY_FREQUENCY: u32> GenericWaveSource<T, PLAY_FREQUENCY> {
    pub fn init(self: &mut Self, wave_type: WaveType, arg_sound_frequency: u32) {
        let table_idx: u32 = 0;
        let table_remainder: u32 = 0;
        // I want (arg_sound_frequency * WAVE_TABLE_SIZE) / (FREQUNCY_MULTIPLIER * PLAY_FREQUENCY);
        let inc_numerator: u32 = arg_sound_frequency * (WAVE_TABLE_SIZE as u32);
        let inc_denominator: u32 = (FREQUENCY_MULTIPLIER * PLAY_FREQUENCY);
        let table_idx_inc: u32 = inc_numerator / inc_denominator;
        let table_remainder_inc: u32 = inc_numerator % inc_denominator;
        *self = Self {
            wave_type,
            table_idx,
            table_remainder,
            table_idx_inc,
            table_remainder_inc,
            _marker: PhantomData {},
        }
    }

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
    fn get_next_table(&mut self, table: &[u16; WAVE_TABLE_SIZE]) -> T {
        // Update table position and fractional value
        //
        self.table_idx += self.table_idx_inc;
        self.table_remainder += self.table_remainder_inc;
        let inc_denominator: u32 = (FREQUENCY_MULTIPLIER * PLAY_FREQUENCY);

        // If the fractional value represents a number greater than 1, increment
        // the table index and decease the fractional value so it's [0..1).
        //
        if self.table_remainder > inc_denominator {
            self.table_remainder -= inc_denominator;
            self.table_idx += 1;
        }
        self.table_idx = self.table_idx & ((WAVE_TABLE_SIZE as u32) - 1);
        T::new(table[self.table_idx as usize])
    }
}

#[allow(unused)]
impl<T: SoundSample, const PLAY_FREQUENCY: u32> SoundSource<T, PLAY_FREQUENCY>
    for GenericWaveSource<T, PLAY_FREQUENCY>
{
    fn get_next(&mut self) -> T {
        self.get_next_table(ALL_WAVE_TABLES[self.wave_type as usize])
    }

    fn has_next(&self) -> bool {
        true
    }
    fn set_attribute(&mut self, key: SoundSourceAttributes, value: usize) {
        if key == SoundSourceAttributes::Frequency {
            let inc_numerator: u32 = (value as u32) * (WAVE_TABLE_SIZE as u32);
            let inc_denominator: u32 = (FREQUENCY_MULTIPLIER * PLAY_FREQUENCY);
            self.table_idx_inc = inc_numerator / inc_denominator;
            self.table_remainder_inc = inc_numerator % inc_denominator;
        }
    }

    fn peer_sound_source(self: &Self) -> Option<SoundSourceId> {
        None
    }

    fn child_sound_source(self: &Self) -> Option<SoundSourceId> {
        None
    }
}

#[allow(unused)]
type WaveSource = GenericWaveSource<SoundSampleI32, 24000>;
#[allow(unused)]
type WavePool = GenericSoundPool<
    SoundSampleI32,
    24000,
    WaveSource,
    3,
    { SoundSourceType::WaveGenerator as usize },
>;

#[cfg(test)]
mod tests {
    use crate::sound_sources::SoundSources;
    use crate::wave_generator::*;

    #[test]
    fn test_square() {
        let mut wave_source = WaveSource::default();
        wave_source.init(WaveType::Square, 2600 * FREQUENCY_MULTIPLIER);
        let mut last = wave_source.get_next();
        let mut transitions: u32 = 0;
        for _ in 0..24000 {
            let current = wave_source.get_next();
            if current != last {
                transitions = transitions + 1;
            }
            last = current;
        }
        assert_eq!(transitions, 2600 * 2);
    }

    #[test]
    fn test_square_from_pool() {
        let mut wave_pool: WavePool = WavePool::new();
        let mut all_pools = SoundSources::<SoundSampleI32, 24000>::create_with_single_pool_for_test(
            &mut wave_pool,
            SoundSourceType::WaveGenerator,
        );
        let wave_id = all_pools.alloc(SoundSourceType::WaveGenerator);
        all_pools.set_attribute(
            &wave_id,
            SoundSourceAttributes::Frequency,
            2600 * (FREQUENCY_MULTIPLIER as usize),
        );

        let mut last = all_pools.get_next(&wave_id);
        let mut transitions: u32 = 0;
        for _ in 0..24000 {
            let current = all_pools.get_next(&wave_id);
            if current != last {
                transitions = transitions + 1;
            }
            last = current;
        }
        assert_eq!(transitions, 2600 * 2);
        all_pools.free(wave_id);
    }
}
