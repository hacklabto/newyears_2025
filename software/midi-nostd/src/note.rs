use crate::adsr::CoreAdsr;
use crate::adsr::SoundSourceAdsrInit;
use crate::amp_mixer::AmpMixerCore;
use crate::double_oscillator::DoubleOscillator;
use crate::midi_notes::midi_note_to_freq;
use crate::oscillator::CoreOscillator;
use crate::oscillator::OscillatorType;
use crate::oscillator::SoundSourceOscillatorInit;
use crate::sound_sample::SoundSampleI32;
use crate::sound_source_core::SoundSourceCore;

#[derive(Clone, PartialEq, Debug)]
pub struct SoundSourceNoteInit {
    pub key: u8,
    pub instrument: u8,
}

impl SoundSourceNoteInit {
    pub fn new(key: u8, instrument: u8) -> Self {
        return Self { key, instrument };
    }
}

type OscillatorPair<const PLAY_FREQUENCY: u32> = DoubleOscillator<
    PLAY_FREQUENCY,
    CoreOscillator<PLAY_FREQUENCY, 50, 75, { OscillatorType::SawTooth as usize }>,
    CoreOscillator<PLAY_FREQUENCY, 15, 100, { OscillatorType::PulseWidth as usize }>,
    true,
>;

type OscilatorAdsrCore<const PLAY_FREQUENCY: u32> = AmpMixerCore<
    PLAY_FREQUENCY,
    OscillatorPair<PLAY_FREQUENCY>,
    CoreAdsr<PLAY_FREQUENCY, 0, 670, 500, 100, 25>,
>;

///
/// Note.  Now sort of a proof of concept.
///
pub struct Note<const PLAY_FREQUENCY: u32> {
    core: OscilatorAdsrCore<PLAY_FREQUENCY>,
}

impl<const PLAY_FREQUENCY: u32> Default for Note<PLAY_FREQUENCY> {
    fn default() -> Self {
        Self {
            core: OscilatorAdsrCore::<PLAY_FREQUENCY>::default(),
        }
    }
}

impl<const PLAY_FREQUENCY: u32> SoundSourceCore<PLAY_FREQUENCY> for Note<PLAY_FREQUENCY> {
    type InitValuesType = SoundSourceNoteInit;

    fn get_next(self: &mut Self) -> SoundSampleI32 {
        self.core.get_next()
    }

    fn has_next(self: &Self) -> bool {
        self.core.has_next()
    }

    fn init(&mut self, init_values: &Self::InitValuesType) {
        let frequency_1 = midi_note_to_freq(init_values.key);
        let frequency_2 = midi_note_to_freq(init_values.key + 16);
        let oscillator_init_1 = SoundSourceOscillatorInit::new(frequency_1);
        let oscillator_init_2 = SoundSourceOscillatorInit::new(frequency_2);
        let adsr_init = SoundSourceAdsrInit::new();
        self.core
            .init(&((oscillator_init_1, oscillator_init_2), adsr_init));
    }

    fn trigger_note_off(self: &mut Self) {
        self.core.trigger_note_off();
    }
}
