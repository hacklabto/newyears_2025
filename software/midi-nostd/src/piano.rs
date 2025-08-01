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

type PianoOscillatorPair<const P_FREQ: u32, const U_FREQ: u32> = DoubleOscillator<
    P_FREQ,
    U_FREQ,
    CoreOscillator<P_FREQ, U_FREQ, 50, 75, { OscillatorType::SawTooth as usize }>,
    CoreOscillator<P_FREQ, U_FREQ, 15, 100, { OscillatorType::PulseWidth as usize }>,
    true,
>;

type PianoOscillatorAdsr<const P_FREQ: u32, const U_FREQ: u32> = AmpMixerCore<
    P_FREQ,
    U_FREQ,
    PianoOscillatorPair<P_FREQ, U_FREQ>,
    CoreAdsr<P_FREQ, U_FREQ, 0, 670, 25, 500>,
>;

type PianoFiltered<const P_FREQ: u32, const U_FREQ: u32> =
    Filter<P_FREQ, U_FREQ, PianoOscillatorAdsr<P_FREQ, U_FREQ>, 1000>;

///
/// Piano.  Now sort of a proof of concept.
///
pub struct Piano<const P_FREQ: u32, const U_FREQ: u32> {
    core: PianoFiltered<P_FREQ, U_FREQ>,
}

impl<const P_FREQ: u32, const U_FREQ: u32> SoundSourceCore<P_FREQ, U_FREQ>
    for Piano<P_FREQ, U_FREQ>
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
        let frequency_2 = midi_note_to_freq(init_values.key + 16);
        let adsr_init = (init_values.velocity as i32) << 8;
        let core = PianoFiltered::<P_FREQ, U_FREQ>::new(((frequency_1, frequency_2), adsr_init));
        return Self { core };
    }

    fn trigger_note_off(self: &mut Self) {
        self.core.trigger_note_off();
    }
}
