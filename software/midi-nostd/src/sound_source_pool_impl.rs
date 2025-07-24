use crate::free_list::FreeList;
use crate::free_list::FreeListImpl;
use crate::sound_sample::SoundSampleI32;
use crate::sound_source::SoundSource;
use crate::sound_source_msgs::SoundSourceMsg;
use crate::sound_source_msgs::SoundSourceMsgs;
use crate::sound_source_pool::SoundSourcePool;
use core::marker::PhantomData;

pub struct GenericSoundPool<
    'a,
    const PLAY_FREQUENCY: u32,
    MySoundSource: SoundSource<'a, PLAY_FREQUENCY> + Default,
    const N: usize,
    const TYPE_ID: usize,
> {
    sound_source: [MySoundSource; N],
    free_list: FreeListImpl<N>,
    _lifetime_marker: PhantomData<&'a ()>,
}

impl<
        'a,
        const PLAY_FREQUENCY: u32,
        MySoundSource: SoundSource<'a, PLAY_FREQUENCY> + Default,
        const N: usize,
        const TYPE_ID: usize,
    > GenericSoundPool<'a, PLAY_FREQUENCY, MySoundSource, N, TYPE_ID>
{
    pub fn new() -> Self {
        let sound_source: [MySoundSource; N] = core::array::from_fn(|_i| MySoundSource::default());
        let free_list: FreeListImpl<N> = FreeListImpl::default();
        Self {
            sound_source,
            free_list,
            _lifetime_marker: PhantomData {},
        }
    }
    pub fn get_pool_entry(self: &Self, id: usize) -> &MySoundSource {
        return &self.sound_source[id];
    }
}

impl<
        'a,
        const PLAY_FREQUENCY: u32,
        MySoundSource: SoundSource<'a, PLAY_FREQUENCY> + Default,
        const N: usize,
        const TYPE_ID: usize,
    > FreeList for GenericSoundPool<'a, PLAY_FREQUENCY, MySoundSource, N, TYPE_ID>
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
        'a,
        const PLAY_FREQUENCY: u32,
        MySoundSource: SoundSource<'a, PLAY_FREQUENCY> + Default,
        const N: usize,
        const TYPE_ID: usize,
    > SoundSourcePool<'a, PLAY_FREQUENCY>
    for GenericSoundPool<'a, PLAY_FREQUENCY, MySoundSource, N, TYPE_ID>
{
    fn pool_has_next(self: &Self, element: usize) -> bool {
        self.sound_source[element].has_next()
    }
    fn pool_get_next(self: &Self, element: usize) -> SoundSampleI32 {
        self.sound_source[element].get_next()
    }
    fn update(self: &mut Self, new_msgs: &mut SoundSourceMsgs) {
        for idx in 0..N {
            if self.free_list.is_active(idx) {
                self.sound_source[idx].update(new_msgs);
            }
        }
    }

    fn pool_handle_msg(self: &mut Self, msg: &SoundSourceMsg, new_msgs: &mut SoundSourceMsgs) {
        self.sound_source[msg.dest_id.id()].handle_msg(msg, new_msgs)
    }
    fn get_type_id(self: &Self) -> usize {
        TYPE_ID
    }
}
