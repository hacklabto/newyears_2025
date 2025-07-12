use crate::free_list::FreeList;
use crate::sound_sample::SoundSample;
use crate::sound_source_id::SoundSourceId;
use crate::sound_source_id::SoundSourceType;
use crate::sound_source_msgs::SoundSourceMsg;
use crate::sound_source_msgs::SoundSourceMsgs;
use crate::sound_sources::SoundSources;

pub trait SoundSourcePool<'a, SAMPLE: SoundSample, const PLAY_FREQUENCY: u32>: FreeList {
    // Functions that need to be filled in by implementor
    //
    fn pool_has_next(
        self: &Self,
        element: usize,
        all_sources: &dyn SoundSources<SAMPLE, PLAY_FREQUENCY>,
    ) -> bool;
    fn pool_get_next(
        self: &Self,
        element: usize,
        all_sources: &dyn SoundSources<SAMPLE, PLAY_FREQUENCY>,
    ) -> SAMPLE;
    fn pool_handle_msg(self: &mut Self, msg: &SoundSourceMsg, new_msgs: &mut SoundSourceMsgs);
    fn get_type_id(self: &Self) -> usize;

    fn pool_alloc(self: &mut Self) -> SoundSourceId {
        let pool_id = self.alloc();
        SoundSourceId::new(SoundSourceType::from_usize(self.get_type_id()), pool_id)
    }

    fn pool_free(self: &mut Self, id: SoundSourceId) {
        assert_eq!(self.get_type_id(), id.source_type() as usize);
        self.free(id.id());
    }

    fn has_next(
        self: &Self,
        id: &SoundSourceId,
        all_sources: &dyn SoundSources<SAMPLE, PLAY_FREQUENCY>,
    ) -> bool {
        assert_eq!(self.get_type_id(), id.source_type() as usize);
        self.pool_has_next(id.id(), all_sources)
    }

    fn get_next(
        self: &Self,
        id: &SoundSourceId,
        all_sources: &dyn SoundSources<SAMPLE, PLAY_FREQUENCY>,
    ) -> SAMPLE {
        assert_eq!(self.get_type_id(), id.source_type() as usize);
        self.pool_get_next(id.id(), all_sources)
    }

    fn update(self: &mut Self, new_msgs: &mut SoundSourceMsgs);

    fn handle_msg(self: &mut Self, msg: &SoundSourceMsg, new_msgs: &mut SoundSourceMsgs) {
        assert_eq!(
            self.get_type_id(),
            msg.dest_id.expect("").source_type() as usize
        );
        self.pool_handle_msg(msg, new_msgs);
    }
}
