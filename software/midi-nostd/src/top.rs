use crate::sound_sample::SoundSample;
use crate::sound_source::SoundSource;
use crate::sound_source_id::SoundSourceId;
use crate::sound_source_msgs::SoundSourceMsg;
use crate::sound_source_msgs::SoundSourceMsgs;
use crate::sound_source_msgs::SoundSourceValue;
use crate::sound_sources::SoundSources;
use core::marker::PhantomData;

///
/// Test Helper
///
#[allow(unused)]
pub struct Top<T: SoundSample, const PLAY_FREQUENCY: u32> {
    creation_id: Option<SoundSourceId>,
    _marker: PhantomData<T>,
}

#[allow(unused)]
impl<T: SoundSample, const PLAY_FREQUENCY: u32> Default for Top<T, PLAY_FREQUENCY> {
    fn default() -> Self {
        Self {
            creation_id: None,
            _marker: PhantomData {},
        }
    }
}

#[allow(unused)]
impl<T: SoundSample, const PLAY_FREQUENCY: u32> Top<T, PLAY_FREQUENCY> {
    pub fn get_creation_id(self: &Self) -> Option<SoundSourceId> {
        self.creation_id
    }
}

#[allow(unused)]
impl<T: SoundSample, const PLAY_FREQUENCY: u32> SoundSource<'_, T, PLAY_FREQUENCY>
    for Top<T, PLAY_FREQUENCY>
{
    fn get_next(self: &Self, _all_sources: &dyn SoundSources<T, PLAY_FREQUENCY>) -> T {
        T::MIN
    }

    fn has_next(self: &Self, _all_sources: &dyn SoundSources<T, PLAY_FREQUENCY>) -> bool {
        true
    }

    fn update(&mut self, new_msgs: &mut SoundSourceMsgs) {}

    fn handle_msg(&mut self, msg: &SoundSourceMsg, new_msgs: &mut SoundSourceMsgs) {
        match &msg.value {
            SoundSourceValue::SoundSourceCreated {} => {
                self.creation_id = Some(msg.src_id.clone());
            }
            _ => todo!(),
        }
    }
}
