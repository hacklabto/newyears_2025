use crate::adsr::CoreAdsr;
use crate::filter::Filter;
use crate::midi_notes::midi_note_to_freq;
use crate::note::SoundSourceNoteInit;
use crate::oscillator::CoreOscillator;
use crate::oscillator::OscillatorType;
use crate::sound_sample::SoundSampleI32;
use crate::sound_source_core::SoundSourceCore;

type FrenchHornOscillatorPair<const P_FREQ: u32, const U_FREQ: u32> =
    CoreOscillator<P_FREQ, U_FREQ, 10, 100, { OscillatorType::PulseWidth as usize }>;

type FrenchHornOscillatorAdsr<const P_FREQ: u32, const U_FREQ: u32> =
    CoreAdsr<P_FREQ, U_FREQ, 0, 3900, 96, 930, FrenchHornOscillatorPair<P_FREQ, U_FREQ>>;

type FrenchHornFiltered<const P_FREQ: u32, const U_FREQ: u32> =
    Filter<P_FREQ, U_FREQ, FrenchHornOscillatorAdsr<P_FREQ, U_FREQ>, 800>;

pub struct FrenchHorn<const P_FREQ: u32, const U_FREQ: u32> {
    core: FrenchHornFiltered<P_FREQ, U_FREQ>,
}

impl<const P_FREQ: u32, const U_FREQ: u32> SoundSourceCore<P_FREQ, U_FREQ>
    for FrenchHorn<P_FREQ, U_FREQ>
{
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
        let adsr_init = (init_values.velocity as i32) << 8;
        let core = FrenchHornFiltered::<P_FREQ, U_FREQ>::new((frequency_1, adsr_init));
        return Self { core };
    }

    fn trigger_note_off(self: &mut Self) {
        self.core.trigger_note_off();
    }
}
