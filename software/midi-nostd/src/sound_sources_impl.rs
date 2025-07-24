use crate::midi::Midi;
use crate::sound_sample::SoundSampleI32;
use crate::sound_source::SoundSource;
use crate::sound_sources::SoundSources;

pub struct SoundSourcesImpl<'a, const PLAY_FREQUENCY: u32, const NUM_NOTES: usize> {
    midi: Midi<'a, PLAY_FREQUENCY>,
}

impl<'a, const PLAY_FREQUENCY: u32, const NUM_NOTES: usize> Default
    for SoundSourcesImpl<'a, PLAY_FREQUENCY, NUM_NOTES>
{
    fn default() -> Self {
        let midi = Midi::<'a, PLAY_FREQUENCY>::default();
        Self { midi }
    }
}

impl<'a, const PLAY_FREQUENCY: u32, const NUM_NOTES: usize>
    SoundSourcesImpl<'a, PLAY_FREQUENCY, NUM_NOTES>
{
}

impl<const PLAY_FREQUENCY: u32, const NUM_NOTES: usize> SoundSources<'_, PLAY_FREQUENCY>
    for SoundSourcesImpl<'_, PLAY_FREQUENCY, NUM_NOTES>
{
    fn has_next(self: &Self) -> bool {
        self.midi.has_next()
    }
    fn get_next(self: &mut Self) -> SoundSampleI32 {
        self.midi.get_next()
    }
}
