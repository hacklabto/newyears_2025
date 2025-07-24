use crate::sound_sample::SoundSampleI32;

///
/// Interface (so far) for a "core" sound source
///
/// This is a source source that doesn't have any lower level dependencies.  So, an
/// oscilator could be considered a core sound source.  I'm building this because I'm
/// concerned about the amount of runtime the abstractions I was building as part of
/// a modular syntheisizer are going to use.
///
pub trait SoundSourceCore<'a, const PLAY_FREQUENCY: u32> {
    /// Initialize to a none default state.

    type InitValuesType;
    fn init(&mut self, init_values: &Self::InitValuesType);

    /// Returns false if the sound source is done playing
    ///
    fn has_next(self: &Self) -> bool;

    /// Draw a sample from a source
    ///
    fn get_next(self: &Self) -> SoundSampleI32;

    /// Update the state one tick
    ///
    fn update(self: &mut Self);

    /// Effective tells the source source to wind down gracefully :)
    /// Note off is the midi term.
    ///
    fn trigger_note_off(self: &mut Self);
}
