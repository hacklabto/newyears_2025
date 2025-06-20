use crate::sound_sample::SoundSample;

/// Different types source sources
///
#[derive(Clone,Copy,PartialEq,Eq,Debug)]
#[repr(usize)]
pub enum SoundSourceType {
    WaveGenerator = 0,
    AdsrEnvelope = 1
}

impl SoundSourceType {
    pub fn from_usize(usize_value: usize) -> Self
    {
        let optional_enum_value: Option<Self> = match usize_value {
            0  => Some(SoundSourceType::WaveGenerator),
            1  => Some(SoundSourceType::AdsrEnvelope),
            _  => None
        };
        optional_enum_value.expect("bad usize to SoundSourceType")
    }
    pub const fn all_variants() -> &'static [SoundSourceType] {
        &[
                SoundSourceType::WaveGenerator,
                SoundSourceType::AdsrEnvelope
        ]
    }
    pub const fn max_variant_id() -> usize {
        let mut max_variant_id: Option<usize> = None;
        let slice = SoundSourceType::all_variants();
        let mut idx = 0;

        while idx < slice.len() {
            let enum_value = slice[ idx ];
            let usize_value = enum_value as usize;
            max_variant_id = if max_variant_id.is_none() { 
                Some( usize_value )
            } else {
                if usize_value > max_variant_id.expect("") { 
                    Some(usize_value) 
                } 
                else {
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
            assert_eq!( enum_value, enum_value_for_check );
        }
    }

    #[test]
    // Each enum value should have a single usize map
    fn sound_source_enum_and_usize_bindings_are_sensible() {
        const MAX_ENUM_MAP:usize = SoundSourceType::max_variant_id()+1;
        let mut times_seen: [u32; MAX_ENUM_MAP] = [0; MAX_ENUM_MAP];
        for enum_value in SoundSourceType::all_variants().iter().copied() {
            let usize_value = enum_value as usize;
            times_seen[ usize_value ] = times_seen[ usize_value ] + 1;
        }
        for times_element_was_seen in times_seen {
            assert_eq!( 1, times_element_was_seen );
        }
    }
}

pub struct SoundSourceId {
    pub source_type: SoundSourceType,
    pub id: usize,
}

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
pub trait SoundSource<SAMPLE: SoundSample, const PLAY_FREQUENCY: u32> {
    fn has_next(self: &Self) -> bool;

    /// Draw a sample from a source
    ///
    fn get_next(self: &mut Self) -> SAMPLE;

    fn peer_sound_source(self: &Self)  -> Option<SoundSourceId>;
    fn child_sound_source(self: &Self) -> Option<SoundSourceId>;

    fn id(self: &Self) -> SoundSourceId;
}

pub struct SoundSourceFreeList<'a> {
    free_list: &'a mut[ Option<usize > ],
    free_list_head: Option<usize>
}

impl<'a> SoundSourceFreeList<'a> {
    pub fn alloc(self: &mut Self) -> usize {
        let allocatedItem = self.free_list_head.expect("Unhandled out of sound pool error");
        self.free_list_head = self.free_list[ allocatedItem ];
        self.free_list[ allocatedItem ] = None;
        allocatedItem
    }
    pub fn free(self: &mut Self, itemToFree: usize ) {
        assert!( self.free_list[ itemToFree ].is_none() );
        self.free_list[ itemToFree ] = self.free_list_head;
        self.free_list_head = Some( itemToFree );
    }
    pub fn new(free_list: &'a mut[ Option<usize>] ) -> SoundSourceFreeList<'a> {
        let free_list_head: Option<usize> = Some(0);

        for idx in 0..free_list.len()-1 {
            free_list[ idx ] = Some(idx+1);
        }
        free_list[ free_list.len()-1 ] = None;

        Self {
            free_list,
            free_list_head
        }
    }
}

#[cfg(test)]
mod free_list_tests {
    use crate::sound_source::*;
    #[test]
    fn free_list_should_alloc_and_free() {
        let mut storage : [Option<usize>; 3] = [None; 3];
        let mut free_list: SoundSourceFreeList = SoundSourceFreeList::new( &mut storage );
        assert_eq!( 0, free_list.alloc() );
        assert_eq!( 1, free_list.alloc() );
        assert_eq!( 2, free_list.alloc() );
        free_list.free(1);
        free_list.free(0);
        free_list.free(2);
        assert_eq!( 2, free_list.alloc() );
        assert_eq!( 0, free_list.alloc() );
        assert_eq!( 1, free_list.alloc() );
    }
}

pub trait SoundSourcePool<SAMPLE: SoundSample, const PLAY_FREQUENCY: u32>
{
    fn pool_alloc(self: &mut Self) -> usize;
    fn pool_has_next(self: &Self, element: usize) -> bool;
    fn pool_get_next(self: &mut Self, element: usize) -> SAMPLE;
    fn get_type_id(self: &Self) -> usize;

    fn alloc(self: &mut Self) -> SoundSourceId {
        let pool_id = self.pool_alloc();
        SoundSourceId::new(SoundSourceType::from_usize(self.get_type_id()), pool_id )
    }

    fn has_next(self: &mut Self, id: &SoundSourceId ) -> bool {
        assert_eq!( self.get_type_id(), id.source_type as usize);
        self.pool_has_next( id.id )
    }

    fn get_next(self: &mut Self, id: &SoundSourceId ) -> SAMPLE {
        assert_eq!( self.get_type_id(), id.source_type as usize);
        self.pool_get_next( id.id )
    }
}

const MAX_ENUM_MAP:usize = SoundSourceType::max_variant_id()+1;

pub struct SoundSources<'a, SAMPLE: SoundSample, const PLAY_FREQUENCY: u32>
{   
    pools: [&'a mut dyn SoundSourcePool<SAMPLE, PLAY_FREQUENCY>; MAX_ENUM_MAP ]
}

impl <'a, SAMPLE: SoundSample, const PLAY_FREQUENCY: u32> SoundSources<'a, SAMPLE, PLAY_FREQUENCY> 
{
    fn has_next(self: &mut Self, id: &SoundSourceId ) -> bool {
        return self.pools[id.source_type as usize].has_next(id);
    }
    fn get_next(self: &mut Self, id: &SoundSourceId ) -> SAMPLE {
        return self.pools[id.source_type as usize].get_next(id);
    }
}

