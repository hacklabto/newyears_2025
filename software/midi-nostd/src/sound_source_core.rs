use crate::sound_sample::SoundSampleI32;

///
/// Interface (so far) for a "core" sound source
///
/// This is a source source that doesn't have any lower level dependencies.  So, an
/// oscilator could be considered a core sound source.  I'm building this because I'm
/// concerned about the amount of runtime the abstractions I was building as part of
/// a modular syntheisizer are going to use.
///
pub trait SoundSourceCore<const P_FREQ: u32, const U_FREQ: u32> {
    /// Initialize to a none default state.

    type InitValuesType;
    fn new(init_values: Self::InitValuesType) -> Self;

    /// Returns false if the sound source is done playing
    ///
    fn has_next(self: &Self) -> bool;

    /// Draw a sample from a source
    ///
    fn get_next(self: &mut Self) -> SoundSampleI32;

    /// Effective tells the source source to wind down gracefully :)
    /// Note off is the midi term.
    ///
    fn trigger_note_off(self: &mut Self) {}

    /// Reset an oscillator
    ///
    fn reset_oscillator(self: &mut Self) {}
}
