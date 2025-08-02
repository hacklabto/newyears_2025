use crate::adsr::CoreAdsr;
use crate::amp_mixer::AmpMixerCore;
use crate::double_oscillator::DoubleOscillator;
use crate::filter::Filter;
use crate::midi_notes::midi_note_to_freq;
use crate::note::SoundSourceNoteInit;
use crate::oscillator::CoreOscillator;
use crate::oscillator::OscillatorType;
use crate::sound_sample::SoundSampleI32;
use crate::sound_source_core::SoundSourceCore;

type GuitarAcousticOscillatorPair<const P_FREQ: u32, const U_FREQ: u32> = DoubleOscillator<
    P_FREQ,
    U_FREQ,
    CoreOscillator<P_FREQ, U_FREQ, 25, 100, { OscillatorType::PulseWidth as usize }>,
    CoreOscillator<P_FREQ, U_FREQ, 10, 90, { OscillatorType::PulseWidth as usize }>,
    true,
>;

type GuitarAcousticOscillatorAdsr<const P_FREQ: u32, const U_FREQ: u32> = AmpMixerCore<
    P_FREQ,
    U_FREQ,
    GuitarAcousticOscillatorPair<P_FREQ, U_FREQ>,
    CoreAdsr<P_FREQ, U_FREQ, 0, 1700, 0, 1700>,
>;

type GuitarAcousticFiltered<const P_FREQ: u32, const U_FREQ: u32> =
    Filter<P_FREQ, U_FREQ, GuitarAcousticOscillatorAdsr<P_FREQ, U_FREQ>, 2000>;

///
/// GuitarAcoustic.  Now sort of a proof of concept.
///
pub struct GuitarAcoustic<const P_FREQ: u32, const U_FREQ: u32> {
    core: GuitarAcousticFiltered<P_FREQ, U_FREQ>,
}

impl<const P_FREQ: u32, const U_FREQ: u32> SoundSourceCore<P_FREQ, U_FREQ>
    for GuitarAcoustic<P_FREQ, U_FREQ>
{
    type InitValuesType = SoundSourceNoteInit;

    fn get_next(self: &mut Self) -> SoundSampleI32 {
        self.core.get_next()
    }

    fn has_next(self: &Self) -> bool {
        self.core.has_next()
    }

    fn new(init_values: Self::InitValuesType) -> Self {
        let frequency_1 = midi_note_to_freq(init_values.key);
        let frequency_2 = midi_note_to_freq(init_values.key + 10);
        let adsr_init = (init_values.velocity as i32) << 8;
        let core =
            GuitarAcousticFiltered::<P_FREQ, U_FREQ>::new(((frequency_1, frequency_2), adsr_init));
        return Self { core };
    }

    fn trigger_note_off(self: &mut Self) {
        self.core.trigger_note_off();
    }
}
