use crate::sound_sample::SoundSample;
use crate::sound_source::SoundSource;
use crate::sound_source_id::SoundSourceId;
use crate::sound_source_msgs::SoundSourceKey;
use crate::sound_source_msgs::SoundSourceMsg;
use crate::sound_source_msgs::SoundSourceMsgs;
use crate::sound_sources::SoundSources;
use core::marker::PhantomData;

///
/// Test Helper
///
#[allow(unused)]
pub struct Top<T: SoundSample, const PLAY_FREQUENCY: u32> {
    creation_id: SoundSourceId,
    _marker: PhantomData<T>,
}

#[allow(unused)]
impl<T: SoundSample, const PLAY_FREQUENCY: u32> Default for Top<T, PLAY_FREQUENCY> {
    fn default() -> Self {
        let creation_id = SoundSourceId::default();

        Self {
            creation_id,
            _marker: PhantomData {},
        }
    }
}

#[allow(unused)]
impl<T: SoundSample, const PLAY_FREQUENCY: u32> Top<T, PLAY_FREQUENCY> {
    pub fn get_creation_id(self: &Self) -> SoundSourceId {
        self.creation_id
    }
}

#[allow(unused)]
impl<T: SoundSample, const PLAY_FREQUENCY: u32> SoundSource<T, PLAY_FREQUENCY>
    for Top<T, PLAY_FREQUENCY>
{
    fn get_next(self: &Self, _all_sources: &dyn SoundSources<T, PLAY_FREQUENCY>) -> T {
        T::min()
    }

    fn has_next(self: &Self, _all_sources: &dyn SoundSources<T, PLAY_FREQUENCY>) -> bool {
        true
    }

    fn update(&mut self, new_msgs: &mut SoundSourceMsgs) {}

    fn handle_msg(&mut self, msg: &SoundSourceMsg) {
        if msg.key == SoundSourceKey::SoundSourceCreated {
            //self.creation_id = origin.clone()
        }
    }
}
