use crate::sound_sample::SoundSample;
use crate::sound_source_id::SoundSourceId;
use crate::sound_source_id::SoundSourceType;
use crate::sound_source_msgs::SoundSourceKey;
use crate::sound_source_msgs::SoundSourceMsgs;
use crate::sound_source_msgs::SoundSourceValue;
use crate::sound_source_pool::SoundSourcePool;
use crate::sound_sources::SoundSources;

const MAX_ENUM_MAP: usize = SoundSourceType::max_variant_id() + 1;

pub struct SoundSourcesImpl<'a, SAMPLE: SoundSample, const PLAY_FREQUENCY: u32> {
    pools: [Option<&'a mut dyn SoundSourcePool<'a, SAMPLE, PLAY_FREQUENCY>>; MAX_ENUM_MAP],
}

impl<'a, SAMPLE: SoundSample, const PLAY_FREQUENCY: u32>
    SoundSourcesImpl<'a, SAMPLE, PLAY_FREQUENCY>
{
    pub fn create_with_single_pool_for_test(
        test_pool: &'a mut dyn SoundSourcePool<'a, SAMPLE, PLAY_FREQUENCY>,
        test_pool_slot: SoundSourceType,
    ) -> Self {
        let mut pools: [Option<&'a mut dyn SoundSourcePool<'a, SAMPLE, PLAY_FREQUENCY>>;
            MAX_ENUM_MAP] = core::array::from_fn(|_i| None);
        pools[test_pool_slot as usize] = Some(test_pool);
        Self { pools }
    }
    fn set_attribute(
        self: &mut Self,
        id: &SoundSourceId,
        key: SoundSourceKey,
        value: SoundSourceValue,
    ) {
        return self.pools[id.source_type() as usize]
            .as_mut()
            .expect("panic if none")
            .set_attribute(id, key, value);
    }
}

impl<'a, SAMPLE: SoundSample, const PLAY_FREQUENCY: u32> SoundSources<'_, SAMPLE, PLAY_FREQUENCY>
    for SoundSourcesImpl<'a, SAMPLE, PLAY_FREQUENCY>
{
    fn update(self: &mut Self, new_msgs: &mut SoundSourceMsgs) {
        for pool in &mut (self.pools) {
            if pool.is_some() {
                pool.as_mut().expect("it exists").update(new_msgs);
            }
        }
    }

    fn alloc(self: &mut Self, sound_source_type: SoundSourceType) -> SoundSourceId {
        self.pools[sound_source_type as usize]
            .as_mut()
            .expect("skill issue")
            .pool_alloc()
    }

    fn free(self: &mut Self, id: SoundSourceId) {
        self.pools[id.source_type() as usize]
            .as_mut()
            .expect("skill issue")
            .pool_free(id)
    }

    fn has_next(self: &Self, id: &SoundSourceId) -> bool {
        return self.pools[id.source_type() as usize]
            .as_ref()
            .expect("panic if none")
            .has_next(id, self);
    }
    fn get_next(self: &Self, id: &SoundSourceId) -> SAMPLE {
        return self.pools[id.source_type() as usize]
            .as_ref()
            .expect("panic if none")
            .get_next(id, self);
    }
    fn process_and_clear_msgs(self: &mut Self, msgs: &mut SoundSourceMsgs) {
        for msg in msgs.get_msgs() {
            self.set_attribute(&msg.dest_id, msg.attribute, msg.value.clone());
        }
        msgs.clear();
    }
}
