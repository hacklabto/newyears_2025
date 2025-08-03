use crate::adsr::CoreAdsr;
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

type CelloOscillatorPair<const P_FREQ: u32, const U_FREQ: u32> = DoubleOscillator<
    P_FREQ,
    U_FREQ,
    CoreOscillator<P_FREQ, U_FREQ, 10, 70, { OscillatorType::PulseWidth as usize }>,
    CoreOscillator<P_FREQ, U_FREQ, 50, 70, { OscillatorType::PulseWidth as usize }>,
    false,
>;

type CelloOscillatorLfo<const P_FREQ: u32, const U_FREQ: u32> = LfoAmplitude<
    P_FREQ,
    U_FREQ,
    CelloOscillatorPair<P_FREQ, U_FREQ>,
    { OscillatorType::Sine as usize },
    { 15 * FREQUENCY_MULTIPLIER / 2 },
    5,
>;

type CelloOscillatorAdsr<const P_FREQ: u32, const U_FREQ: u32> =
    CoreAdsr<P_FREQ, U_FREQ, 006, 5000, 100, 300, CelloOscillatorLfo<P_FREQ, U_FREQ>>;

type CelloFiltered<const P_FREQ: u32, const U_FREQ: u32> =
    Filter<P_FREQ, U_FREQ, CelloOscillatorAdsr<P_FREQ, U_FREQ>, 1000>;

///
/// Cello.  Now sort of a proof of concept.
///
pub struct Cello<const P_FREQ: u32, const U_FREQ: u32> {
    core: CelloFiltered<P_FREQ, U_FREQ>,
}

impl<const P_FREQ: u32, const U_FREQ: u32> SoundSourceCore<P_FREQ, U_FREQ>
    for Cello<P_FREQ, U_FREQ>
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
        let frequency_2 = midi_note_to_freq(init_values.key);
        let adsr_init = (init_values.velocity as i32) << 8;
        let core = CelloFiltered::<P_FREQ, U_FREQ>::new(((frequency_1, frequency_2), adsr_init));
        return Self { core };
    }

    fn trigger_note_off(self: &mut Self) {
        self.core.trigger_note_off();
    }

    fn restart(self: &mut Self, vel: u8) {
        self.core.restart(vel);
    }
}
