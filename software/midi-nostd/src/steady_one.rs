use crate::note::SoundSourceNoteInit;
use crate::sound_sample::SoundSampleI32;
use crate::sound_source_core::OscillatorInterface;
use crate::sound_source_core::SoundSourceCore;

///
/// SteadyOne.  Used when I don't have an instrument.
///
pub struct SteadyOne<const P_FREQ: u32, const U_FREQ: u32> {
    steady_something: SoundSampleI32,
}

impl<const P_FREQ: u32, const U_FREQ: u32> SoundSourceCore<P_FREQ, U_FREQ>
    for SteadyOne<P_FREQ, U_FREQ>
{
    type InitValuesType = SoundSourceNoteInit;

    fn get_next(self: &mut Self) -> SoundSampleI32 {
        self.steady_something
    }

    fn update(self: &mut Self) {}

    fn has_next(self: &Self) -> bool {
        false
    }

    fn new(_init_values: Self::InitValuesType) -> Self {
        return Self {
            steady_something: SoundSampleI32::MAX,
        };
    }

    fn trigger_note_off(self: &mut Self) {}

    fn restart(self: &mut Self, _vel: u8) {}
}

impl<const P_FREQ: u32, const U_FREQ: u32> OscillatorInterface<P_FREQ, U_FREQ>
    for SteadyOne<P_FREQ, U_FREQ>
{
    fn set_amplitude_adjust(self: &mut Self, adjust: SoundSampleI32) {
        self.steady_something = adjust;
    }
}
