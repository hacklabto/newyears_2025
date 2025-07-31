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

type SaxOscillatorPair<const PLAY_FREQUENCY: u32> = DoubleOscillator<
    PLAY_FREQUENCY,
    CoreOscillator<PLAY_FREQUENCY, 30, 100, { OscillatorType::PulseWidth as usize }>,
    CoreOscillator<PLAY_FREQUENCY, 45, 75, { OscillatorType::PulseWidth as usize }>,
    true,
>;

type SaxOscillatorLfo<const PLAY_FREQUENCY: u32> = LfoAmplitude<
    PLAY_FREQUENCY,
    SaxOscillatorPair<PLAY_FREQUENCY>,
    { OscillatorType::Triangle as usize },
    { 15 * FREQUENCY_MULTIPLIER / 2 },
    10,
>;

type SaxOscillatorAdsr<const PLAY_FREQUENCY: u32> = AmpMixerCore<
    PLAY_FREQUENCY,
    SaxOscillatorLfo<PLAY_FREQUENCY>,
    CoreAdsr<PLAY_FREQUENCY, 0, 5000, 100, 300>,
>;

type SaxFiltered<const PLAY_FREQUENCY: u32> =
    Filter<PLAY_FREQUENCY, SaxOscillatorAdsr<PLAY_FREQUENCY>, 2000>;

///
/// Sax.  Now sort of a proof of concept.
///
pub struct Sax<const PLAY_FREQUENCY: u32> {
    core: SaxFiltered<PLAY_FREQUENCY>,
}

impl<const PLAY_FREQUENCY: u32> SoundSourceCore<PLAY_FREQUENCY> for Sax<PLAY_FREQUENCY> {
    type InitValuesType = SoundSourceNoteInit;

    fn get_next(self: &mut Self) -> SoundSampleI32 {
        self.core.get_next()
    }

    fn has_next(self: &Self) -> bool {
        self.core.has_next()
    }

    fn new(init_values: Self::InitValuesType) -> Self {
        let frequency_1 = midi_note_to_freq(init_values.key);
        let frequency_2 = midi_note_to_freq(init_values.key + 8);
        let adsr_init = (init_values.velocity as i32) << 8;
        let core = SaxFiltered::<PLAY_FREQUENCY>::new(((frequency_1, frequency_2), adsr_init));
        return Self { core };
    }

    fn trigger_note_off(self: &mut Self) {
        self.core.trigger_note_off();
    }
}
