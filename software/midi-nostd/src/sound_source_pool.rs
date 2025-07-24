use crate::free_list::FreeList;
use crate::sound_sample::SoundSampleI32;
use crate::sound_source_id::SoundSourceId;
use crate::sound_source_id::SoundSourceType;
use crate::sound_source_msgs::SoundSourceMsg;
use crate::sound_source_msgs::SoundSourceMsgs;

pub trait SoundSourcePool<'a, const PLAY_FREQUENCY: u32>: FreeList {
    // Functions that need to be filled in by implementor
    //
    fn pool_has_next(self: &Self, element: usize) -> bool;
    fn pool_get_next(self: &mut Self, element: usize) -> SoundSampleI32;
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

    fn has_next(self: &Self, id: &SoundSourceId) -> bool {
        assert_eq!(self.get_type_id(), id.source_type() as usize);
        self.pool_has_next(id.id())
    }

    fn get_next(self: &mut Self, id: &SoundSourceId) -> SoundSampleI32 {
        assert_eq!(self.get_type_id(), id.source_type() as usize);
        self.pool_get_next(id.id())
    }

    fn handle_msg(self: &mut Self, msg: &SoundSourceMsg, new_msgs: &mut SoundSourceMsgs) {
        assert_eq!(self.get_type_id(), msg.dest_id.source_type() as usize);
        self.pool_handle_msg(msg, new_msgs);
    }
}
