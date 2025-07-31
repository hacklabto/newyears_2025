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

type ChoirOscillatorPair<const PLAY_FREQUENCY: u32> = DoubleOscillator<
    PLAY_FREQUENCY,
    CoreOscillator<PLAY_FREQUENCY, 15, 100, { OscillatorType::PulseWidth as usize }>,
    CoreOscillator<PLAY_FREQUENCY, 25, 100, { OscillatorType::PulseWidth as usize }>,
    false,
>;

type ChoirOscillatorLfo<const PLAY_FREQUENCY: u32> = LfoAmplitude<
    PLAY_FREQUENCY,
    ChoirOscillatorPair<PLAY_FREQUENCY>,
    { OscillatorType::Triangle as usize },
    { 24 * FREQUENCY_MULTIPLIER / 10 },
    10,
>;

type ChoirOscillatorAdsr<const PLAY_FREQUENCY: u32> = AmpMixerCore<
    PLAY_FREQUENCY,
    ChoirOscillatorLfo<PLAY_FREQUENCY>,
    CoreAdsr<PLAY_FREQUENCY, 320, 5000, 100, 930>,
>;

type ChoirFiltered<const PLAY_FREQUENCY: u32> =
    Filter<PLAY_FREQUENCY, ChoirOscillatorAdsr<PLAY_FREQUENCY>, 1000>;

///
/// Choir.  Now sort of a proof of concept.
///
pub struct Choir<const PLAY_FREQUENCY: u32> {
    core: ChoirFiltered<PLAY_FREQUENCY>,
}

impl<const PLAY_FREQUENCY: u32> SoundSourceCore<PLAY_FREQUENCY> for Choir<PLAY_FREQUENCY> {
    type InitValuesType = SoundSourceNoteInit;

    fn get_next(self: &mut Self) -> SoundSampleI32 {
        self.core.get_next()
    }

    fn has_next(self: &Self) -> bool {
        self.core.has_next()
    }

    fn new(init_values: Self::InitValuesType) -> Self {
        let frequency_1 = midi_note_to_freq(init_values.key);
        let frequency_2 = midi_note_to_freq(init_values.key - 24);
        let adsr_init = (init_values.velocity as i32) << 8;
        let core = ChoirFiltered::<PLAY_FREQUENCY>::new(((frequency_1, frequency_2), adsr_init));
        return Self { core };
    }

    fn trigger_note_off(self: &mut Self) {
        self.core.trigger_note_off();
    }
}
