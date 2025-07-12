use crate::free_list::FreeList;
use crate::free_list::FreeListImpl;
use crate::sound_sample::SoundSample;
use crate::sound_source::SoundSource;
use crate::sound_source_msgs::SoundSourceMsg;
use crate::sound_source_msgs::SoundSourceMsgs;
use crate::sound_source_pool::SoundSourcePool;
use crate::sound_sources::SoundSources;
use core::marker::PhantomData;

pub struct GenericSoundPool<
    SAMPLE: SoundSample,
    const PLAY_FREQUENCY: u32,
    MySoundSource: SoundSource<SAMPLE, PLAY_FREQUENCY> + Default,
    const N: usize,
    const TYPE_ID: usize,
> {
    sound_source: [MySoundSource; N],
    free_list: FreeListImpl<N>,
    _marker: PhantomData<SAMPLE>,
}

impl<
        SAMPLE: SoundSample,
        const PLAY_FREQUENCY: u32,
        MySoundSource: SoundSource<SAMPLE, PLAY_FREQUENCY> + Default,
        const N: usize,
        const TYPE_ID: usize,
    > GenericSoundPool<SAMPLE, PLAY_FREQUENCY, MySoundSource, N, TYPE_ID>
{
    pub fn new() -> Self {
        let sound_source: [MySoundSource; N] = core::array::from_fn(|_i| MySoundSource::default());
        let free_list: FreeListImpl<N> = FreeListImpl::default();
        Self {
            sound_source,
            free_list,
            _marker: PhantomData {},
        }
    }
    pub fn get_pool_entry(self: &Self, id: usize) -> &MySoundSource {
        return &self.sound_source[id];
    }
}

impl<
        SAMPLE: SoundSample,
        const PLAY_FREQUENCY: u32,
        MySoundSource: SoundSource<SAMPLE, PLAY_FREQUENCY> + Default,
        const N: usize,
        const TYPE_ID: usize,
    > FreeList for GenericSoundPool<SAMPLE, PLAY_FREQUENCY, MySoundSource, N, TYPE_ID>
{
    fn alloc(self: &mut Self) -> usize {
        self.free_list.alloc()
    }
    fn free(self: &mut Self, item_to_free: usize) {
        self.free_list.free(item_to_free)
    }
    fn is_active(self: &Self, idx: usize) -> bool {
        self.free_list.is_active(idx)
    }
}

impl<
        SAMPLE: SoundSample,
        const PLAY_FREQUENCY: u32,
        MySoundSource: SoundSource<SAMPLE, PLAY_FREQUENCY> + Default,
        const N: usize,
        const TYPE_ID: usize,
    > SoundSourcePool<'_, SAMPLE, PLAY_FREQUENCY>
    for GenericSoundPool<SAMPLE, PLAY_FREQUENCY, MySoundSource, N, TYPE_ID>
{
    fn pool_has_next(
        self: &Self,
        element: usize,
        all_sources: &dyn SoundSources<SAMPLE, PLAY_FREQUENCY>,
    ) -> bool {
        self.sound_source[element].has_next(all_sources)
    }
    fn pool_get_next(
        self: &Self,
        element: usize,
        all_sources: &dyn SoundSources<SAMPLE, PLAY_FREQUENCY>,
    ) -> SAMPLE {
        self.sound_source[element].get_next(all_sources)
    }
    fn update(self: &mut Self, new_msgs: &mut SoundSourceMsgs) {
        for idx in 0..N {
            if self.free_list.is_active(idx) {
                self.sound_source[idx].update(new_msgs);
            }
        }
    }

    fn pool_handle_msg(self: &mut Self, msg: &SoundSourceMsg, new_msgs: &mut SoundSourceMsgs) {
        self.sound_source[msg.dest_id.expect("").id()].handle_msg(msg, new_msgs)
    }
    fn get_type_id(self: &Self) -> usize {
        TYPE_ID
    }
}
