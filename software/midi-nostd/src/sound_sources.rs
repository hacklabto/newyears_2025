use crate::sound_sample::SoundSample;
use crate::sound_source_id::SoundSourceId;
use crate::sound_source_id::SoundSourceType;
use crate::sound_source_msgs::SoundSourceMsgs;

pub trait SoundSources<'a, SAMPLE: SoundSample, const PLAY_FREQUENCY: u32> {
    fn update(self: &mut Self, new_msgs: &mut SoundSourceMsgs);
    fn alloc(self: &mut Self, sound_source_type: SoundSourceType) -> SoundSourceId;
    fn free(self: &mut Self, id: SoundSourceId);
    fn has_next(self: &Self, id: &SoundSourceId) -> bool;
    fn get_next(self: &Self, id: &SoundSourceId) -> SAMPLE;
    fn process_and_clear_msgs(self: &mut Self, msgs: &mut SoundSourceMsgs);
    fn get_top_id(self: &Self) -> SoundSourceId;
    fn get_last_created_sound_source(self: &Self) -> Option<SoundSourceId>;
}
