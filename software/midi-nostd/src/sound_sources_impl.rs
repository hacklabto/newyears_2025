use crate::midi::Midi;
use crate::sound_sample::SoundSampleI32;
use crate::sound_source_id::SoundSourceId;
use crate::sound_source_id::SoundSourceType;
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

        Self { top_pool, midi }
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
}

impl<const PLAY_FREQUENCY: u32, const NUM_NOTES: usize> SoundSources<'_, PLAY_FREQUENCY>
    for SoundSourcesImpl<'_, PLAY_FREQUENCY, NUM_NOTES>
{
    fn alloc(self: &mut Self, sound_source_type: SoundSourceType) -> SoundSourceId {
        self.get_pool(sound_source_type).pool_alloc()
    }

    fn free(self: &mut Self, id: SoundSourceId) {
        self.get_pool(id.source_type()).pool_free(id)
    }

    fn has_next(self: &Self, id: &SoundSourceId) -> bool {
        self.get_const_pool(id.source_type()).has_next(id)
    }
    fn get_next(self: &mut Self, id: &SoundSourceId) -> SoundSampleI32 {
        let result = self.get_pool(id.source_type()).get_next(id);
        result
    }
}
