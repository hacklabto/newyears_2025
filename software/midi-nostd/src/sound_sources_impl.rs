use crate::adsr::GenericAdsr;
use crate::oscillator::GenericOscillator;
use crate::sound_sample::SoundSample;
use crate::sound_source_id::SoundSourceId;
use crate::sound_source_id::SoundSourceType;
use crate::sound_source_msgs::SoundSourceKey;
use crate::sound_source_msgs::SoundSourceMsgs;
use crate::sound_source_msgs::SoundSourceValue;
use crate::sound_source_pool::SoundSourcePool;
use crate::sound_source_pool_impl::GenericSoundPool;
use crate::sound_sources::SoundSources;

//const MAX_ENUM_MAP: usize = SoundSourceType::max_variant_id() + 1;

pub struct SoundSourcesImpl<
    SAMPLE: SoundSample,
    const PLAY_FREQUENCY: u32,
    const NUM_OSCILATORS: usize,
    const NUM_ADSRS: usize,
> {
    oscillator_pool: GenericSoundPool<
        SAMPLE,
        PLAY_FREQUENCY,
        GenericOscillator<SAMPLE, PLAY_FREQUENCY>,
        NUM_OSCILATORS,
        { SoundSourceType::Oscillator as usize },
    >,
    adsr_pool: GenericSoundPool<
        SAMPLE,
        PLAY_FREQUENCY,
        GenericAdsr<SAMPLE, PLAY_FREQUENCY>,
        NUM_ADSRS,
        { SoundSourceType::Adsr as usize },
    >,
}

impl<
        SAMPLE: SoundSample,
        const PLAY_FREQUENCY: u32,
        const NUM_OSCILATORS: usize,
        const NUM_ADSRS: usize,
    > Default for SoundSourcesImpl<SAMPLE, PLAY_FREQUENCY, NUM_OSCILATORS, NUM_ADSRS>
{
    fn default() -> Self {
        let oscillator_pool = GenericSoundPool::<
            SAMPLE,
            PLAY_FREQUENCY,
            GenericOscillator<SAMPLE, PLAY_FREQUENCY>,
            NUM_OSCILATORS,
            { SoundSourceType::Oscillator as usize },
        >::new();
        let adsr_pool = GenericSoundPool::<
            SAMPLE,
            PLAY_FREQUENCY,
            GenericAdsr<SAMPLE, PLAY_FREQUENCY>,
            NUM_ADSRS,
            { SoundSourceType::Adsr as usize },
        >::new();
        Self {
            oscillator_pool,
            adsr_pool,
        }
    }
}

impl<
        SAMPLE: SoundSample,
        const PLAY_FREQUENCY: u32,
        const NUM_OSCILATORS: usize,
        const NUM_ADSRS: usize,
    > SoundSourcesImpl<SAMPLE, PLAY_FREQUENCY, NUM_OSCILATORS, NUM_ADSRS>
{
    pub fn get_pool<'a>(
        self: &'a mut Self,
        sound_source_type: SoundSourceType,
    ) -> &'a mut dyn SoundSourcePool<'a, SAMPLE, PLAY_FREQUENCY> {
        match sound_source_type {
            SoundSourceType::Oscillator => &mut self.oscillator_pool,
            SoundSourceType::Adsr => &mut self.adsr_pool,
        }
    }

    pub fn get_const_pool<'a>(
        self: &'a Self,
        sound_source_type: SoundSourceType,
    ) -> &'a dyn SoundSourcePool<'a, SAMPLE, PLAY_FREQUENCY> {
        match sound_source_type {
            SoundSourceType::Oscillator => &self.oscillator_pool,
            SoundSourceType::Adsr => &self.adsr_pool,
        }
    }

    fn set_attribute(
        self: &mut Self,
        id: SoundSourceId,
        key: SoundSourceKey,
        value: SoundSourceValue,
    ) {
        return self
            .get_pool(id.source_type())
            .set_attribute(id, key, value);
    }
}

impl<
        SAMPLE: SoundSample,
        const PLAY_FREQUENCY: u32,
        const NUM_OSCILATORS: usize,
        const NUM_ADSRS: usize,
    > SoundSources<'_, SAMPLE, PLAY_FREQUENCY>
    for SoundSourcesImpl<SAMPLE, PLAY_FREQUENCY, NUM_OSCILATORS, NUM_ADSRS>
{
    fn update(self: &mut Self, new_msgs: &mut SoundSourceMsgs) {
        self.oscillator_pool.update(new_msgs);
        self.adsr_pool.update(new_msgs);
    }

    fn alloc(self: &mut Self, sound_source_type: SoundSourceType) -> SoundSourceId {
        self.get_pool(sound_source_type).pool_alloc()
    }

    fn free(self: &mut Self, id: SoundSourceId) {
        self.get_pool(id.source_type()).pool_free(id)
    }

    fn has_next(self: &Self, id: &SoundSourceId) -> bool {
        self.get_const_pool(id.source_type()).has_next(id, self)
    }
    fn get_next(self: &Self, id: &SoundSourceId) -> SAMPLE {
        self.get_const_pool(id.source_type()).get_next(id, self)
    }
    fn process_and_clear_msgs(self: &mut Self, msgs: &mut SoundSourceMsgs) {
        for msg in msgs.get_msgs() {
            self.set_attribute(msg.dest_id.expect("todo"), msg.attribute, msg.value.clone());
        }
        msgs.clear();
    }
}
