use crate::adsr::CoreAdsr;
use crate::adsr::SoundSourceAdsrInit;
use crate::amp_mixer::AmpMixerCore;
use crate::double_oscillator::DoubleOscillator;
use crate::filter::Filter;
use crate::midi_notes::midi_note_to_freq;
use crate::note::SoundSourceNoteInit;
use crate::oscillator::CoreOscillator;
use crate::oscillator::OscillatorType;
use crate::oscillator::SoundSourceOscillatorInit;
use crate::sound_sample::SoundSampleI32;
use crate::sound_source_core::SoundSourceCore;

type GuitarAcousticOscillatorPair<const PLAY_FREQUENCY: u32> = DoubleOscillator<
    PLAY_FREQUENCY,
    CoreOscillator<PLAY_FREQUENCY, 25, 100, { OscillatorType::PulseWidth as usize }>,
    CoreOscillator<PLAY_FREQUENCY, 10, 90, { OscillatorType::PulseWidth as usize }>,
    true,
>;

type GuitarAcousticOscillatorAdsr<const PLAY_FREQUENCY: u32> = AmpMixerCore<
    PLAY_FREQUENCY,
    GuitarAcousticOscillatorPair<PLAY_FREQUENCY>,
    CoreAdsr<PLAY_FREQUENCY, 0, 1700, 0, 1700>,
>;

type GuitarAcousticFiltered<const PLAY_FREQUENCY: u32> = Filter<
    PLAY_FREQUENCY,
    GuitarAcousticOscillatorAdsr<PLAY_FREQUENCY>,
    106278871,
    813063803,
    -919342675,
>;

///
/// GuitarAcoustic.  Now sort of a proof of concept.
///
pub struct GuitarAcoustic<const PLAY_FREQUENCY: u32> {
    core: GuitarAcousticFiltered<PLAY_FREQUENCY>,
}

impl<const PLAY_FREQUENCY: u32> Default for GuitarAcoustic<PLAY_FREQUENCY> {
    fn default() -> Self {
        Self {
            core: GuitarAcousticFiltered::<PLAY_FREQUENCY>::default(),
        }
    }
}

impl<const PLAY_FREQUENCY: u32> SoundSourceCore<PLAY_FREQUENCY> for GuitarAcoustic<PLAY_FREQUENCY> {
    type InitValuesType = SoundSourceNoteInit;

    fn get_next(self: &mut Self) -> SoundSampleI32 {
        self.core.get_next()
    }

    fn has_next(self: &Self) -> bool {
        self.core.has_next()
    }

    fn new(init_values: &Self::InitValuesType) -> Self {
        let frequency_1 = midi_note_to_freq(init_values.key);
        let frequency_2 = midi_note_to_freq(init_values.key + 10);
        let oscillator_init_1 = SoundSourceOscillatorInit::new(frequency_1);
        let oscillator_init_2 = SoundSourceOscillatorInit::new(frequency_2);
        let adsr_init = SoundSourceAdsrInit::new();
        let core = GuitarAcousticFiltered::<PLAY_FREQUENCY>::new(&(
            (oscillator_init_1, oscillator_init_2),
            adsr_init,
        ));
        return Self { core };
    }

    fn trigger_note_off(self: &mut Self) {
        self.core.trigger_note_off();
    }
}
