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

type ViolinOscillatorPair<const P_FREQ: u32, const U_FREQ: u32> = DoubleOscillator<
    P_FREQ,
    U_FREQ,
    CoreOscillator<P_FREQ, U_FREQ, 50, 1, { OscillatorType::PulseWidth as usize }>,
    CoreOscillator<P_FREQ, U_FREQ, 50, 100, { OscillatorType::SawTooth as usize }>,
    true,
>;

type ViolinOscillatorLfo<const P_FREQ: u32, const U_FREQ: u32> = LfoAmplitude<
    P_FREQ,
    U_FREQ,
    ViolinOscillatorPair<P_FREQ, U_FREQ>,
    { OscillatorType::Triangle as usize },
    { 6 * FREQUENCY_MULTIPLIER / 2 },
    10,
>;

type ViolinOscillatorAdsr<const P_FREQ: u32, const U_FREQ: u32> =
    CoreAdsr<P_FREQ, U_FREQ, 03, 5000, 100, 350, ViolinOscillatorLfo<P_FREQ, U_FREQ>>;

type ViolinFiltered<const P_FREQ: u32, const U_FREQ: u32> =
    Filter<P_FREQ, U_FREQ, ViolinOscillatorAdsr<P_FREQ, U_FREQ>>;

///
/// Violin.  Now sort of a proof of concept.
///
pub struct Violin<const P_FREQ: u32, const U_FREQ: u32> {
    core: ViolinFiltered<P_FREQ, U_FREQ>,
}

impl<const P_FREQ: u32, const U_FREQ: u32> SoundSourceCore<P_FREQ, U_FREQ>
    for Violin<P_FREQ, U_FREQ>
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
        let frequency_2 = midi_note_to_freq(init_values.key + 6);
        let adsr_init = (init_values.velocity as i32) << 8;
        let core =
            ViolinFiltered::<P_FREQ, U_FREQ>::new((((frequency_1, frequency_2), adsr_init), 1900));
        return Self { core };
    }

    fn trigger_note_off(self: &mut Self) {
        self.core.trigger_note_off();
    }

    fn restart(self: &mut Self, vel: u8) {
        self.core.restart(vel);
    }
}
