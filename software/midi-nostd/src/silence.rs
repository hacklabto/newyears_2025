use crate::note::SoundSourceNoteInit;
use crate::sound_sample::SoundSampleI32;
use crate::sound_source_core::SoundSourceCore;

///
/// Silence.  Used when I don't have an instrument.
///
pub struct Silence<const P_FREQ: u32, const U_FREQ: u32> {}

impl<const P_FREQ: u32, const U_FREQ: u32> SoundSourceCore<P_FREQ, U_FREQ>
    for Silence<P_FREQ, U_FREQ>
{
    type InitValuesType = SoundSourceNoteInit;

    fn get_next(self: &mut Self) -> SoundSampleI32 {
        SoundSampleI32::ZERO
    }

    fn has_next(self: &Self) -> bool {
        false
    }

    fn new(_init_values: Self::InitValuesType) -> Self {
        return Self {};
    }

    fn trigger_note_off(self: &mut Self) {}
}
