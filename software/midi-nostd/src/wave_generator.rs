use crate::sound_sample::SoundSample;
use crate::sound_sample::SoundSampleI32;
use crate::sound_source::SoundSource;
use core::marker::PhantomData;

/// Start with just square waves
///
pub enum WaveType {
    Square,
}

///
/// Wave source generic for a sample type and frequency
///
struct GenericWaveSource<T: SoundSample, const PLAY_FREQUENCY: u32> {
    wave_type: WaveType,
    sound_frequency: u32,
    count: u32,
    _marker: PhantomData<T>,
}

impl<T: SoundSample, const PLAY_FREQUENCY: u32> GenericWaveSource<T, PLAY_FREQUENCY> {
    pub fn new(wave_type: WaveType, arg_sound_frequency: u32) -> Self {
        let count: u32 = 0;
        let sound_frequency = arg_sound_frequency * 1000;
        Self {
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

impl<T: SoundSample, const PLAY_FREQUENCY: u32> SoundSource<T>
    for GenericWaveSource<T, PLAY_FREQUENCY>
{
    fn get_next(&mut self) -> T {
        self.get_next_square()
    }
}

type WaveSource = GenericWaveSource<SoundSampleI32, 24000>;

#[cfg(test)]
mod tests {
    use crate::wave_generator::*;

    #[test]
    fn test_square() {
        let mut wave_source = WaveSource::new(WaveType::Square, 2600);
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
}
