use crate::sound_sample::SoundSample;

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
pub trait SoundSource<'s, SAMPLE: SoundSample> {
    /// Draw a sample from a source
    ///
    fn get_next(self: &mut Self) -> SAMPLE;

    /// What sound sources does this source depend on.
    /// TODO - use for clean-up when a note finishes playing and we want
    /// to recycle the resources it used.
    fn downstream_sources(self: &mut Self) -> Option<&'s [Option<&Self>]>;
}
