use crate::sound_sample::SoundSample;
use crate::sound_sample::SoundSampleI32;
use crate::sound_source::SoundSource;
use crate::sound_source::SoundSourceId;
use crate::sound_source::SoundSourceType;
use crate::sound_source::WaveType;
use crate::sound_source::SoundSourceAttributes;
use crate::wave_tables::WAVE_TABLE_SIZE;
use crate::sound_source_pool_impl::GenericSoundPool;
use core::marker::PhantomData;

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
        // I want (arg_sound_frequency * WAVE_TABLE_SIZE) / (100 * PLAY_FREQUENCY);
        let inc_numerator: u32  = arg_sound_frequency * (WAVE_TABLE_SIZE as u32);
        let inc_denominator: u32 = (100 * PLAY_FREQUENCY );
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

    // Read sample from table that has wave amplitude values
    //
    // For now, sound_frequency is 100x the actual frequency, mostly because I
    // multplied the note frequencies by 100 when setting up the note to frequency
    // table.  If I also make the wave table 100 entries, then you can think of the
    // value in self.sound_frequency as table values per second.  i.e.,
    //
    // frequency = entire single wave tables / second
    // frequency *100 = entire single wave tables / second * 100
    // frequency *100 = (single wave table entry (for size 100 table) / 100)/ second * 100
    // frequency *100 = single wave table entry (for size 100 table) second 
    //
    // PLAY_FREQUENCY is the playback frequency, so
    //
    // frequency *100 /PLAY_FREQUENCY  = single wave table entry (size 100 ) second / PLAY_FREQUENY
    //
    // If I look at an A4 note, frequency * 100 is 44000.  If PLAY_FREQUENCY is 24000, it means
    // we go through one table entry every 24000/44000 updates, which is just over .5.
    //
    // If I look at A0, frequency * 100 is 2750.  24000/2750 is just under 9.  That's not
    // fantastic, in the sense that I could get better resolution with a bigger table.
    //
    // Next consideration - I probably want my table to be a power of 2, so I can loop
    // through the table my masking off the upper bits instead of doing a remainder, which
    // may be an expensive operation on some hardware.  So say WAVE_TABLE_SIZE.
    //
    // Ideally, I want my input frequency to be the target frequency * WAVE_TABLE_SIZE.
    //
    fn get_next_square(&mut self) -> T {
        self.table_idx += self.table_idx_inc;
        self.table_remainder += self.table_remainder_inc;
        let inc_denominator: u32 = (100 * PLAY_FREQUENCY );
        if self.table_remainder > inc_denominator {
            self.table_remainder -= inc_denominator;
            self.table_idx += 1;
        }
        self.table_idx = self.table_idx & (( WAVE_TABLE_SIZE as u32) -1 );
        if self.table_idx < 512 {
            T::min()
        } else {
            T::max()
        }
    }
}

#[allow(unused)]
impl<T: SoundSample, const PLAY_FREQUENCY: u32> SoundSource<T, PLAY_FREQUENCY>
    for GenericWaveSource<T, PLAY_FREQUENCY>
{
    fn get_next(&mut self) -> T {
        self.get_next_square()
    }

    fn has_next(&self) -> bool {
        true
    }
    fn set_attribute( &mut self, key: SoundSourceAttributes, value: usize ) {
        if key == SoundSourceAttributes::Frequency {
            let inc_numerator: u32  = (value as u32) * (WAVE_TABLE_SIZE as u32);
            let inc_denominator: u32 = (100 * PLAY_FREQUENCY );
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
    use crate::wave_generator::*;
    use crate::sound_sources::SoundSources;

    #[test]
    fn test_square() {
        let mut wave_source = WaveSource::default();
        wave_source.init(WaveType::Square, 2600*100);
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
        let mut all_pools = SoundSources::<SoundSampleI32, 24000 >::create_with_single_pool_for_test(
            &mut wave_pool,
            SoundSourceType::WaveGenerator ); 
        let wave_id = all_pools.alloc( SoundSourceType::WaveGenerator );
        all_pools.set_attribute( &wave_id, SoundSourceAttributes::Frequency, 2600 * 100 );

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
        all_pools.free( wave_id );
    }
}
