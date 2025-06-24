use crate::sound_sample::SoundSample;
use crate::sound_sample::SoundSampleI32;
use crate::sound_source::GenericSoundPool;
use crate::sound_source::SoundSource;
use crate::sound_source::SoundSourceId;
use crate::sound_source::SoundSourceType;
use core::marker::PhantomData;

/// Start with just square waves
///
#[allow(unused)]
pub enum WaveType {
    Square,
}

///
/// Wave source generic for a sample type and frequency
///
#[allow(unused)]
struct GenericWaveSource<T: SoundSample, const PLAY_FREQUENCY: u32> {
    wave_type: WaveType,
    sound_frequency: u32,
    count: u32,
    _marker: PhantomData<T>,
}
//impl<T: SoundSample, const PLAY_FREQUENCY: u32> Drop for GenericWaveSource<T, PLAY_FREQUENCY> {
//    fn drop(&mut self) {}
//}

#[allow(unused)]
impl<T: SoundSample, const PLAY_FREQUENCY: u32> Default for GenericWaveSource<T, PLAY_FREQUENCY> {
    fn default() -> Self {
        let wave_type = WaveType::Square;
        let count: u32 = 0;
        let sound_frequency: u32 = 0;
        Self {
            wave_type,
            sound_frequency,
            count,
            _marker: PhantomData {},
        }
    }
}

#[allow(unused)]
impl<T: SoundSample, const PLAY_FREQUENCY: u32> GenericWaveSource<T, PLAY_FREQUENCY> {
    pub fn init(self: &mut Self, wave_type: WaveType, arg_sound_frequency: u32) {
        let count: u32 = 0;
        let sound_frequency = arg_sound_frequency * 1000;
        *self = Self {
            wave_type,
            sound_frequency,
            count,
            _marker: PhantomData {},
        }
    }

    // Implement a square wave generatoor using something like Bresenhan's line algorithm
    //
    fn get_next_square(&mut self) -> T {
        self.count += self.sound_frequency;
        if self.count > PLAY_FREQUENCY * 1000 {
            self.count -= PLAY_FREQUENCY * 1000;
        }
        if self.count < PLAY_FREQUENCY * 500 {
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
    use crate::sound_source::SoundSources;

    #[test]
    fn test_square() {
        let mut wave_source = WaveSource::default();
        wave_source.init(WaveType::Square, 2600);
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
        all_pools.free( wave_id );
    }
}
