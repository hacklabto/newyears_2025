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

type ElectricPianoOscillatorPair<const PLAY_FREQUENCY: u32> = DoubleOscillator<
    PLAY_FREQUENCY,
    CoreOscillator<PLAY_FREQUENCY, 50, 100, { OscillatorType::SawTooth as usize }>,
    CoreOscillator<PLAY_FREQUENCY, 5, 100, { OscillatorType::PulseWidth as usize }>,
    true,
>;

type ElectricPianoOscillatorAdsr<const PLAY_FREQUENCY: u32> = AmpMixerCore<
    PLAY_FREQUENCY,
    ElectricPianoOscillatorPair<PLAY_FREQUENCY>,
    CoreAdsr<PLAY_FREQUENCY, 0, 5140, 50, 660>,
>;

type ElectricPianoFiltered<const PLAY_FREQUENCY: u32> =
    Filter<PLAY_FREQUENCY, ElectricPianoOscillatorAdsr<PLAY_FREQUENCY>, 200>;

pub struct ElectricPiano<const PLAY_FREQUENCY: u32> {
    core: ElectricPianoFiltered<PLAY_FREQUENCY>,
}

impl<const PLAY_FREQUENCY: u32> SoundSourceCore<PLAY_FREQUENCY> for ElectricPiano<PLAY_FREQUENCY> {
    type InitValuesType = SoundSourceNoteInit;

    fn get_next(self: &mut Self) -> SoundSampleI32 {
        self.core.get_next()
    }

    fn has_next(self: &Self) -> bool {
        self.core.has_next()
    }

    fn new(init_values: Self::InitValuesType) -> Self {
        let frequency_1 = midi_note_to_freq(init_values.key);
        let frequency_2 = midi_note_to_freq(init_values.key + 24 + 9);
        let adsr_init = (init_values.velocity as i32) << 8;
        let core =
            ElectricPianoFiltered::<PLAY_FREQUENCY>::new(((frequency_1, frequency_2), adsr_init));
        return Self { core };
    }

    fn trigger_note_off(self: &mut Self) {
        self.core.trigger_note_off();
    }
}
