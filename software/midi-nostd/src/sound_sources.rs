use crate::sound_sample::SoundSample;
use crate::sound_source::SoundSourceAttributes;
use crate::sound_source::SoundSourceId;
use crate::sound_source::SoundSourceType;
use crate::sound_source_pool::SoundSourcePool;

#[allow(unused)]
const MAX_ENUM_MAP: usize = SoundSourceType::max_variant_id() + 1;

#[allow(unused)]
pub struct SoundSources<'a, SAMPLE: SoundSample, const PLAY_FREQUENCY: u32> {
    pools: [Option<&'a mut dyn SoundSourcePool<'a, SAMPLE, PLAY_FREQUENCY>>; MAX_ENUM_MAP],
}

#[allow(unused)]
impl<'a, SAMPLE: SoundSample, const PLAY_FREQUENCY: u32> SoundSources<'a, SAMPLE, PLAY_FREQUENCY> {
    pub fn create_with_single_pool_for_test(
        test_pool: &'a mut dyn SoundSourcePool<'a, SAMPLE, PLAY_FREQUENCY>,
        test_pool_slot: SoundSourceType ) -> Self
    {
        let mut pools: [Option<&'a mut dyn SoundSourcePool<'a, SAMPLE, PLAY_FREQUENCY>>; MAX_ENUM_MAP] =
            core::array::from_fn(|_i| None );
        pools[ test_pool_slot as usize] = Some(test_pool);
        Self{ pools }
    }

    pub fn alloc( self: &mut Self, sound_source_type: SoundSourceType ) -> SoundSourceId 
    {
        self.pools[sound_source_type as usize].as_mut().expect("skill issue").pool_alloc()
    }

    pub fn free( self: &mut Self, id: SoundSourceId)
    {
        self.pools[id.source_type as usize].as_mut().expect("skill issue").pool_free( id )
    }

    pub fn has_next(self: &mut Self, id: &SoundSourceId) -> bool {
        return self.pools[id.source_type as usize].as_mut().expect("panic if none").has_next(id);
    }
    pub fn get_next(self: &mut Self, id: &SoundSourceId) -> SAMPLE {
        return self.pools[id.source_type as usize].as_mut().expect("panic if none").get_next(id);
    }
    pub fn set_attribute (self: &mut Self, id: &SoundSourceId, key: SoundSourceAttributes, value: usize) {
        return self.pools[id.source_type as usize].as_mut().expect("panic if none").set_attribute(id, key, value );
    }
}

