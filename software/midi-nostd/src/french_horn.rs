use crate::adsr::CoreAdsr;
use crate::amp_mixer::AmpMixerCore;
use crate::filter::Filter;
use crate::midi_notes::midi_note_to_freq;
use crate::note::SoundSourceNoteInit;
use crate::oscillator::CoreOscillator;
use crate::oscillator::OscillatorType;
use crate::sound_sample::SoundSampleI32;
use crate::sound_source_core::SoundSourceCore;

type FrenchHornOscillatorPair<const PLAY_FREQUENCY: u32> =
    CoreOscillator<PLAY_FREQUENCY, 10, 100, { OscillatorType::PulseWidth as usize }>;

type FrenchHornOscillatorAdsr<const PLAY_FREQUENCY: u32> = AmpMixerCore<
    PLAY_FREQUENCY,
    FrenchHornOscillatorPair<PLAY_FREQUENCY>,
    CoreAdsr<PLAY_FREQUENCY, 0, 3900, 96, 930>,
>;

type FrenchHornFiltered<const PLAY_FREQUENCY: u32> =
    Filter<PLAY_FREQUENCY, FrenchHornOscillatorAdsr<PLAY_FREQUENCY>, 800>;

pub struct FrenchHorn<const PLAY_FREQUENCY: u32> {
    core: FrenchHornFiltered<PLAY_FREQUENCY>,
}

impl<const PLAY_FREQUENCY: u32> SoundSourceCore<PLAY_FREQUENCY> for FrenchHorn<PLAY_FREQUENCY> {
    type InitValuesType = SoundSourceNoteInit;

    fn get_next(self: &mut Self) -> SoundSampleI32 {
        self.core.get_next()
    }

    fn has_next(self: &Self) -> bool {
        self.core.has_next()
    }

    fn new(init_values: Self::InitValuesType) -> Self {
        let frequency_1 = midi_note_to_freq(init_values.key);
        let adsr_init = (init_values.velocity as i32) << 8;
        let core = FrenchHornFiltered::<PLAY_FREQUENCY>::new((frequency_1, adsr_init));
        return Self { core };
    }

    fn trigger_note_off(self: &mut Self) {
        self.core.trigger_note_off();
    }
}
