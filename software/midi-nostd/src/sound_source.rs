use crate::sound_sample::SoundSample;
//use core::marker::PhantomData;

/// Different types source sources
///
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(usize)]
#[allow(unused)]
pub enum SoundSourceType {
    WaveGenerator = 0,
    AdsrEnvelope = 1,
}

#[allow(unused)]
impl SoundSourceType {
    pub fn from_usize(usize_value: usize) -> Self {
        let optional_enum_value: Option<Self> = match usize_value {
            0 => Some(SoundSourceType::WaveGenerator),
            1 => Some(SoundSourceType::AdsrEnvelope),
            _ => None,
        };
        optional_enum_value.expect("bad usize to SoundSourceType")
    }
    pub const fn all_variants() -> &'static [SoundSourceType] {
        &[
            SoundSourceType::WaveGenerator,
            SoundSourceType::AdsrEnvelope,
        ]
    }
    pub const fn max_variant_id() -> usize {
        let mut max_variant_id: Option<usize> = None;
        let slice = SoundSourceType::all_variants();
        let mut idx = 0;

        while idx < slice.len() {
            let enum_value = slice[idx];
            let usize_value = enum_value as usize;
            max_variant_id = if max_variant_id.is_none() {
                Some(usize_value)
            } else {
                if usize_value > max_variant_id.expect("") {
                    Some(usize_value)
                } else {
                    max_variant_id
                }
            };
            idx = idx + 1;
        }
        max_variant_id.expect("ENUM had no values!?!?")
    }
}

#[cfg(test)]
mod tests {
    use crate::sound_source::*;

    #[test]
    fn sound_source_enum_and_usize_bindings_are_consistent() {
        for enum_value in SoundSourceType::all_variants().iter().copied() {
            let usize_value = enum_value as usize;
            let enum_value_for_check = SoundSourceType::from_usize(usize_value);
            assert_eq!(enum_value, enum_value_for_check);
        }
    }

    #[test]
    // Each enum value should have a single usize map
    fn sound_source_enum_and_usize_bindings_are_sensible() {
        const MAX_ENUM_MAP: usize = SoundSourceType::max_variant_id() + 1;
        let mut times_seen: [u32; MAX_ENUM_MAP] = [0; MAX_ENUM_MAP];
        for enum_value in SoundSourceType::all_variants().iter().copied() {
            let usize_value = enum_value as usize;
            times_seen[usize_value] = times_seen[usize_value] + 1;
        }
        for times_element_was_seen in times_seen {
            assert_eq!(1, times_element_was_seen);
        }
    }
}

#[allow(unused)]
pub struct SoundSourceId {
    pub source_type: SoundSourceType,
    pub id: usize,
}

///
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[allow(unused)]
pub enum SoundSourceAttributes {
    WaveType,
    Frequiency,
    Volume
}

/// Start with just square waves
///
#[allow(unused)]
#[repr(usize)]
pub enum WaveType {
    Square = 0,
}


#[allow(unused)]
impl SoundSourceId {
    pub fn new(source_type: SoundSourceType, id: usize) -> Self {
        Self { source_type, id }
    }
}

///
/// Interface (so far) for a sound source  
///
/// A sound source is simply a source of sound.  The caller gets sound samples through
/// the get_next method.  This interface is abstract - an actual sound source may be
/// something like a waveform generator (i.e., sine or square waves) or may be something
/// more complicated
///
/// One idea is that we should be able to chain sound sources together.  For example,
/// a note might be created by  taking a waveform at the note's frequency and modifying
/// it using an ADSR amplitude envelope.
///
#[allow(unused)]
pub trait SoundSource<SAMPLE: SoundSample, const PLAY_FREQUENCY: u32> {
    /// Returns false if the sound source is done playing
    ///
    fn has_next(self: &Self) -> bool;

    /// Draw a sample from a source
    ///
    fn get_next(self: &mut Self) -> SAMPLE;

    /// Set Attribute
    fn set_attribute( key: SoundSourceAttributes, value: usize );

    fn peer_sound_source(self: &Self) -> Option<SoundSourceId>;
    fn child_sound_source(self: &Self) -> Option<SoundSourceId>;
}

#[allow(unused)]
pub trait SoundSourceFreeList {
    fn alloc(self: &mut Self) -> usize;
    fn free(self: &mut Self, item_to_free: usize);
}

#[allow(unused)]
pub struct SoundSourceFreeListImpl<const N: usize> {
    free_list: [Option<usize>; N],
    free_list_head: Option<usize>,
}

#[allow(unused)]
impl<const N: usize> SoundSourceFreeList for SoundSourceFreeListImpl<N> {
    fn alloc(self: &mut Self) -> usize {
        let allocated_item = self
            .free_list_head
            .expect("Unhandled out of sound pool error");
        self.free_list_head = self.free_list[allocated_item];
        self.free_list[allocated_item] = None;
        allocated_item
    }
    fn free(self: &mut Self, item_to_free: usize) {
        assert!(self.free_list[item_to_free].is_none());
        self.free_list[item_to_free] = self.free_list_head;
        self.free_list_head = Some(item_to_free);
    }
}

#[allow(unused)]
impl<const N: usize> SoundSourceFreeListImpl<N> {
    pub fn new() -> Self {
        let free_list: [Option<usize>; N] =
            core::array::from_fn(|idx| if idx == N - 1 { None } else { Some(idx + 1) });
        let free_list_head: Option<usize> = Some(0);

        Self {
            free_list,
            free_list_head,
        }
    }
}

#[cfg(test)]
mod free_list_tests {
    use crate::sound_source::*;
    #[test]
    fn free_list_should_alloc_and_free() {
        let mut free_list: SoundSourceFreeListImpl<3> = SoundSourceFreeListImpl::new();
        assert_eq!(0, free_list.alloc());
        assert_eq!(1, free_list.alloc());
        assert_eq!(2, free_list.alloc());
        free_list.free(1);
        free_list.free(0);
        free_list.free(2);
        assert_eq!(2, free_list.alloc());
        assert_eq!(0, free_list.alloc());
        assert_eq!(1, free_list.alloc());
    }
}

#[allow(unused)]
pub trait SoundSourcePool<'a, SAMPLE: SoundSample, const PLAY_FREQUENCY: u32>:
    SoundSourceFreeList
{
    // Functions that need to be filled in by implementor
    //
    fn pool_has_next(self: &Self, element: usize) -> bool;
    fn pool_get_next(self: &mut Self, element: usize) -> SAMPLE;
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
}

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
}

//_SAMPLE: PhantomData<SoundSample>,
#[allow(unused)]
pub struct GenericSoundPool<
    SAMPLE: SoundSample,
    const PLAY_FREQUENCY: u32,
    MySoundSource: SoundSource<SAMPLE, PLAY_FREQUENCY> + Default,
    const N: usize,
    const TYPE_ID: usize,
> {
    sound_source: [MySoundSource; N],
    free_list: SoundSourceFreeListImpl<N>,
    fake: SAMPLE, // TODO, spiral on phantom data
}

#[allow(unused)]
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
        let free_list: SoundSourceFreeListImpl<N> = SoundSourceFreeListImpl::new();
        let fake = SAMPLE::default();
        Self {
            sound_source,
            free_list,
            fake,
        }
    }
}

#[allow(unused)]
impl<
        SAMPLE: SoundSample,
        const PLAY_FREQUENCY: u32,
        MySoundSource: SoundSource<SAMPLE, PLAY_FREQUENCY> + Default,
        const N: usize,
        const TYPE_ID: usize,
    > SoundSourceFreeList for GenericSoundPool<SAMPLE, PLAY_FREQUENCY, MySoundSource, N, TYPE_ID>
{
    fn alloc(self: &mut Self) -> usize {
        self.free_list.alloc()
    }
    fn free(self: &mut Self, item_to_free: usize) {
        self.free_list.free(item_to_free)
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
    fn pool_has_next(self: &Self, element: usize) -> bool {
        self.sound_source[element].has_next()
    }
    fn pool_get_next(self: &mut Self, element: usize) -> SAMPLE {
        self.sound_source[element].get_next()
    }
    fn get_type_id(self: &Self) -> usize {
        TYPE_ID
    }
}
