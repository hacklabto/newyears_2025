use crate::adsr::CoreAdsr;
use crate::amp_mixer::AmpMixerCore;
use crate::double_oscillator::DoubleOscillator;
use crate::filter::Filter;
use crate::lfo_amplitude::LfoAmplitude;
use crate::midi_notes::midi_note_to_freq;
use crate::midi_notes::FREQUENCY_MULTIPLIER;
use crate::note::SoundSourceNoteInit;
use crate::oscillator::CoreOscillator;
use crate::oscillator::OscillatorType;
use crate::sound_sample::SoundSampleI32;
use crate::sound_source_core::SoundSourceCore;

type SaxOscillatorPair<const P_FREQ: u32, const U_FREQ: u32> = DoubleOscillator<
    P_FREQ,
    U_FREQ,
    CoreOscillator<P_FREQ, U_FREQ, 30, 100, { OscillatorType::PulseWidth as usize }>,
    CoreOscillator<P_FREQ, U_FREQ, 45, 75, { OscillatorType::PulseWidth as usize }>,
    true,
>;

type SaxOscillatorLfo<const P_FREQ: u32, const U_FREQ: u32> = LfoAmplitude<
    P_FREQ,
    U_FREQ,
    SaxOscillatorPair<P_FREQ, U_FREQ>,
    { OscillatorType::Triangle as usize },
    { 15 * FREQUENCY_MULTIPLIER / 2 },
    10,
>;

type SaxOscillatorAdsr<const P_FREQ: u32, const U_FREQ: u32> = AmpMixerCore<
    P_FREQ,
    U_FREQ,
    SaxOscillatorLfo<P_FREQ, U_FREQ>,
    CoreAdsr<P_FREQ, U_FREQ, 0, 5000, 100, 300>,
>;

type SaxFiltered<const P_FREQ: u32, const U_FREQ: u32> =
    Filter<P_FREQ, U_FREQ, SaxOscillatorAdsr<P_FREQ, U_FREQ>, 2000>;

///
/// Sax.  Now sort of a proof of concept.
///
pub struct Sax<const P_FREQ: u32, const U_FREQ: u32> {
    core: SaxFiltered<P_FREQ, U_FREQ>,
}

impl<const P_FREQ: u32, const U_FREQ: u32> SoundSourceCore<P_FREQ, U_FREQ> for Sax<P_FREQ, U_FREQ> {
    type InitValuesType = SoundSourceNoteInit;

    fn get_next(self: &mut Self) -> SoundSampleI32 {
        self.core.get_next()
    }

    fn update(self: &mut Self) {
        self.core.update()
    }

    fn has_next(self: &Self) -> bool {
        self.core.has_next()
    }

    fn new(init_values: Self::InitValuesType) -> Self {
        let frequency_1 = midi_note_to_freq(init_values.key);
        let frequency_2 = midi_note_to_freq(init_values.key + 8);
        let adsr_init = (init_values.velocity as i32) << 8;
        let core = SaxFiltered::<P_FREQ, U_FREQ>::new(((frequency_1, frequency_2), adsr_init));
        return Self { core };
    }

    fn trigger_note_off(self: &mut Self) {
        self.core.trigger_note_off();
    }
}
