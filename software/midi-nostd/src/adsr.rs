use crate::sound_sample::SoundSample;
use crate::sound_sample::SoundScale;
use crate::sound_source::SoundSource;
use crate::sound_source::SoundSourceAttributes;
use crate::sound_source::SoundSourceId;
//use crate::sound_source_pool_impl::GenericSoundPool;
use core::marker::PhantomData;

///
/// ADSR envelope
///
#[allow(unused)]
struct GenericWaveSource<T: SoundSample, const PLAY_FREQUENCY: u32> {
    attack_max_volume: SoundScale, // Reduction in volume after attack finishes
    sustain_volume: SoundScale,    // Reduction in volume during sustain phase
    a: u32,                        // timed, units are 1/PLAY_FREQUENCY
    d: u32,                        // timed, units are 1/PLAY_FREQUENCY
    r: u32,                        // timed, units are 1/PLAY_FREQUENCY
    child_sound: SoundSourceId,    // Source of the sound we're eveloping
    _marker: PhantomData<T>,
}

#[allow(unused)]
impl<T: SoundSample, const PLAY_FREQUENCY: u32> Default for GenericWaveSource<T, PLAY_FREQUENCY> {
    fn default() -> Self {
        let attack_max_volume = SoundScale::default();
        let a = PLAY_FREQUENCY / 8;
        let d = PLAY_FREQUENCY / 3;
        let sustain_volume = SoundScale::default();
        let r = PLAY_FREQUENCY / 5;
        let child_sound = SoundSourceId::default();

        Self {
            attack_max_volume,
            a,
            d,
            sustain_volume,
            r,
            child_sound,
            _marker: PhantomData {},
        }
    }
}

#[allow(unused)]
impl<T: SoundSample, const PLAY_FREQUENCY: u32> GenericWaveSource<T, PLAY_FREQUENCY> {
    //pub fn init(self: &mut Self, wave_type: WaveType, arg_sound_frequency: u32) {
    //}
}

#[allow(unused)]
impl<T: SoundSample, const PLAY_FREQUENCY: u32> SoundSource<T, PLAY_FREQUENCY>
    for GenericWaveSource<T, PLAY_FREQUENCY>
{
    fn get_next(&mut self) -> T {
        T::max()
    }

    fn has_next(&self) -> bool {
        true
    }
    fn set_attribute(&mut self, key: SoundSourceAttributes, value: usize) {}

    fn peer_sound_source(self: &Self) -> Option<SoundSourceId> {
        None
    }

    fn child_sound_source(self: &Self) -> Option<SoundSourceId> {
        None
    }
}
