use crate::free_list::FreeList;
use crate::sound_sample::SoundSample;
use crate::sound_source::SoundSourceAttributes;
use crate::sound_source::SoundSourceId;
use crate::sound_source::SoundSourceType;

#[allow(unused)]
pub trait SoundSourcePool<'a, SAMPLE: SoundSample, const PLAY_FREQUENCY: u32>: FreeList {
    // Functions that need to be filled in by implementor
    //
    fn pool_has_next(self: &Self, element: usize) -> bool;
    fn pool_get_next(self: &mut Self, element: usize) -> SAMPLE;
    fn pool_set_attribute(
        self: &mut Self,
        element: usize,
        key: SoundSourceAttributes,
        value: usize,
    );
    fn get_type_id(self: &Self) -> usize;

    fn pool_alloc(self: &mut Self) -> SoundSourceId {
        let pool_id = self.alloc();
        SoundSourceId::new(SoundSourceType::from_usize(self.get_type_id()), pool_id)
    }

    fn pool_free(self: &mut Self, id: SoundSourceId) {
        assert_eq!(self.get_type_id(), id.source_type as usize);
        self.free(id.id);
    }

    fn has_next(self: &mut Self, id: &SoundSourceId) -> bool {
        assert_eq!(self.get_type_id(), id.source_type as usize);
        self.pool_has_next(id.id)
    }

    fn get_next(self: &mut Self, id: &SoundSourceId) -> SAMPLE {
        assert_eq!(self.get_type_id(), id.source_type as usize);
        self.pool_get_next(id.id)
    }
    fn set_attribute(
        self: &mut Self,
        id: &SoundSourceId,
        key: SoundSourceAttributes,
        value: usize,
    ) {
        assert_eq!(self.get_type_id(), id.source_type as usize);
        self.pool_set_attribute(id.id, key, value)
    }
}
