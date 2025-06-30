use crate::sound_sample::SoundSample;
use crate::sound_source::SoundSource;
use crate::sound_source::SoundSourceAttributes;
use crate::sound_source::SoundSourceId;
//use crate::sound_source_pool_impl::GenericSoundPool;

///
/// ADSR envelope
///
#[allow(unused)]
struct GenericWaveSource<T: SoundSample, const PLAY_FREQUENCY: u32> {
    max_volume: T,     // The loudest the sound will get
    a: u32,            // timed, units are 1/PLAY_FREQUENCY
    d: u32,            // timed, units are 1/PLAY_FREQUENCY
    sustain_volume: T, // How loud to play in the sustain phase
    r: u32,            // timed, units are 1/PLAY_FREQUENCY
}

#[allow(unused)]
impl<T: SoundSample, const PLAY_FREQUENCY: u32> Default for GenericWaveSource<T, PLAY_FREQUENCY> {
    fn default() -> Self {
        let max_volume = T::max();
        let a = PLAY_FREQUENCY / 8;
        let d = PLAY_FREQUENCY / 3;
        let sustain_volume = T::max();
        let r = PLAY_FREQUENCY / 5;

        Self {
            max_volume,
            a,
            d,
            sustain_volume,
            r,
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
