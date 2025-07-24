use crate::sound_sample::SoundSampleI32;
use crate::sound_source_id::SoundSourceId;
use crate::sound_source_id::SoundSourceType;

pub trait SoundSources<'a, const PLAY_FREQUENCY: u32> {
    fn alloc(self: &mut Self, sound_source_type: SoundSourceType) -> SoundSourceId;
    fn free(self: &mut Self, id: SoundSourceId);
    fn has_next(self: &Self, id: &SoundSourceId) -> bool;
    fn get_next(self: &mut Self, id: &SoundSourceId) -> SoundSampleI32;
}
