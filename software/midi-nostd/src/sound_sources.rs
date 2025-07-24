use crate::sound_sample::SoundSampleI32;

pub trait SoundSources<'a, const PLAY_FREQUENCY: u32> {
    fn has_next(self: &Self) -> bool;
    fn get_next(self: &mut Self) -> SoundSampleI32;
}
