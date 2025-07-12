use crate::adsr::GenericAdsr;
use crate::oscillator::GenericOscillator;
use crate::sound_sample::SoundSample;
use crate::sound_source_id::SoundSourceId;
use crate::sound_source_id::SoundSourceType;
use crate::sound_source_msgs::SoundSourceKey;
use crate::sound_source_msgs::SoundSourceMsg;
use crate::sound_source_msgs::SoundSourceMsgs;
use crate::sound_source_pool::SoundSourcePool;
use crate::sound_source_pool_impl::GenericSoundPool;
use crate::sound_sources::SoundSources;
use crate::top::Top;

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
    top_pool: GenericSoundPool<
        SAMPLE,
        PLAY_FREQUENCY,
        Top<SAMPLE, PLAY_FREQUENCY>,
        1,
        { SoundSourceType::Top as usize },
    >,
    top_id: SoundSourceId,
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
        let top_pool = GenericSoundPool::<
            SAMPLE,
            PLAY_FREQUENCY,
            Top<SAMPLE, PLAY_FREQUENCY>,
            1,
            { SoundSourceType::Top as usize },
        >::new();
        let top_id = Self::create_top_id();

        Self {
            oscillator_pool,
            adsr_pool,
            top_pool,
            top_id,
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
            SoundSourceType::Top => &mut self.top_pool,
        }
    }

    pub fn get_const_pool<'a>(
        self: &'a Self,
        sound_source_type: SoundSourceType,
    ) -> &'a dyn SoundSourcePool<'a, SAMPLE, PLAY_FREQUENCY> {
        match sound_source_type {
            SoundSourceType::Oscillator => &self.oscillator_pool,
            SoundSourceType::Adsr => &self.adsr_pool,
            SoundSourceType::Top => &self.top_pool,
        }
    }

    fn handle_msg(self: &mut Self, msg: &SoundSourceMsg, new_msgs: &mut SoundSourceMsgs) {
        return self
            .get_pool(msg.dest_id.expect("").source_type())
            .handle_msg(msg, new_msgs);
    }

    fn process_meta_message(
        self: &mut Self,
        msg: &SoundSourceMsg,
        new_msgs: &mut SoundSourceMsgs,
    ) -> bool {
        if msg.key == SoundSourceKey::InitOscillator {
            let oscillator_id = self.alloc(SoundSourceType::Oscillator);

            let oscilator_init_msg = SoundSourceMsg::new(
                Some(oscillator_id),
                self.get_top_id(),
                msg.key.clone(),
                msg.value.clone(),
            );
            new_msgs.append(oscilator_init_msg);
            true
        } else if msg.key == SoundSourceKey::InitAdsr {
            let adsr_id = self.alloc(SoundSourceType::Adsr);

            let adsr_init_msg = SoundSourceMsg::new(
                Some(adsr_id),
                self.get_top_id(),
                msg.key.clone(),
                msg.value.clone(),
            );
            new_msgs.append(adsr_init_msg);
            true
        } else {
            false
        }
    }

    fn process_and_clear_msgs_single_iter(
        self: &mut Self,
        msgs: &mut SoundSourceMsgs,
        new_msgs: &mut SoundSourceMsgs,
    ) {
        for msg in msgs.get_msgs() {
            let mut handled: bool = false;

            if msg.dest_id.is_none() || self.top_id == msg.dest_id.expect("") {
                handled = self.process_meta_message(&msg, new_msgs);
            }
            if !handled {
                self.handle_msg(&msg, new_msgs)
            }
        }
        msgs.clear();
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
        // Hopefully not a performance problem WRT to clearing on init
        let mut new_msgs = SoundSourceMsgs::default();

        loop {
            if msgs.get_msgs().len() == 0 {
                break;
            }
            self.process_and_clear_msgs_single_iter(msgs, &mut new_msgs);

            if new_msgs.get_msgs().len() == 0 {
                break;
            }
            self.process_and_clear_msgs_single_iter(&mut new_msgs, msgs);
        }
    }
    fn get_last_created_sound_source(self: &Self) -> Option<SoundSourceId> {
        return self.top_pool.get_pool_entry(0).get_creation_id();
    }
    fn get_top_id(self: &Self) -> SoundSourceId {
        return self.top_id.clone();
    }
}
impl<
        SAMPLE: SoundSample,
        const PLAY_FREQUENCY: u32,
        const NUM_OSCILATORS: usize,
        const NUM_ADSRS: usize,
    > SoundSourcesImpl<SAMPLE, PLAY_FREQUENCY, NUM_OSCILATORS, NUM_ADSRS>
{
    fn create_top_id() -> SoundSourceId {
        return SoundSourceId::new(SoundSourceType::Top, 0);
    }
}
