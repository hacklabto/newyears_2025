use crate::sound_sample::SoundSample;

/// Different tyoes source sources
///
pub enum SoundSourceType {
    WaveGenerator,
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
}

