use crate::midi::Midi;
use crate::sound_sample::SoundSampleI32;
use crate::sound_source_id::SoundSourceId;
use crate::sound_source_id::SoundSourceType;
use crate::sound_source_msgs::SoundSourceMsg;
use crate::sound_source_msgs::SoundSourceMsgs;
use crate::sound_source_pool::SoundSourcePool;
use crate::sound_source_pool_impl::GenericSoundPool;
use crate::sound_sources::SoundSources;
use crate::top::Top;

//const MAX_ENUM_MAP: usize = SoundSourceType::max_variant_id() + 1;

pub struct SoundSourcesImpl<'a, const PLAY_FREQUENCY: u32, const NUM_NOTES: usize> {
    top_pool: GenericSoundPool<
        'a,
        PLAY_FREQUENCY,
        Top<PLAY_FREQUENCY>,
        1,
        { SoundSourceType::Top as usize },
    >,
    midi: GenericSoundPool<
        'a,
        PLAY_FREQUENCY,
        Midi<'a, PLAY_FREQUENCY>,
        1,
        { SoundSourceType::Midi as usize },
    >,
    top_id: SoundSourceId,
}

impl<'a, const PLAY_FREQUENCY: u32, const NUM_NOTES: usize> Default
    for SoundSourcesImpl<'a, PLAY_FREQUENCY, NUM_NOTES>
{
    fn default() -> Self {
        let mut top_pool = GenericSoundPool::<
            'a,
            PLAY_FREQUENCY,
            Top<PLAY_FREQUENCY>,
            1,
            { SoundSourceType::Top as usize },
        >::new();
        let midi = GenericSoundPool::<
            'a,
            PLAY_FREQUENCY,
            Midi<'a, PLAY_FREQUENCY>,
            1,
            { SoundSourceType::Midi as usize },
        >::new();

        top_pool.pool_alloc();

        let top_id = SoundSourceId::get_top_id();

        Self {
            top_pool,
            midi,
            top_id,
        }
    }
}

impl<'a, const PLAY_FREQUENCY: u32, const NUM_NOTES: usize>
    SoundSourcesImpl<'a, PLAY_FREQUENCY, NUM_NOTES>
{
    pub fn get_pool(
        self: &mut Self,
        sound_source_type: SoundSourceType,
    ) -> &mut dyn SoundSourcePool<'a, PLAY_FREQUENCY> {
        match sound_source_type {
            SoundSourceType::Top => &mut self.top_pool,
            SoundSourceType::Midi => &mut self.midi,
        }
    }

    pub fn get_const_pool(
        self: &Self,
        sound_source_type: SoundSourceType,
    ) -> &dyn SoundSourcePool<'a, PLAY_FREQUENCY> {
        match sound_source_type {
            SoundSourceType::Top => &self.top_pool,
            SoundSourceType::Midi => &self.midi,
        }
    }

    fn handle_msg(self: &mut Self, msg: &SoundSourceMsg, new_msgs: &mut SoundSourceMsgs) {
        return self
            .get_pool(msg.dest_id.source_type())
            .handle_msg(msg, new_msgs);
    }

    fn process_meta_message(
        self: &mut Self,
        msg: &SoundSourceMsg,
        _new_msgs: &mut SoundSourceMsgs,
    ) -> bool {
        match &msg.value {
            _ => false,
        }
    }

    fn process_and_clear_msgs_single_iter(
        self: &mut Self,
        msgs: &mut SoundSourceMsgs,
        new_msgs: &mut SoundSourceMsgs,
    ) {
        for msg in msgs.get_msgs() {
            let mut handled: bool = false;

            if self.top_id == msg.dest_id {
                handled = self.process_meta_message(&msg, new_msgs);
            }
            if !handled {
                self.handle_msg(&msg, new_msgs)
            }
        }
        msgs.clear();
    }
}

impl<const PLAY_FREQUENCY: u32, const NUM_NOTES: usize> SoundSources<'_, PLAY_FREQUENCY>
    for SoundSourcesImpl<'_, PLAY_FREQUENCY, NUM_NOTES>
{
    fn update(self: &mut Self, new_msgs: &mut SoundSourceMsgs) {
        self.midi.update(new_msgs);
        self.process_and_clear_msgs(new_msgs);
    }

    fn alloc(self: &mut Self, sound_source_type: SoundSourceType) -> SoundSourceId {
        self.get_pool(sound_source_type).pool_alloc()
    }

    fn free(self: &mut Self, id: SoundSourceId) {
        self.get_pool(id.source_type()).pool_free(id)
    }

    fn has_next(self: &Self, id: &SoundSourceId) -> bool {
        self.get_const_pool(id.source_type()).has_next(id)
    }
    fn get_next(self: &Self, id: &SoundSourceId) -> SoundSampleI32 {
        self.get_const_pool(id.source_type()).get_next(id)
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
}
