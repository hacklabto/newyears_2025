use crate::sound_sample::SoundSampleI32;
use crate::sound_source::SoundSource;
use crate::sound_source_id::SoundSourceId;

///
/// Test Helper
///
#[allow(unused)]
pub struct Top<const PLAY_FREQUENCY: u32> {
    creation_id: Option<SoundSourceId>,
}

#[allow(unused)]
impl<const PLAY_FREQUENCY: u32> Default for Top<PLAY_FREQUENCY> {
    fn default() -> Self {
        Self { creation_id: None }
    }
}

#[allow(unused)]
impl<const PLAY_FREQUENCY: u32> Top<PLAY_FREQUENCY> {
    pub fn get_creation_id(self: &Self) -> Option<SoundSourceId> {
        self.creation_id
    }
}

#[allow(unused)]
impl<const PLAY_FREQUENCY: u32> SoundSource<'_, PLAY_FREQUENCY> for Top<PLAY_FREQUENCY> {
    fn get_next(self: &mut Self) -> SoundSampleI32 {
        SoundSampleI32::ZERO
    }

    fn has_next(self: &Self) -> bool {
        true
    }
}
