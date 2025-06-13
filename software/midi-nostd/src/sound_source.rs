use crate::sound_sample::SoundSample;

/// Different types source sources
///
#[derive(Clone,Copy,PartialEq,Eq,Debug)]
#[repr(i32)]
pub enum SoundSourceType {
    WaveGenerator = 0
}

impl SoundSourceType {
    pub fn from_i32(i32_value: i32 ) -> Self
    {
        let optional_enum_value: Option<Self> = match i32_value {
            0  => Some(SoundSourceType::WaveGenerator),
            _  => None
        };
        optional_enum_value.expect("bad i32 to SoundSourceType")
    }
    pub fn all_variants() -> &'static [SoundSourceType] {
        &[
                SoundSourceType::WaveGenerator
        ]
    }
}

#[cfg(test)]
mod tests {
    use crate::sound_source::*;

    #[test]
    fn sound_source_enum_and_i32_bindings_are_consistent() {
        for enum_value in SoundSourceType::all_variants().iter().copied() {
            let i32_value = enum_value as i32;
            let enum_value_for_check = SoundSourceType::from_i32(i32_value);
            assert_eq!( enum_value, enum_value_for_check );
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

type Deleter = dyn Fn(&SoundSourceId);

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
pub trait SoundSource<'s, SAMPLE: SoundSample, const PLAY_FREQUENCY: u32> {
    fn has_next(self: &Self) -> bool;

    /// Draw a sample from a source
    ///
    fn get_next(self: &mut Self) -> SAMPLE;

    /// What sound sources does this source depend on.
    /// TODO - use for clean-up when a note finishes playing and we want
    /// to recycle the resources it used.
    fn downstream_sources(self: &Self) -> Option<&'s [Option<SoundSourceId>]>;

    fn id(self: &Self) -> SoundSourceId;

    fn delete(self: &Self, delete_fn: &Deleter) {
        let downstream_maybe = self.downstream_sources();
        if downstream_maybe.is_some() {
            let downstream = downstream_maybe.unwrap();
            for id_maybe in downstream {
                if id_maybe.is_some() {
                    let id = id_maybe.as_ref().unwrap();
                    delete_fn(id);
                }
            }
        }
        delete_fn(&(self.id()));
    }
}

pub trait SoundSourcePool<'s, SAMPLE: SoundSample, const PLAY_FREQUENCY: u32, const TYPE: i32, const POOL_SIZE: usize >
{
    fn pool_alloc(self: &mut Self) -> usize;

    fn alloc(self: &mut Self) -> SoundSourceId {
        let pool_id = self.pool_alloc();
        SoundSourceId::new(SoundSourceType::from_i32(TYPE), pool_id )
    }
}

