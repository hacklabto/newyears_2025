use crate::sound_sample::SoundSample;
use crate::sound_sample::SoundSampleI32;
use crate::sound_source::SoundSource;
use core::marker::PhantomData;

pub enum WaveType {
    Square,
    Triangle,
}

struct GenWaveSource<T: SoundSample, const PLAY_FREQUENCY: u32> {
    wave_type: WaveType,
    sound_frequency: u32,
    count: u32,
    _marker: PhantomData<T>,
}

impl<T: SoundSample, const PLAY_FREQUENCY: u32> GenWaveSource<T, PLAY_FREQUENCY> {
    pub fn new(wave_type: WaveType, sound_frequency: u32) -> Self {
        let count: u32 = 0;
        Self {
            wave_type,
            sound_frequency,
            count,
            _marker: PhantomData {},
        }
    }

    fn get_next_square(&mut self) -> T {
        self.count += self.sound_frequency;
        if self.count > PLAY_FREQUENCY {
            self.count -= PLAY_FREQUENCY;
        }
        if self.count < PLAY_FREQUENCY / 2 {
            T::min()
        } else {
            T::max()
        }
    }
}

impl<T: SoundSample, const PLAY_FREQUENCY: u32> SoundSource<T>
    for GenWaveSource<T, PLAY_FREQUENCY>
{
    fn get_next(&mut self) -> T {
        self.get_next_square()
    }
}

type WaveSource = GenWaveSource<SoundSampleI32, 24000>;

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
